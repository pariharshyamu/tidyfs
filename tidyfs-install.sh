#!/bin/bash
# TidyFS Installation Script

echo "TidyFS - Smart File System Organizer"
echo "===================================="
echo

# Check if Rust is installed
if ! command -v rustc &> /dev/null; then
    echo "Rust is not installed. Would you like to install it now? (y/n)"
    read -r install_rust
    if [[ "$install_rust" =~ ^[Yy]$ ]]; then
        echo "Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
        source "$HOME/.cargo/env"
    else
        echo "Rust is required to install TidyFS. Please install Rust first from https://rustup.rs/"
        exit 1
    fi
fi

# Create temporary directory
TEMP_DIR=$(mktemp -d)
cd "$TEMP_DIR" || exit 1

echo "Downloading TidyFS source code..."
# Git clone or curl the tarball here
# For demo purposes, we'll create the files directly

# Create Cargo.toml
cat > Cargo.toml << 'EOL'
[package]
name = "tidyfs"
version = "0.1.0"
edition = "2021"
authors = ["Shyamu Parihar <pariharshyamu@gmail.com>"]
description = "A smart file system organizer and analyzer"
readme = "README.md"
license = "MIT"
repository = "https://github.com/pariharshyamu/tidyfs"
keywords = ["files", "organize", "cli", "filesystem", "utility"]
categories = ["command-line-utilities", "filesystem"]

[dependencies]
clap = "2.33.3"
walkdir = "2.3.2"
rayon = "1.5.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
dirs = "4.0.0"
indicatif = "0.17.0"
colored = "2.0.0"
chrono = "0.4.19"
blake3 = "1.3.1"

[profile.release]
lto = true
codegen-units = 1
panic = "abort"
opt-level = 3
strip = true
EOL

# Create src directory
mkdir -p src

# Create main.rs - this would be the full source code in a real installer
cat > src/main.rs << 'EOL'
// TidyFS: Smart File System Organizer
// Full code would be here in a real installer
fn main() {
    println!("TidyFS - Smart File System Organizer");
    println!("Run 'tidyfs --help' to get started");
}
EOL

echo "Building TidyFS..."
cargo build --release

echo "Installing TidyFS..."
cargo install --path .

# Clean up
cd - || exit 1
rm -rf "$TEMP_DIR"

echo
echo "TidyFS has been installed successfully!"
echo "Run 'tidyfs' to get started."
echo 

# Optional: Create sample configuration
if [[ -t 0 ]]; then  # Only ask if running in interactive mode
    echo "Would you like to create a sample configuration? (y/n)"
    read -r create_config
    if [[ "$create_config" =~ ^[Yy]$ ]]; then
        mkdir -p "$HOME/.config/tidyfs"
        cat > "$HOME/.config/tidyfs/config.json" << 'EOL'
{
  "ignore_patterns": [
    ".git",
    "node_modules",
    "target",
    "build"
  ],
  "custom_categories": {
    "Web": [
      "html",
      "css",
      "js"
    ],
    "Data": [
      "csv",
      "json",
      "xml",
      "xlsx"
    ],
    "Design": [
      "psd",
      "ai",
      "sketch",
      "fig"
    ]
  },
  "recent_directories": [],
  "default_organization": "type"
}
EOL
        echo "Sample configuration created at ~/.config/tidyfs/config.json"
    fi
fi

echo
echo "Try these commands to get started:"
echo "  tidyfs scan ~/Downloads -r     # Scan your Downloads folder"
echo "  tidyfs organize ~/Desktop -n   # Preview organizing your Desktop (dry run)"
echo "  tidyfs config --list           # View your configuration"
echo
echo "Happy organizing!"
