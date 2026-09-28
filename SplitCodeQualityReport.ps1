<#
.SYNOPSIS
    Replaces a code quality report with one report for AI coding agents and one for developers.

.DESCRIPTION
    Reads a code quality report in the format described below, writes two reports next to it
    and then removes it:

        <report name>_FOR_AI_CODING_AGENTS.md   findings an AI coding agent should apply
        <report name>_FOR_DEVELOPERS.md         findings that need a developer

    From then on, the two split reports are the only record of their findings, so no copy can
    drift from them. They keep the text and order of the report. A level-1 or level-2 section
    appears only in a split report that holds a finding below it. Sections without findings,
    such as the introduction and the resolution order, appear in both. A notice below the
    title names the other split report, where a finding referred to may be.

    Because the split reports are the record, the script never overwrites one, and it refuses
    to split a report whose name ends like a split report's. When the report breaks the
    format, the script lists every problem and fails without writing or removing anything.
    Run it with -WhatIf to check a report without writing or removing anything.

    The prompt in .ai/CODE_QUALITY_REPORT_PROMPT.md asks AI agents to write reports in this
    format. Change both together.

    REPORT FORMAT

    The report is Markdown. Only headings of levels 1 to 3 at the start of a line ("#", "##"
    or "###" followed by a space) give it structure. Deeper headings and everything inside
    fenced code blocks (``` or ~~~) are content.

    #     The title on the first line, for example "# Code Quality Report"; a group of files,
          for example "# Platform Assembly (`Assets/Scripts/Platform`)"; or a section without
          findings, for example "# Recommended Resolution Order".
    ##    A file, a group of files or a cross-cutting pattern, with an optional summary; or a
          section without findings, for example "## How To Read This Report".
    ###   A finding. Every level-3 heading is a finding, and findings use no other level.

    A finding consists of these paragraphs, in this order:

        ### `<file name>`: `<type, member or concept>`

        **Violates:** <principles>

        <description of the violations>

        Recommendation: <recommendation, which may continue over paragraphs and lists>

        Developer Decision: The recommendations should be applied by <audience>.

    The finding ends with its Developer Decision line, which names the audience in one of
    exactly two ways:

        Developer Decision: The recommendations should be applied by a developer.
        Developer Decision: The recommendations should be applied by an AI coding agent.

    The audience is a developer when fixing the violation needs a human decision on how to fix
    it: the recommendation names several ways, asks to decide, choose or consider something,
    needs a check whose result changes the fix, or depends on a choice still open in another
    finding. Whether to fix the violation at all does not count. The audience is an AI coding
    agent when the recommendation prescribes exactly one fix. The prompt lists the criteria in
    detail.

    Developers record their decisions before the split. They write the decision in front of
    "The recommendations", rewrite the recommendation to follow it, and switch the audience to
    an AI coding agent unless another decision is still open:

        Developer Decision: Health pickup is obsolete. The recommendations should be applied by an AI coding agent.

    After the split, a developer who decides a finding moves it by hand into the report for AI
    coding agents, under the same level-1 and level-2 headings.

    A line "---" with a blank line on each side may stand directly in front of a heading. It
    stays in front of that heading in the split reports.

    The script stops without writing or removing anything when
    - the report's name ends like a split report's;
    - the report does not start with its title;
    - a finding has no Developer Decision line, or more than one;
    - a Developer Decision line differs from the forms above, for example when it is bold or
      names another audience;
    - a Developer Decision line stands outside a finding;
    - a fenced code block is never closed;
    - a split report already exists. With -WhatIf this is only a warning, so that a new
      report's format can be checked while an earlier report's split reports are in use.

.PARAMETER ReportPath
    The report to split. Defaults to CODE_QUALITY_REPORT.md in the repository root. The split
    reports are written next to it, and the report is removed once both exist.

.EXAMPLE
    ./SplitCodeQualityReport.ps1
    ./SplitCodeQualityReport.ps1 -WhatIf
    ./SplitCodeQualityReport.ps1 -ReportPath ./OLD_CODE_QUALITY_REPORT.md
#>

[CmdletBinding(SupportsShouldProcess)]
param(
    [string]$ReportPath
)

$ErrorActionPreference = 'Stop'

# Windows PowerShell leaves $PSScriptRoot empty while it binds parameters, so the default is
# resolved here instead of in the param block.
if (-not $ReportPath) {
    $ReportPath = Join-Path (Split-Path -Parent $PSScriptRoot) 'CODE_QUALITY_REPORT.md'
}

$decisionLinePattern = '^Developer Decision: (?:\S.*? )?The recommendations should be applied by (?<Audience>a developer|an AI coding agent)\.\s*$'

# Also matches bold, lower-case and other near misses, so that they are reported instead of
# leaving their finding without a Developer Decision line.
$decisionLikeLinePattern = '^[\s>*_-]*Developer\s+Decision[\s*_]*:'

$headingPattern = '^(?<Marks>#{1,3})[ \t]+\S'
# A backtick fence cannot have backticks after it: "```x```" at the start of a line is inline code.
$fenceOpeningPattern = '^ {0,3}(?:(?<Fence>`{3,})[^`]*|(?<Fence>~{3,}).*)$'
$fenceClosingPattern = '^ {0,3}(?<Fence>`{3,}|~{3,})\s*$'

