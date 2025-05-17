# TidyFS Usage Examples

This document provides practical examples of how to use TidyFS in various scenarios.

## Scenario 1: Clean Up a Messy Downloads Folder

Over time, your Downloads folder has become chaotic with hundreds of files of different types. Here's how to clean it up:

```bash
# First, analyze what's in there
tidyfs scan ~/Downloads -r

# Check for any duplicate files that can be removed
tidyfs scan ~/Downloads -r -d

# Do a dry run to see how the organization would look
tidyfs organize ~/Downloads -n -b type

# Organize files by type
tidyfs organize ~/Downloads -b type
```

After running these commands, your Downloads folder will have subdirectories like Documents, Images, Videos, etc., with your files neatly organized.

## Scenario 2: Prepare Files for Archiving by Date

You want to archive your project files based on when they were last modified:

```bash
# Create an archive directory
mkdir ~/Archive

# Scan your project directory
tidyfs scan ~/Projects -r

# Organize files into the archive by date (year-month)
tidyfs organize ~/Projects -t ~/Archive -b date -r
```

Now your files will be organized in folders like "2023-05", "2023-06", etc., making it easy to find files from a specific time period.

## Scenario 3: Custom Organization for Media Files

You have a large collection of media files and want to organize them with custom categories:

```bash
# Add custom categories for different media types
tidyfs config --add-category "Photos:jpg,jpeg,png,heic,raw,cr2,nef"
tidyfs config --add-category "Movies:mp4,mkv,avi,mov"
tidyfs config --add-category "Music:mp3,flac,wav,aac,ogg"

# Scan your media directory
tidyfs scan ~/Media -r

# Organize with your custom categories
tidyfs organize ~/Media -r
```

## Scenario 4: Find Large Files Consuming Disk Space

You're running low on disk space and need to find what's taking up space:

```bash
# Scan your home directory (this may take a while)
tidyfs scan ~ -r

# The output will show categories and the largest files
```

TidyFS will display the largest files found, making it easy to identify space-hogging files that you might want to delete or move.

## Scenario 5: Preparing a USB Drive for Sharing

You want to organize files on a USB drive before sharing it with someone:

```bash
# Assuming the USB is mounted at /media/usb
tidyfs scan /media/usb

# Clean up by finding duplicates
tidyfs scan /media/usb -d

# Organize by file type for easy browsing
tidyfs organize /media/usb -b type
```

## Scenario 6: Managing Project Files by Extension

For a development project with many different file types:

```bash
# Add a custom configuration for your project
tidyfs config --add-category "Source:rs,go,py,js,ts"
tidyfs config --add-category "Config:json,yaml,toml,ini"
tidyfs config --add-category "Docs:md,txt,pdf,docx"

# Organize your project directory
tidyfs organize ~/projects/my-project -b ext
```

This will organize files based on their extension, which is useful for development projects.

## Scenario 7: Creating a Monthly Backup Strategy

You want to back up files modified in the current month:

```bash
# Create a backup directory for the current month
MONTH=$(date +%Y-%m)
mkdir -p ~/Backups/$MONTH

# Find recently modified files
tidyfs scan ~/Documents -r

# Organize recent files into the backup directory
tidyfs organize ~/Documents -t ~/Backups/$MONTH -b date -r
```

## Scenario 8: Cleaning Up After File Extractions

After extracting several archives that created many files:

```bash
# Scan the extraction directory
tidyfs scan ~/extracted -r

# Organize into a cleaner structure
tidyfs organize ~/extracted -b type
```

## Scenario 9: Setting Up Ignores for Development Directories

When scanning development directories, you often want to ignore certain folders:

```bash
# Add common development directories to ignore
tidyfs config --add-ignore "node_modules"
tidyfs config --add-ignore "target"
tidyfs config --add-ignore ".git"
tidyfs config --add-ignore "dist"
tidyfs config --add-ignore "__pycache__"

# Now scanning will skip these directories
tidyfs scan ~/projects -r
```

## Scenario 10: Batch Processing Multiple Directories

You can use shell scripting to process multiple directories:

```bash
# Process multiple directories
for dir in ~/Documents ~/Downloads ~/Desktop; do
    echo "Processing $dir"
    tidyfs scan "$dir" -r
    tidyfs organize "$dir" -b type
done
```

This script will organize Documents, Downloads, and Desktop by file type.
