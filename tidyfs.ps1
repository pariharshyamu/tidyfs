param(
    [Parameter(Position=0)]
    [string]$Command,
    
    [Parameter(Position=1)]
    [string]$Directory = ".",
    
    [Parameter()]
    [switch]$Recursive,
    
    [Parameter()]
    [switch]$Duplicates,
    
    [Parameter()]
    [string]$Target,
    
    [Parameter()]
    [string]$By = "type",
    
    [Parameter()]
    [switch]$DryRun
)

# Function to get file category based on extension
function Get-FileCategory {
    param([string]$Extension)
    
    $ext = $Extension.ToLower()
    
    switch -Regex ($ext) {
        '^(pdf|doc|docx|txt|rtf|odt|md|xls|xlsx|ppt|pptx)$' { return "Documents" }
        '^(jpg|jpeg|png|gif|bmp|tiff|svg|webp)$' { return "Images" }
        '^(mp4|avi|mov|wmv|flv|mkv|webm)$' { return "Videos" }
        '^(mp3|wav|ogg|flac|aac|m4a)$' { return "Audio" }
        '^(zip|rar|7z|tar|gz|bz2|xz)$' { return "Archives" }
        '^(rs|py|js|html|css|java|c|cpp|h|go|rb|php|sh)$' { return "Code" }
        '^(exe|msi|app|dmg|deb|rpm)$' { return "Executables" }
        default { return "Other" }
    }
}

# Function to format file size
function Format-FileSize {
    param([long]$Size)
    
    if ($Size -ge 1GB) {
        return "{0:N2} GB" -f ($Size / 1GB)
    } elseif ($Size -ge 1MB) {
        return "{0:N2} MB" -f ($Size / 1MB)
    } elseif ($Size -ge 1KB) {
        return "{0:N2} KB" -f ($Size / 1KB)
    } else {
        return "$Size bytes"
    }
}

# Function to scan directory
function Scan-Directory {
    param(
        [string]$Path,
        [bool]$FindDuplicates,
        [bool]$IsRecursive
    )
    
    Write-Host "Scanning directory: $Path" -ForegroundColor Green
    
    # Get files
    $searchOption = if ($IsRecursive) { "AllDirectories" } else { "TopDirectoryOnly" }
    $files = Get-ChildItem -Path $Path -File -Recurse:$IsRecursive
    
    if ($files.Count -eq 0) {
        Write-Host "No files found in the specified directory." -ForegroundColor Yellow
        return
    }
    
    # Process files
    $fileInfos = @()
    $categories = @{}
    $totalSize = 0
    
    foreach ($file in $files) {
        $category = Get-FileCategory -Extension $file.Extension.TrimStart('.')
        
        $fileInfo = @{
            Path = $file.FullName
            Size = $file.Length
            LastModified = $file.LastWriteTime
            Category = $category
        }
        
        # Add to category statistics
        if (-not $categories.ContainsKey($category)) {
            $categories[$category] = @{
                Count = 0
                Size = 0
            }
        }
        
        $categories[$category].Count++
        $categories[$category].Size += $file.Length
        $totalSize += $file.Length
        
        # Add hash for duplicate detection if needed
        if ($FindDuplicates) {
            $hash = (Get-FileHash -Path $file.FullName -Algorithm SHA256).Hash
            $fileInfo.Hash = $hash
        }
        
        $fileInfos += $fileInfo
    }
    
    # Display report
    Write-Host "`nStorage Usage Report" -ForegroundColor Cyan
    Write-Host "Total: $($files.Count) files, $(Format-FileSize -Size $totalSize)" -ForegroundColor White
    
    Write-Host "`nCategory              Size             Files     % of Total" -ForegroundColor White
    Write-Host "----------------------------------------------------------------"
    
    $sortedCategories = $categories.GetEnumerator() | Sort-Object { $_.Value.Size } -Descending
    
    foreach ($cat in $sortedCategories) {
        $percentage = ($cat.Value.Size / $totalSize) * 100
        $sizeFormatted = Format-FileSize -Size $cat.Value.Size
        Write-Host ("{0,-20} {1,-15} {2,-10} {3,4:N1}%" -f $cat.Key, $sizeFormatted, $cat.Value.Count, $percentage)
    }
    
    # Display largest files
    Write-Host "`nLargest Files:" -ForegroundColor Cyan
    $largestFiles = $fileInfos | Sort-Object { $_.Size } -Descending | Select-Object -First 5
    
    foreach ($file in $largestFiles) {
        Write-Host "$($file.Path) ($(Format-FileSize -Size $file.Size))"
    }
    
    # Find duplicates if requested
    if ($FindDuplicates) {
        $duplicates = $fileInfos | Where-Object { $_.Hash } | Group-Object -Property Hash | Where-Object { $_.Count -gt 1 }
        
        if ($duplicates.Count -eq 0) {
            Write-Host "`nNo duplicate files found." -ForegroundColor Green
        } else {
            $totalDuplicates = ($duplicates | Measure-Object -Property Count -Sum).Sum - $duplicates.Count
            $wastedSpace = 0
            
            foreach ($group in $duplicates) {
                $firstFileSize = ($group.Group | Select-Object -First 1).Size
                $wastedSpace += $firstFileSize * ($group.Count - 1)
            }
            
            Write-Host "`nDuplicate Files Found ($totalDuplicates duplicate files in $($duplicates.Count) groups, wasting $(Format-FileSize -Size $wastedSpace))" -ForegroundColor Yellow
            
            # Display top 5 duplicate groups
            $sortedDuplicates = $duplicates | Sort-Object { 
                $firstFileSize = ($_.Group | Select-Object -First 1).Size
                $firstFileSize * ($_.Count - 1)
            } -Descending | Select-Object -First 5
            
            $groupNumber = 1
            foreach ($group in $sortedDuplicates) {
                $firstFileSize = ($group.Group | Select-Object -First 1).Size
                $wasted = $firstFileSize * ($group.Count - 1)
                
                Write-Host "`nGroup $groupNumber - $($group.Count - 1) duplicates, wasting $(Format-FileSize -Size $wasted):" -ForegroundColor Yellow
                foreach ($file in $group.Group) {
                    Write-Host "  $($file.Path)"
                }
                
                $groupNumber++
            }
            
            if ($duplicates.Count -gt 5) {
                Write-Host "`n... and $($duplicates.Count - 5) more duplicate groups"
            }
        }
    }
}