$audiences = @(
    @{ Name = 'an AI coding agent'; FileSuffix = '_FOR_AI_CODING_AGENTS' }
    @{ Name = 'a developer'; FileSuffix = '_FOR_DEVELOPERS' }
)

function New-ReportSection {
    param(
        [int]$Level,
        [object]$Parent,
        [int]$LineNumber,
        [bool]$HasRuleBefore,
        [bool]$IsTitle
    )

    [pscustomobject]@{
        Level                 = $Level
        Parent                = $Parent
        LineNumber            = $LineNumber
        HasRuleBefore         = $HasRuleBefore
        IsTitle               = $IsTitle
        Lines                 = [System.Collections.Generic.List[string]]::new()
        Children              = [System.Collections.Generic.List[object]]::new()
        DecisionLines         = [System.Collections.Generic.List[object]]::new()
        FindingAudiences      = [System.Collections.Generic.HashSet[string]]::new()
        ContainsSharedSection = $false
    }
}

function Remove-TrailingBlankLines {
    param([System.Collections.Generic.List[string]]$Lines)

    while ($Lines.Count -gt 0 -and [string]::IsNullOrWhiteSpace($Lines[$Lines.Count - 1])) {
        $Lines.RemoveAt($Lines.Count - 1)
    }
}

# A "---" that separates two sections belongs to the heading below it, so that it is written
# only together with that heading. Directly below text, "---" turns that text into a heading
# instead, so only a "---" after a blank line counts.
function Remove-TrailingRule {
    param([System.Collections.Generic.List[string]]$Lines)

    Remove-TrailingBlankLines -Lines $Lines
    $lastIndex = $Lines.Count - 1
    $endsWithRule = $lastIndex -ge 1 -and
        $Lines[$lastIndex].Trim() -eq '---' -and
        [string]::IsNullOrWhiteSpace($Lines[$lastIndex - 1])
    if ($endsWithRule) {
        $Lines.RemoveAt($lastIndex)
        Remove-TrailingBlankLines -Lines $Lines
    }
    return $endsWithRule
}

function Read-Report {
    param([string[]]$Lines)

    $root = New-ReportSection -Level 0 -Parent $null -LineNumber 0 -HasRuleBefore $false -IsTitle $false
    $sections = [System.Collections.Generic.List[object]]::new()
    $sections.Add($root)
    $current = $root
    $openFence = $null

    for ($index = 0; $index -lt $Lines.Count; $index++) {
        $line = $Lines[$index]
        $lineNumber = $index + 1

        if ($openFence) {
            $current.Lines.Add($line)
            if ($line -match $fenceClosingPattern -and
                $Matches.Fence[0] -eq $openFence.Character -and
                $Matches.Fence.Length -ge $openFence.Length) {
                $openFence = $null
            }
            continue
        }

        if ($line -match $fenceOpeningPattern) {
            $openFence = @{ Character = $Matches.Fence[0]; Length = $Matches.Fence.Length; LineNumber = $lineNumber }
            $current.Lines.Add($line)
            continue
        }

        if ($line -match $headingPattern) {
            $level = $Matches.Marks.Length
            $isTitle = $level -eq 1 -and $sections.Count -eq 1
            $hasRuleBefore = Remove-TrailingRule -Lines $current.Lines
            while ($current.Level -ge $level) {
                $current = $current.Parent
            }
            $section = New-ReportSection -Level $level -Parent $current -LineNumber $lineNumber -HasRuleBefore $hasRuleBefore -IsTitle $isTitle
            $section.Lines.Add($line)
            $current.Children.Add($section)
            $sections.Add($section)
            $current = $section
            continue
        }

        if ($line -match $decisionLikeLinePattern) {
            $current.DecisionLines.Add([pscustomobject]@{ Text = $line; LineNumber = $lineNumber })
        }
        $current.Lines.Add($line)
    }

    Remove-TrailingBlankLines -Lines $current.Lines

    [pscustomobject]@{
        Root                    = $root
        Sections                = $sections
        UnclosedFenceLineNumber = if ($openFence) { $openFence.LineNumber } else { 0 }
    }
}

