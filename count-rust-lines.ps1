# Get all .rs files recursively from the current directory
$files = Get-ChildItem -Path . -Filter "*.rs" -Recurse -File | Where-Object {
    $_.FullName -like "*\src\*"
} | ForEach-Object {
    # Count lines in each file
    $lineCount = (Get-Content $_.FullName | Measure-Object -Line).Lines
    
    # Create a custom object with line count and relative path
    $relativePath = $_.FullName.Replace("$PWD\", "")
    $firstDir = $relativePath.Split('\')[0]
    $pathWithoutFirstDir = $relativePath.Substring($firstDir.Length + 1)
    
    # Remove 'src\' prefix if present
    if ($pathWithoutFirstDir.StartsWith("src\")) {
        $pathWithoutFirstDir = $pathWithoutFirstDir.Substring(4)
    }
    
    # Extract second directory (first segment after removing first dir and src)
    $secondDir = if ($pathWithoutFirstDir.Contains('\')) {
        $pathWithoutFirstDir.Split('\')[0]
    } else {
        ""
    }
    
    [PSCustomObject]@{
        Lines = $lineCount
        Path = $relativePath
        FirstDir = $firstDir
        SecondDir = $secondDir
        PathWithoutFirstDir = $pathWithoutFirstDir
    }
} | Sort-Object -Property FirstDir, SecondDir, @{Expression = {$_.Lines}; Descending = $true}

# Find the maximum line count length for padding
$maxLineLength = ($files | ForEach-Object { $_.Lines.ToString().Length } | Measure-Object -Maximum).Maximum

# Print with aligned paths, grouped by first directory
$currentGroup = $null
$files | ForEach-Object {
    # Print group header when directory changes
    if ($currentGroup -ne $_.FirstDir) {
        if ($currentGroup -ne $null) {
            Write-Output ""
        }
        Write-Output "$($_.FirstDir):"
        $currentGroup = $_.FirstDir
    }
    
    # Print in the format: {lines_of_code} {path_without_first_dir} with padding
    $paddedLines = $_.Lines.ToString().PadLeft($maxLineLength)
    Write-Output "$paddedLines $($_.PathWithoutFirstDir)"
}

# Add final newline
Write-Output ""
