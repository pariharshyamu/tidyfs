# TidyFS: Smart File System Organizer

TidyFS is a powerful command-line tool written in Rust that helps you organize, analyze, and manage your files efficiently.

## Features

- **Intelligent Scanning**: Analyze directories to get insights about file types, sizes, and usage patterns
- **Smart Organization**: Automatically organize files by type, date, or extension
- **Duplicate Detection**: Find and manage duplicate files to reclaim disk space
- **Customizable Categories**: Define custom file categories based on extensions
- **Detailed Reports**: Generate storage usage reports with visual breakdowns
- **Fast & Efficient**: Built with Rust for high performance, even with large directories
- **Configuration System**: Save and manage your preferences between sessions

## Installation

### From Source

1. Make sure you have Rust and Cargo installed. If not, install them from [rustup.rs](https://rustup.rs/).
2. Clone the repository:
   ```
   git clone https://github.com/pariharshyamu/tidyfs.git
   cd tidyfs
   ```
3. Build and install:
   ```
   cargo build --release
   cargo install --path .
   ```

### From Cargo

```
cargo install tidyfs
```

## Usage

TidyFS has several subcommands for different operations:

### Scanning Directories

```
tidyfs scan [DIR] [OPTIONS]
```

Options:
- `-r, --recursive`: Scan subdirectories recursively
- `-d, --duplicates`: Find duplicate files

Example:
```
tidyfs scan ~/Documents -r -d
```

This will scan your Documents directory recursively and find any duplicate files.

### Organizing Files

```
tidyfs organize [DIR] [OPTIONS]
```

Options:
- `-t, --target [DIR]`: Target directory for organized files
- `-b, --by [METHOD]`: Organization method (type, date, ext)
- `-n, --dry-run`: Show what would be done without making changes
- `-r, --recursive`: Process subdirectories recursively

Examples:
```
# Organize current directory by file type
tidyfs organize .

# Organize Downloads by date into a separate directory
tidyfs organize ~/Downloads -t ~/Organized -b date

# Preview organization without making changes
tidyfs organize ~/Desktop -n
```

### Configuration

```
tidyfs config [OPTIONS]
```

Options:
- `--list`: List current configuration
- `--add-ignore [PATTERN]`: Add pattern to ignore list
- `--remove-ignore [PATTERN]`: Remove pattern from ignore list
- `--add-category [CATEGORY:EXT1,EXT2]`: Add custom category
- `--set-default-org [METHOD]`: Set default organization method

Examples:
```
# Add a custom category for design files
tidyfs config --add-category "Design:psd,ai,sketch,fig"

# Ignore node_modules directories
tidyfs config --add-ignore "node_modules"
```

## File Categories

TidyFS automatically categorizes files into:

- **Documents**: pdf, doc, docx, txt, etc.
- **Images**: jpg, png, gif, svg, etc.
- **Videos**: mp4, avi, mov, mkv, etc.
- **Audio**: mp3, wav, ogg, flac, etc.
- **Archives**: zip, rar, 7z, tar.gz, etc.
- **Code**: rs, py, js, html, java, etc.
- **Executables**: exe, msi, app, dmg, etc.
- **Other**: Files with other extensions

You can add custom categories through the configuration system.

## Storage Reports

When scanning directories, TidyFS generates detailed reports showing:

- Total storage usage and file count
- Breakdown by category with percentages
- Largest files in the scanned directory
- Duplicate files (when using the `-d` option)

## Example Workflow

```bash
# First, scan to see what's in your Downloads folder
tidyfs scan ~/Downloads -r

# Look for duplicate files to free up space
tidyfs scan ~/Downloads -r -d

# Do a dry-run to preview organization
tidyfs organize ~/Downloads -n

# Organize files by type
tidyfs organize ~/Downloads -r

# Add custom categories for specific file types
tidyfs config --add-category "Projects:ipynb,rproj,rmd"

# Analyze again with your custom categories
tidyfs scan ~/Downloads
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