function Get-FormatProblem {
    param([object]$Report)

    $root = $Report.Root
    if ($root.Lines.Count -gt 0 -or $root.Children.Count -eq 0 -or -not $root.Children[0].IsTitle) {
        "Line 1: the report does not start with its title, a level-1 heading such as '# Code Quality Report'."
    }

    foreach ($section in $Report.Sections) {
        if ($section.Level -ne 3) {
            foreach ($decisionLine in $section.DecisionLines) {
                "Line $($decisionLine.LineNumber): this Developer Decision line stands outside a finding. Only '###' sections end with one."
            }
            continue
        }

        if ($section.DecisionLines.Count -ne 1) {
            "Line $($section.LineNumber): $($section.Lines[0]) has $($section.DecisionLines.Count) Developer Decision lines instead of one."
            continue
        }

        $decisionLine = $section.DecisionLines[0]
        if ($decisionLine.Text -cnotmatch $decisionLinePattern) {
            "Line $($decisionLine.LineNumber): '$($decisionLine.Text)' is not one of the two Developer Decision forms."
        }
    }

    if ($Report.UnclosedFenceLineNumber -gt 0) {
        "Line $($Report.UnclosedFenceLineNumber): this code block is never closed, so everything below it counts as code."
    }
}

function Set-SectionAudience {
    param([object]$Report)

    foreach ($section in $Report.Sections) {
        if ($section.Level -ne 3) {
            continue
        }
        $null = $section.DecisionLines[0].Text -cmatch $decisionLinePattern
        $audience = $Matches.Audience
        for ($enclosing = $section; $null -ne $enclosing; $enclosing = $enclosing.Parent) {
            $null = $enclosing.FindingAudiences.Add($audience)
        }
    }

    # Sections are listed in document order, so walking backwards visits children first.
    for ($index = $Report.Sections.Count - 1; $index -ge 0; $index--) {
        $section = $Report.Sections[$index]
        $section.ContainsSharedSection = $section.FindingAudiences.Count -eq 0 -or
            @($section.Children | Where-Object { $_.ContainsSharedSection }).Count -gt 0
    }
}

function Add-SectionText {
    param(
        [object]$Section,
        [string]$Audience,
        [System.Collections.Generic.List[string]]$Output
    )

    $isWritten = $Section.Level -eq 0 -or $Section.IsTitle -or
        $Section.FindingAudiences.Contains($Audience) -or $Section.ContainsSharedSection
    if (-not $isWritten) {
        return
    }

    if ($Section.Lines.Count -gt 0) {
        if ($Output.Count -gt 0) {
            $Output.Add('')
            if ($Section.HasRuleBefore) {
                $Output.Add('---')
                $Output.Add('')
            }
        }
        $Output.AddRange($Section.Lines)
    }

    foreach ($child in $Section.Children) {
        Add-SectionText -Section $child -Audience $Audience -Output $Output
    }
}

function New-AudienceReport {
    param(
        [object]$Report,
        [string]$Audience,
        [string]$Notice,
        [string]$Newline
    )

    $output = [System.Collections.Generic.List[string]]::new()
    Add-SectionText -Section $Report.Root -Audience $Audience -Output $output

    # The first line is the title, which is always written.
    $output.InsertRange(1, [string[]]@('', $Notice))

    ($output -join $Newline) + $Newline
}

