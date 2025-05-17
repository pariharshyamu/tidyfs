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
   git clone https://github.com/yourusername/tidyfs.git
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

## License

MIT
