use clap::{App, Arg, SubCommand};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use walkdir::WalkDir;
use chrono::{DateTime, Local, Utc};
use std::sync::{Arc, Mutex};
use blake3::Hasher;

// File categories for organization
#[derive(Debug, Clone, Serialize, Deserialize)]
enum FileCategory {
    Document,
    Image,
    Video,
    Audio,
    Archive,
    Code,
    Executable,
    Other(String),
}

// File information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct FileInfo {
    path: PathBuf,
    size: u64,
    last_modified: u64,
    category: FileCategory,
    hash: Option<String>, // For duplicate detection
}

// Config structure for persistent settings
#[derive(Debug, Serialize, Deserialize)]
struct TidyConfig {
    ignore_patterns: Vec<String>,
    custom_categories: HashMap<String, Vec<String>>,
    recent_directories: Vec<PathBuf>,
    default_organization: String,
}

impl Default for TidyConfig {
    fn default() -> Self {
        TidyConfig {
            ignore_patterns: vec![".git".to_string(), "node_modules".to_string()],
            custom_categories: HashMap::new(),
            recent_directories: Vec::new(),
            default_organization: "type".to_string(),
        }
    }
}

// Function to determine file category based on extension
fn determine_category(path: &Path, config: &TidyConfig) -> FileCategory {
    if let Some(extension) = path.extension() {
        let ext = extension.to_string_lossy().to_lowercase();
        
        // Check custom categories first
        for (category, extensions) in &config.custom_categories {
            if extensions.iter().any(|e| e.to_lowercase() == ext) {
                return FileCategory::Other(category.clone());
            }
        }
        
        // Standard categories
        match ext.as_str() {
            "pdf" | "doc" | "docx" | "txt" | "rtf" | "odt" | "md" | "xls" | "xlsx" | "ppt" | "pptx" => {
                FileCategory::Document
            }
            "jpg" | "jpeg" | "png" | "gif" | "bmp" | "tiff" | "svg" | "webp" => FileCategory::Image,
            "mp4" | "avi" | "mov" | "wmv" | "flv" | "mkv" | "webm" => FileCategory::Video,
            "mp3" | "wav" | "ogg" | "flac" | "aac" | "m4a" => FileCategory::Audio,
            "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" => FileCategory::Archive,
            "rs" | "py" | "js" | "html" | "css" | "java" | "c" | "cpp" | "h" | "go" | "rb" | "php" | "sh" => {
                FileCategory::Code
            }
            "exe" | "msi" | "app" | "dmg" | "deb" | "rpm" => FileCategory::Executable,
            _ => FileCategory::Other(ext.to_string()),
        }
    } else {
        FileCategory::Other("unknown".to_string())
    }
}

// Calculate file hash for duplicate detection
fn calculate_hash(path: &Path) -> Result<String, Box<dyn Error>> {
    let mut file = File::open(path)?;
    let mut hasher = Hasher::new();
    let mut buffer = [0; 8192];
    
    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    
    Ok(hasher.finalize().to_hex().to_string())
}

// Get file info including size, modification time, and category
fn get_file_info(path: &Path, config: &TidyConfig, calculate_hashes: bool) -> Result<FileInfo, Box<dyn Error>> {
    let metadata = fs::metadata(path)?;
    let size = metadata.len();
    
    let last_modified = metadata
        .modified()?
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    
    let category = determine_category(path, config);
    
    let hash = if calculate_hashes {
        Some(calculate_hash(path)?)
    } else {
        None
    };
    
    Ok(FileInfo {
        path: path.to_path_buf(),
        size,
        last_modified,
        category,
        hash,
    })
}