$reportFile = $PSCmdlet.GetUnresolvedProviderPathFromPSPath($ReportPath)
if (-not (Test-Path -LiteralPath $reportFile -PathType Leaf)) {
    throw "No report at '$reportFile'. Pass its path with -ReportPath."
}

$reportFileName = Split-Path -Leaf $reportFile
$reportBaseName = [System.IO.Path]::GetFileNameWithoutExtension($reportFile)
foreach ($audience in $audiences) {
    if ($reportBaseName.EndsWith($audience.FileSuffix, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "'$reportFile' is already a split report, and splitting it would remove it."
    }
    $audience.File = Join-Path (Split-Path -Parent $reportFile) ($reportBaseName + $audience.FileSuffix + '.md')
}

# Get-Content in Windows PowerShell reads UTF-8 without a byte order mark as ANSI, which would
# garble every dash and arrow in the report.
$reportText = [System.IO.File]::ReadAllText($reportFile, [System.Text.Encoding]::UTF8)
$newline = if ($reportText.Contains("`r`n")) { "`r`n" } else { "`n" }
$report = Read-Report -Lines ([regex]::Split($reportText, '\r\n|\n|\r'))

$problems = @(Get-FormatProblem -Report $report)
if ($problems.Count -gt 0) {
    foreach ($problem in $problems) {
        Write-Host $problem -ForegroundColor Red
    }
    throw "$reportFile does not match the format that Get-Help $PSCommandPath -Full describes. Nothing was written or removed; fix the problems listed above."
}

# Once the report is gone, the split reports are the only record of their findings, so an
# existing one is never overwritten. -WhatIf only warns, so that a new report's format can be
# checked while an earlier report's split reports are still in use.
$existingSplitReports = @($audiences | ForEach-Object { $_.File } | Where-Object { Test-Path -LiteralPath $_ })
if ($existingSplitReports.Count -gt 0) {
    $message = "The split reports are the record of their findings, so the split does not overwrite $($existingSplitReports -join ' and '). Move them away first if this report replaces them."
    if ($WhatIfPreference) {
        Write-Warning $message
    } else {
        throw $message
    }
}

Set-SectionAudience -Report $report

$findings = @($report.Sections | Where-Object { $_.Level -eq 3 })
if ($findings.Count -eq 0) {
    Write-Warning "$reportFile contains no findings, so both split reports only repeat its other sections."
}

$scriptName = Split-Path -Leaf $PSCommandPath
$utf8WithoutByteOrderMark = New-Object System.Text.UTF8Encoding $false
$allSplitReportsWritten = $true

foreach ($audience in $audiences) {
    $otherSplitReportName = Split-Path -Leaf ($audiences | Where-Object { $_ -ne $audience }).File
    $findingCount = @($findings | Where-Object { $_.FindingAudiences.Contains($audience.Name) }).Count
    $countedFindings = if ($findingCount -eq 1) { '1 finding' } else { "$findingCount findings" }
    $notice = "> This file and ``$otherSplitReportName`` replace ``$reportFileName``, split by ``$scriptName``. This file holds the findings whose recommendations should be applied by $($audience.Name); a finding it refers to may be in the other file."
    $text = New-AudienceReport -Report $report -Audience $audience.Name -Notice $notice -Newline $newline

    if ($PSCmdlet.ShouldProcess($audience.File, "Write $countedFindings for $($audience.Name)")) {
        [System.IO.File]::WriteAllText($audience.File, $text, $utf8WithoutByteOrderMark)
        Write-Host "Wrote $countedFindings for $($audience.Name) to $($audience.File)"
    } else {
        $allSplitReportsWritten = $false
    }
}

# The report goes only once both split reports exist, so that none of its findings is lost.
# File.Delete instead of Remove-Item, which would ask a second time under -Confirm.
if (($allSplitReportsWritten -or $WhatIfPreference) -and
    $PSCmdlet.ShouldProcess($reportFile, 'Remove the report, which the split reports replace')) {
    [System.IO.File]::Delete($reportFile)
    Write-Host "Removed $reportFile, which the split reports replace."
}
