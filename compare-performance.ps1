<#
.SYNOPSIS
    Compares how long `.\x.ps1 test` takes on a baseline commit versus the currently checked out revision.

.DESCRIPTION
    Checking out the baseline commit removes this script from the working tree, so when it is started
    from inside the repository it copies itself next to the repository and relaunches from there.

.PARAMETER BaselineCommit
    The commit to compare the current revision against.

.EXAMPLE
    .\compare-performance.ps1 d1389bd
#>

param(
    [Parameter(Mandatory = $true, Position = 0)]
    [string]$BaselineCommit
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# Exit codes of git and cargo are inspected explicitly, so a non-zero one must not terminate on its own.
$PSNativeCommandUseErrorActionPreference = $false

$RepositoryDirectoryName = "SpacetimeDSL"
$MeasuredIterationCount = 20

# The smallest difference the two-decimal report can still show as a non-zero number of seconds.
$ReportedSecondsResolution = 0.005

function Get-RunningPowerShellPath {
    (Get-Process -Id $PID).Path
}

function Test-RunningNextToRepository {
    Test-Path -LiteralPath (Join-Path -Path (Get-Location) -ChildPath $RepositoryDirectoryName) -PathType Container
}

function Start-NextToRepository {
    $parentDirectory = Split-Path -Path $PSScriptRoot -Parent
    $relaunchedScriptPath = Join-Path -Path $parentDirectory -ChildPath (Split-Path -Path $PSCommandPath -Leaf)

    Copy-Item -LiteralPath $PSCommandPath -Destination $relaunchedScriptPath -Force

    Write-Host "Relaunching from $parentDirectory, where a checkout of $BaselineCommit cannot remove this script."

    Start-Process -FilePath (Get-RunningPowerShellPath) `
        -WorkingDirectory $parentDirectory `
        -ArgumentList @(
            "-NoExit",
            "-NoProfile",
            "-File", "`"$relaunchedScriptPath`"",
            "`"$BaselineCommit`""
        )
}

function Invoke-Git {
    param([string[]]$Arguments)

    $output = & git @Arguments 2>&1
    if ($LASTEXITCODE -ne 0) {
        throw "'git $($Arguments -join ' ')' failed: $output"
    }
    $output
}

function Assert-NoTrackedChanges {
    $trackedChanges = Invoke-Git -Arguments @("status", "--porcelain", "--untracked-files=no")
    if ($trackedChanges) {
        throw "The repository has uncommitted changes in tracked files. Commit or stash them before comparing:`n$($trackedChanges -join "`n")"
    }
}

function Get-CheckedOutRevision {
    $branchName = & git symbolic-ref --quiet --short HEAD 2>&1
    if ($LASTEXITCODE -eq 0 -and $branchName) {
        return "$branchName".Trim()
    }

    # A detached HEAD has no branch name, so the commit itself is what has to be restored.
    "$(Invoke-Git -Arguments @('rev-parse', 'HEAD'))".Trim()
}

function Switch-Revision {
    param([string]$Revision)

    Invoke-Git -Arguments @("checkout", $Revision) | Out-Null
}

function Invoke-TestCommand {
    # `x.ps1` ignores the exit codes of the commands it runs, so a failing `spacetime publish` would
    # otherwise be hidden behind the exit code of the `spacetime delete` that follows it. A child
    # process turns any non-zero exit code into a terminating error and keeps the `Set-Location`
    # calls of `x.ps1` from leaking into the next iteration.
    & (Get-RunningPowerShellPath) -NoProfile -Command @"
`$ErrorActionPreference = 'Stop'
`$PSNativeCommandUseErrorActionPreference = `$true
.\x.ps1 test
"@ *>&1 | Out-Null

    if ($LASTEXITCODE -ne 0) {
        throw "'.\x.ps1 test' failed with exit code $LASTEXITCODE."
    }
}

function Measure-TestCommand {
    $startTimestamp = Get-Date
    Invoke-TestCommand
    $endTimestamp = Get-Date

    $endTimestamp - $startTimestamp
}

function Format-Seconds {
    param([double]$Seconds)

    # Formatting stays culture-invariant so the report reads the same on any machine locale.
    [string]::Format([cultureinfo]::InvariantCulture, "{0,8:n2} s", $Seconds)
}

function Measure-Revision {
    param([string]$RevisionLabel)

    Write-Host ""
    Write-Host "--- $RevisionLabel ---"

    Write-Host "Removing build artifacts..."
    # Output has to be discarded, or it would end up in the durations this function returns.
    & cargo clean *>&1 | Out-Null
    if ($LASTEXITCODE -ne 0) {
        throw "'cargo clean' failed for $RevisionLabel."
    }

    Write-Host "Warming up..."
    $warmUpDuration = Measure-TestCommand
    Write-Host "Warm-up run:$(Format-Seconds $warmUpDuration.TotalSeconds) (not counted)"

    $durations = @()
    for ($iteration = 1; $iteration -le $MeasuredIterationCount; $iteration++) {
        $duration = Measure-TestCommand
        $durations += $duration
        Write-Host ("Run {0,2}/{1}:{2}" -f $iteration, $MeasuredIterationCount, (Format-Seconds $duration.TotalSeconds))
    }

    , $durations
}

function Get-DurationStatistics {
    param([TimeSpan[]]$Durations)

    $measurement = $Durations | ForEach-Object { $_.TotalSeconds } |
        Measure-Object -Sum -Average -Minimum -Maximum

    [PSCustomObject]@{
        Total   = $measurement.Sum
        Average = $measurement.Average
        Minimum = $measurement.Minimum
        Maximum = $measurement.Maximum
        Spread  = $measurement.Maximum - $measurement.Minimum
    }
}

function Write-RevisionReport {
    param(
        [string]$RevisionLabel,
        [PSCustomObject]$Statistics
    )

    Write-Host ""
    Write-Host $RevisionLabel
    Write-Host "  Total  :$(Format-Seconds $Statistics.Total)  over $MeasuredIterationCount runs"
    Write-Host "  Average:$(Format-Seconds $Statistics.Average)"
    Write-Host "  Fastest:$(Format-Seconds $Statistics.Minimum)"
    Write-Host "  Slowest:$(Format-Seconds $Statistics.Maximum)"
    Write-Host "  Spread :$(Format-Seconds $Statistics.Spread)"
}

function Write-ComparisonReport {
    param(
        [string]$BaselineLabel,
        [TimeSpan[]]$BaselineDurations,
        [string]$CurrentLabel,
        [TimeSpan[]]$CurrentDurations
    )

    $baselineStatistics = Get-DurationStatistics -Durations $BaselineDurations
    $currentStatistics = Get-DurationStatistics -Durations $CurrentDurations

    Write-Host ""
    Write-Host "================ Result ================"
    Write-RevisionReport -RevisionLabel $BaselineLabel -Statistics $baselineStatistics
    Write-RevisionReport -RevisionLabel $CurrentLabel -Statistics $currentStatistics

    $difference = $currentStatistics.Total - $baselineStatistics.Total
    $percentage = if ($baselineStatistics.Total -gt 0) { 100 * $difference / $baselineStatistics.Total } else { 0 }
    $direction = if ($difference -lt 0) { "faster" } else { "slower" }

    Write-Host ""
    if ([Math]::Abs($difference) -lt $ReportedSecondsResolution) {
        # A percentage of a difference too small to render would read as "0.00 s (0.14 %) slower".
        Write-Host "$CurrentLabel and $BaselineLabel take the same total time at the precision reported here."
    }
    else {
        Write-Host ([string]::Format([cultureinfo]::InvariantCulture,
            "{0} is{1} ({2:n2} %) {3} in total than {4}.",
            $CurrentLabel, (Format-Seconds ([Math]::Abs($difference))), [Math]::Abs($percentage), $direction, $BaselineLabel))
    }
    Write-Host ""
}

function Compare-Performance {
    Set-Location -LiteralPath (Join-Path -Path (Get-Location) -ChildPath $RepositoryDirectoryName)

    Assert-NoTrackedChanges
    $currentRevision = Get-CheckedOutRevision

    $baselineLabel = "baseline $BaselineCommit"
    $currentLabel = "current $currentRevision"

    Write-Host "Comparing '.\x.ps1 test' on $baselineLabel against $currentLabel."

    try {
        Switch-Revision -Revision $BaselineCommit
        $baselineDurations = Measure-Revision -RevisionLabel $baselineLabel
    }
    finally {
        Switch-Revision -Revision $currentRevision
    }

    $currentDurations = Measure-Revision -RevisionLabel $currentLabel

    Write-ComparisonReport -BaselineLabel $baselineLabel -BaselineDurations $baselineDurations `
        -CurrentLabel $currentLabel -CurrentDurations $currentDurations
}

if (Test-RunningNextToRepository) {
    Compare-Performance
}
else {
    Start-NextToRepository
}