// Scan directory and collect file information
fn scan_directory(
    dir: &Path, 
    config: &TidyConfig, 
    calculate_hashes: bool,
    recursive: bool
) -> Result<Vec<FileInfo>, Box<dyn Error>> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈"),
    );
    pb.set_message("Scanning directory...");
    
    let files_info = Arc::new(Mutex::new(Vec::new()));
    let error_count = Arc::new(Mutex::new(0));
    let file_count = Arc::new(Mutex::new(0));
    
    let walker = if recursive {
        WalkDir::new(dir)
    } else {
        WalkDir::new(dir).max_depth(1)
    };
    
    let entries: Vec<_> = walker
        .into_iter()
        .filter_entry(|e| {
            let path = e.path();
            !config.ignore_patterns.iter().any(|pattern| {
                path.to_string_lossy().contains(pattern)
            })
        })
        .filter_map(|e| e.ok())
        .collect();
    
    pb.set_length(entries.len() as u64);
    pb.set_message("Processing files...");
    
    entries.into_par_iter().for_each(|entry| {
        let path = entry.path();
        if path.is_file() {
            match get_file_info(path, config, calculate_hashes) {
                Ok(info) => {
                    let mut file_infos = files_info.lock().unwrap();
                    file_infos.push(info);
                    
                    let mut count = file_count.lock().unwrap();
                    *count += 1;
                    if *count % 100 == 0 {
                        pb.set_message(format!("Processed {} files...", *count));
                    }
                }
                Err(_) => {
                    let mut errors = error_count.lock().unwrap();
                    *errors += 1;
                }
            }
        }
    });
    
    let error_count = *error_count.lock().unwrap();
    let file_count = *file_count.lock().unwrap();
    
    pb.finish_with_message(format!(
        "Scan complete. Processed {} files with {} errors",
        file_count, error_count
    ));
    
    Ok(Arc::try_unwrap(files_info).unwrap().into_inner()?)
}

// Find duplicate files based on hash
fn find_duplicates(files: &[FileInfo]) -> HashMap<String, Vec<&FileInfo>> {
    let mut duplicates: HashMap<String, Vec<&FileInfo>> = HashMap::new();
    
    for file in files {
        if let Some(hash) = &file.hash {
            duplicates.entry(hash.clone()).or_default().push(file);
        }
    }
    
    // Keep only entries with more than one file (actual duplicates)
    duplicates.retain(|_, files| files.len() > 1);
    
    duplicates
}

// Format size in human-readable form
fn format_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    
    if size >= GB {
        format!("{:.2} GB", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.2} MB", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.2} KB", size as f64 / KB as f64)
    } else {
        format!("{} bytes", size)
    }
}

// Format timestamp as readable date
fn format_timestamp(timestamp: u64) -> String {
    let datetime = DateTime::<Utc>::from_timestamp(timestamp as i64, 0).unwrap();
    let local_time = datetime.with_timezone(&Local);
    local_time.format("%Y-%m-%d %H:%M:%S").to_string()
}

// Organize files by moving them to category folders
fn organize_files(
    files: &[FileInfo],
    target_dir: &Path,
    organization_type: &str,
    dry_run: bool,
) -> Result<(), Box<dyn Error>> {
    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("##-"),
    );
    
    let move_count = Arc::new(Mutex::new(0));
    let error_count = Arc::new(Mutex::new(0));
    
    for file in files {
        let target_subdir = match organization_type {
            "type" => {
                match &file.category {
                    FileCategory::Document => "Documents",
                    FileCategory::Image => "Images",
                    FileCategory::Video => "Videos",
                    FileCategory::Audio => "Audio",
                    FileCategory::Archive => "Archives",
                    FileCategory::Code => "Code",
                    FileCategory::Executable => "Executables",
                    FileCategory::Other(ext) => {
                        if ext == "unknown" {
                            "Other"
                        } else {
                            "Miscellaneous"
                        }
                    }
                }
            }
            "date" => {
                let datetime = DateTime::<Utc>::from_timestamp(file.last_modified as i64, 0).unwrap();
                let local_time = datetime.with_timezone(&Local);
                local_time.format("%Y-%m").to_string().as_str().into()
            }
            "ext" => {
                if let Some(extension) = file.path.extension() {
                    extension.to_string_lossy().to_string().as_str()
                } else {
                    "no_extension"
                }
            }
            _ => "Unsorted",
        };
        
        let target_path = target_dir.join(target_subdir);
        
        if !dry_run {
            fs::create_dir_all(&target_path)?;
            
            let file_name = file.path.file_name().unwrap();
            let destination = target_path.join(file_name);
            
            if destination.exists() {
                // Handle name collision by adding a timestamp
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                
                let file_stem = file.path.file_stem().unwrap().to_string_lossy();
                let extension = file.path
                    .extension()
                    .map(|ext| format!(".{}", ext.to_string_lossy()))
                    .unwrap_or_default();
                
                let new_name = format!("{}_{}{}", file_stem, now, extension);
                let destination = target_path.join(new_name);
                
                match fs::rename(&file.path, &destination) {
                    Ok(_) => {
                        let mut count = move_count.lock().unwrap();
                        *count += 1;
                    }
                    Err(_) => {
                        let mut errors = error_count.lock().unwrap();
                        *errors += 1;
                    }
                }
            } else {
                match fs::rename(&file.path, &destination) {
                    Ok(_) => {
                        let mut count = move_count.lock().unwrap();
                        *count += 1;
                    }
                    Err(_) => {
                        let mut errors = error_count.lock().unwrap();
                        *errors += 1;
                    }
                }
            }
        }
        
        pb.inc(1);
        pb.set_message(format!("Moving to {}", target_subdir));
    }
    
    let move_count = *move_count.lock().unwrap();
    let error_count = *error_count.lock().unwrap();
    
    if dry_run {
        pb.finish_with_message("Dry run complete. No files were moved.");
    } else {
        pb.finish_with_message(format!(
            "Organization complete. Moved {} files with {} errors",
            move_count, error_count
        ));
    }
    
    Ok(())
}