# Function to organize files
function Organize-Files {
    param(
        [string]$Path,
        [string]$TargetDir,
        [string]$OrganizationType,
        [bool]$IsDryRun,
        [bool]$IsRecursive
    )
    
    Write-Host "Organizing files in $Path by $OrganizationType$(if ($IsDryRun) { " (DRY RUN)" })" -ForegroundColor Green
    
    # Get files
    $searchOption = if ($IsRecursive) { "AllDirectories" } else { "TopDirectoryOnly" }
    $files = Get-ChildItem -Path $Path -File -Recurse:$IsRecursive
    
    if ($files.Count -eq 0) {
        Write-Host "No files found in the specified directory." -ForegroundColor Yellow
        return
    }
    
    $moveCount = 0
    $errorCount = 0
    
    foreach ($file in $files) {
        $targetSubdir = switch ($OrganizationType) {
            "type" { 
                Get-FileCategory -Extension $file.Extension.TrimStart('.')
            }
            "date" { 
                $file.LastWriteTime.ToString("yyyy-MM")
            }
            "ext" { 
                if ($file.Extension) {
                    $file.Extension.TrimStart('.')
                } else {
                    "no_extension"
                }
            }
            default { "Unsorted" }
        }
        
        $targetPath = Join-Path -Path $TargetDir -ChildPath $targetSubdir
        Write-Host "Moving to $targetSubdir`: $($file.Name)"
        
        if (-not $IsDryRun) {
            try {
                # Create target directory if it doesn't exist
                if (-not (Test-Path -Path $targetPath -PathType Container)) {
                    New-Item -Path $targetPath -ItemType Directory -Force | Out-Null
                }
                
                $destination = Join-Path -Path $targetPath -ChildPath $file.Name
                
                # Handle name collision
                if (Test-Path -Path $destination) {
                    $fileNameWithoutExt = [System.IO.Path]::GetFileNameWithoutExtension($file.Name)
                    $fileExt = $file.Extension
                    $timestamp = Get-Date -Format "yyyyMMddHHmmss"
                    $newName = "$fileNameWithoutExt`_$timestamp$fileExt"
                    $destination = Join-Path -Path $targetPath -ChildPath $newName
                }
                
                Move-Item -Path $file.FullName -Destination $destination -Force
                $moveCount++
            } catch {
                Write-Host "Error moving file $($file.Name): $_" -ForegroundColor Red
                $errorCount++
            }
        }
    }
    
    if ($IsDryRun) {
        Write-Host "`nDry run complete. No files were moved." -ForegroundColor Yellow
    } else {
        Write-Host "`nOrganization complete. Moved $moveCount files with $errorCount errors" -ForegroundColor Green
    }
}

# Main script
switch ($Command) {
    "scan" {
        Scan-Directory -Path $Directory -FindDuplicates $Duplicates -IsRecursive $Recursive
    }
    "organize" {
        $targetDirectory = if ($Target) { $Target } else { $Directory }
        Organize-Files -Path $Directory -TargetDir $targetDirectory -OrganizationType $By -IsDryRun $DryRun -IsRecursive $Recursive
    }
    default {
        Write-Host "TidyFS - Smart File System Organizer" -ForegroundColor Green
        Write-Host "Run with a subcommand to begin:"
        Write-Host "  scan - Scan directory and show statistics" -ForegroundColor Cyan
        Write-Host "  organize - Organize files into folders" -ForegroundColor Cyan
        Write-Host "`nExamples:"
        Write-Host "  .\tidyfs.ps1 scan . -Recursive -Duplicates" 
        Write-Host "  .\tidyfs.ps1 organize . -By type -Target D:\Organized"
        Write-Host "`nUse -help with any subcommand for more information."
    }
}