// Load config from file or create default
fn load_config() -> Result<TidyConfig, Box<dyn Error>> {
    let config_dir = dirs::config_dir()
        .ok_or("Could not determine config directory")?
        .join("tidyfs");
    
    fs::create_dir_all(&config_dir)?;
    
    let config_path = config_dir.join("config.json");
    
    if config_path.exists() {
        let mut file = File::open(config_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        
        let config: TidyConfig = serde_json::from_str(&contents)?;
        Ok(config)
    } else {
        let config = TidyConfig::default();
        save_config(&config)?;
        Ok(config)
    }
}

// Save config to file
fn save_config(config: &TidyConfig) -> Result<(), Box<dyn Error>> {
    let config_dir = dirs::config_dir()
        .ok_or("Could not determine config directory")?
        .join("tidyfs");
    
    fs::create_dir_all(&config_dir)?;
    
    let config_path = config_dir.join("config.json");
    let config_json = serde_json::to_string_pretty(config)?;
    
    let mut file = File::create(config_path)?;
    file.write_all(config_json.as_bytes())?;
    
    Ok(())
}

// Update config with a new recently used directory
fn update_recent_directories(config: &mut TidyConfig, dir: &Path) -> Result<(), Box<dyn Error>> {
    let dir_path = dir.to_path_buf();
    
    // Remove if already exists
    config.recent_directories.retain(|path| path != &dir_path);
    
    // Add to front
    config.recent_directories.insert(0, dir_path);
    
    // Keep only the 10 most recent
    if config.recent_directories.len() > 10 {
        config.recent_directories.truncate(10);
    }
    
    save_config(config)
}

// Display storage usage report
fn display_storage_report(files: &[FileInfo]) {
    let total_size: u64 = files.iter().map(|f| f.size).sum();
    let file_count = files.len();
    
    // Group by category
    let mut category_sizes: HashMap<String, u64> = HashMap::new();
    let mut category_counts: HashMap<String, usize> = HashMap::new();
    
    for file in files {
        let category = match &file.category {
            FileCategory::Document => "Documents".to_string(),
            FileCategory::Image => "Images".to_string(),
            FileCategory::Video => "Videos".to_string(),
            FileCategory::Audio => "Audio".to_string(),
            FileCategory::Archive => "Archives".to_string(),
            FileCategory::Code => "Code".to_string(),
            FileCategory::Executable => "Executables".to_string(),
            FileCategory::Other(ext) => {
                if ext == "unknown" {
                    "Unknown".to_string()
                } else {
                    format!("Other (.{})", ext)
                }
            }
        };
        
        *category_sizes.entry(category.clone()).or_insert(0) += file.size;
        *category_counts.entry(category).or_insert(0) += 1;
    }
    
    // Sort categories by size descending
    let mut categories: Vec<(String, u64, usize)> = category_sizes
        .iter()
        .map(|(k, v)| (k.clone(), *v, *category_counts.get(k).unwrap_or(&0)))
        .collect();
    
    categories.sort_by(|a, b| b.1.cmp(&a.1));
    
    println!("\n{}", "Storage Usage Report".bold().underline());
    println!(
        "Total: {} files, {}",
        file_count,
        format_size(total_size).bold()
    );
    
    println!("\n{:<20} {:<15} {:<10} {:<10}", 
             "Category".bold(), 
             "Size".bold(), 
             "Files".bold(), 
             "% of Total".bold());
    
    println!("{}", "-".repeat(55));
    
    for (category, size, count) in categories {
        let percentage = (size as f64 / total_size as f64) * 100.0;
        
        println!(
            "{:<20} {:<15} {:<10} {:.1}%",
            category,
            format_size(size),
            count,
            percentage
        );
    }
    
    // Find largest files
    let mut largest_files = files.to_vec();
    largest_files.sort_by(|a, b| b.size.cmp(&a.size));
    largest_files.truncate(5);
    
    println!("\n{}", "Largest Files:".bold().underline());
    for file in largest_files {
        println!(
            "{} ({})",
            file.path.display().to_string().cyan(),
            format_size(file.size).yellow()
        );
    }
}

// Main function with CLI handling
fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("TidyFS")
        .version("1.0")
        .author("Claude")
        .about("Smart file system organizer and analyzer")
        .subcommand(
            SubCommand::with_name("scan")
                .about("Scan directory and show statistics")
                .arg(
                    Arg::with_name("dir")
                        .help("Directory to scan")
                        .default_value(".")
                        .index(1),
                )
                .arg(
                    Arg::with_name("recursive")
                        .short("r")
                        .long("recursive")
                        .help("Scan subdirectories recursively"),
                )
                .arg(
                    Arg::with_name("duplicates")
                        .short("d")
                        .long("duplicates")
                        .help("Find duplicate files"),
                ),
        )
        .subcommand(
            SubCommand::with_name("organize")
                .about("Organize files into folders")
                .arg(
                    Arg::with_name("dir")
                        .help("Directory to organize")
                        .default_value(".")
                        .index(1),
                )
                .arg(
                    Arg::with_name("target")
                        .help("Target directory for organized files")
                        .short("t")
                        .long("target")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("by")
                        .help("Organization method (type, date, ext)")
                        .short("b")
                        .long("by")
                        .takes_value(true)
                        .default_value("type"),
                )
                .arg(
                    Arg::with_name("dry-run")
                        .help("Show what would be done without making changes")
                        .short("n")
                        .long("dry-run"),
                )
                .arg(
                    Arg::with_name("recursive")
                        .short("r")
                        .long("recursive")
                        .help("Process subdirectories recursively"),
                ),
        )
        .subcommand(
            SubCommand::with_name("config")
                .about("Configure TidyFS settings")
                .arg(
                    Arg::with_name("add-ignore")
                        .long("add-ignore")
                        .help("Add pattern to ignore list")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("remove-ignore")
                        .long("remove-ignore")
                        .help("Remove pattern from ignore list")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("list")
                        .long("list")
                        .help("List current configuration"),
                )
                .arg(
                    Arg::with_name("add-category")
                        .long("add-category")
                        .help("Add custom category (format: 'category:ext1,ext2')")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("set-default-org")
                        .long("set-default-org")
                        .help("Set default organization method (type, date, ext)")
                        .takes_value(true),
                ),
        )
        .get_matches();
    
    let mut config = load_config()?;
    
    match matches.subcommand() {
        ("scan", Some(scan_matches)) => {
            let dir_str = scan_matches.value_of("dir").unwrap();
            let dir = Path::new(dir_str);
            
            update_recent_directories(&mut config, dir)?;
            
            let recursive = scan_matches.is_present("recursive");
            let find_duplicates = scan_matches.is_present("duplicates");
            
            println!(
                "{}",
                format!("Scanning directory: {}", dir.display()).bold().green()
            );
            
            let files = scan_directory(dir, &config, find_duplicates, recursive)?;
            
            if files.is_empty() {
                println!("No files found in the specified directory.");
                return Ok(());
            }
            
            display_storage_report(&files);
            
            if find_duplicates {
                let duplicates = find_duplicates(&files);
                
                if duplicates.is_empty() {
                    println!("\n{}", "No duplicate files found.".bold());
                } else {
                    let total_groups = duplicates.len();
                    let total_duplicates: usize = duplicates.values().map(|files| files.len() - 1).sum();
                    let wasted_space: u64 = duplicates
                        .values()
                        .map(|files| files[0].size * (files.len() as u64 - 1))
                        .sum();
                    
                    println!(
                        "\n{} ({} duplicate files in {} groups, wasting {})",
                        "Duplicate Files Found".bold().yellow(),
                        total_duplicates,
                        total_groups,
                        format_size(wasted_space).bold()
                    );
                    
                    // Sort duplicates by wasted space (largest first)
                    let mut sorted_duplicates: Vec<_> = duplicates.iter().collect();
                    sorted_duplicates.sort_by(|a, b| {
                        let a_size = a.1[0].size * (a.1.len() as u64 - 1);
                        let b_size = b.1[0].size * (b.1.len() as u64 - 1);
                        b_size.cmp(&a_size)
                    });
                    
                    // Show top 5 duplicate groups
                    for (i, (_, files)) in sorted_duplicates.iter().take(5).enumerate() {
                        let wasted = files[0].size * (files.len() as u64 - 1);
                        println!(
                            "\nGroup {} - {} duplicates, wasting {}:",
                            i + 1,
                            files.len() - 1,
                            format_size(wasted).yellow()
                        );
                        
                        for file in *files {
                            println!("  {}", file.path.display());
                        }
                    }
                    
                    if sorted_duplicates.len() > 5 {
                        println!("\n... and {} more duplicate groups", sorted_duplicates.len() - 5);
                    }
                }
            }
        }
        ("organize", Some(org_matches)) => {
            let dir_str = org_matches.value_of("dir").unwrap();
            let dir = Path::new(dir_str);
            
            let target_dir = if let Some(target) = org_matches.value_of("target") {
                Path::new(target).to_path_buf()
            } else {
                dir.to_path_buf()
            };
            
            update_recent_directories(&mut config, dir)?;
            
            let organization_type = org_matches.value_of("by").unwrap();
            let dry_run = org_matches.is_present("dry-run");
            let recursive = org_matches.is_present("recursive");
            
            println!(
                "{}",
                format!(
                    "Organizing files in {} by {}{}",
                    dir.display(),
                    organization_type,
                    if dry_run { " (DRY RUN)" } else { "" }
                )
                .bold()
                .green()
            );
            
            let files = scan_directory(dir, &config, false, recursive)?;
            
            if files.is_empty() {
                println!("No files found in the specified directory.");
                return Ok(());
            }
            
            organize_files(&files, &target_dir, organization_type, dry_run)?;
        }
        ("config", Some(config_matches)) => {
            if config_matches.is_present("list") {
                println!("{}", "Current Configuration:".bold().underline());
                println!("Ignored patterns:");
                for pattern in &config.ignore_patterns {
                    println!("  - {}", pattern);
                }
                
                println!("\nCustom categories:");
                for (category, extensions) in &config.custom_categories {
                    println!("  - {}: {}", category, extensions.join(", "));
                }
                
                println!("\nDefault organization method: {}", config.default_organization);
                
                println!("\nRecent directories:");
                for dir in &config.recent_directories {
                    println!("  - {}", dir.display());
                }
            }
            
            if let Some(pattern) = config_matches.value_of("add-ignore") {
                config.ignore_patterns.push(pattern.to_string());
                save_config(&config)?;
                println!("Added '{}' to ignore patterns", pattern);
            }
            
            if let Some(pattern) = config_matches.value_of("remove-ignore") {
                let before_len = config.ignore_patterns.len();
                config.ignore_patterns.retain(|p| p != pattern);
                
                if config.ignore_patterns.len() < before_len {
                    save_config(&config)?;
                    println!("Removed '{}' from ignore patterns", pattern);
                } else {
                    println!("Pattern '{}' not found in ignore list", pattern);
                }
            }
            
            if let Some(category_def) = config_matches.value_of("add-category") {
                if let Some(colon_pos) = category_def.find(':') {
                    let category = &category_def[0..colon_pos];
                    let extensions: Vec<String> = category_def[colon_pos + 1..]
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .collect();
                    
                    if !extensions.is_empty() {
                        config
                            .custom_categories
                            .insert(category.to_string(), extensions.clone());
                        save_config(&config)?;
                        println!(
                            "Added custom category '{}' with extensions: {}",
                            category,
                            extensions.join(", ")
                        );
                    } else {
                        println!("No extensions specified for category");
                    }
                } else {
                    println!("Invalid category format. Use 'category:ext1,ext2'");
                }
            }
            
            if let Some(org_method) = config_matches.value_of("set-default-org") {
                match org_method {
                    "type" | "date" | "ext" => {
                        config.default_organization = org_method.to_string();
                        save_config(&config)?;
                        println!("Default organization method set to '{}'", org_method);
                    }
                    _ => {
                        println!("Invalid organization method. Use 'type', 'date', or 'ext'");
                    }
                }
            }
        }
        _ => {
            println!("{}", "TidyFS - Smart File System Organizer".bold().green());
            println!("Run with a subcommand to begin:");
            println!("  {} - Scan directory and show statistics", "scan".cyan());
            println!("  {} - Organize files into folders", "organize".cyan());
            println!("  {} - Configure TidyFS settings", "config".cyan());
            println!("\nUse --help with any subcommand for more information.");
        }
    }
    
    Ok(())
}
