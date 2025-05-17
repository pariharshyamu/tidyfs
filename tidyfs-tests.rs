#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::{Path, PathBuf};
    use tempfile::tempdir;

    // Import the necessary parts from our crate
    use super::*;

    // Helper function to create test files
    fn create_test_file(dir: &Path, name: &str, content: &str) -> PathBuf {
        let path = dir.join(name);
        let mut file = File::create(&path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        path
    }

    #[test]
    fn test_determine_category() {
        let config = TidyConfig::default();
        
        // Test document types
        assert!(matches!(
            determine_category(&Path::new("test.pdf"), &config),
            FileCategory::Document
        ));
        assert!(matches!(
            determine_category(&Path::new("test.docx"), &config),
            FileCategory::Document
        ));
        
        // Test image types
        assert!(matches!(
            determine_category(&Path::new("test.jpg"), &config),
            FileCategory::Image
        ));
        assert!(matches!(
            determine_category(&Path::new("test.png"), &config),
            FileCategory::Image
        ));
        
        // Test code types
        assert!(matches!(
            determine_category(&Path::new("test.rs"), &config),
            FileCategory::Code
        ));
        assert!(matches!(
            determine_category(&Path::new("test.py"), &config),
            FileCategory::Code
        ));
        
        // Test unknown type
        assert!(matches!(
            determine_category(&Path::new("test.xyz"), &config),
            FileCategory::Other(ext) if ext == "xyz"
        ));
        
        // Test no extension
        assert!(matches!(
            determine_category(&Path::new("test"), &config),
            FileCategory::Other(ext) if ext == "unknown"
        ));
    }

    #[test]
    fn test_determine_category_with_custom_config() {
        let mut config = TidyConfig::default();
        
        // Add a custom category
        let mut custom_categories = HashMap::new();
        custom_categories.insert(
            "CustomCategory".to_string(),
            vec!["abc".to_string(), "xyz".to_string()],
        );
        config.custom_categories = custom_categories;
        
        // Test standard category still works
        assert!(matches!(
            determine_category(&Path::new("test.jpg"), &config),
            FileCategory::Image
        ));
        
        // Test custom category
        assert!(matches!(
            determine_category(&Path::new("test.abc"), &config),
            FileCategory::Other(cat) if cat == "CustomCategory"
        ));
        assert!(matches!(
            determine_category(&Path::new("test.xyz"), &config),
            FileCategory::Other(cat) if cat == "CustomCategory"
        ));
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(500), "500 bytes");
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1024 * 1024), "1.00 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.00 GB");
        assert_eq!(format_size(1024 * 1024 + 512 * 1024), "1.50 MB");
    }

    #[test]
    fn test_calculate_hash() {
        let dir = tempdir().unwrap();
        
        // Create two identical files
        let file1 = create_test_file(&dir.path(), "file1.txt", "test content");
        let file2 = create_test_file(&dir.path(), "file2.txt", "test content");
        
        // Create a different file
        let file3 = create_test_file(&dir.path(), "file3.txt", "different content");
        
        // Calculate hashes
        let hash1 = calculate_hash(&file1).unwrap();
        let hash2 = calculate_hash(&file2).unwrap();
        let hash3 = calculate_hash(&file3).unwrap();
        
        // Identical files should have identical hashes
        assert_eq!(hash1, hash2);
        
        // Different files should have different hashes
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_find_duplicates() {
        // Create some file info objects with hashes
        let file1 = FileInfo {
            path: PathBuf::from("file1.txt"),
            size: 100,
            last_modified: 12345,
            category: FileCategory::Document,
            hash: Some("hash1".to_string()),
        };
        
        let file2 = FileInfo {
            path: PathBuf::from("file2.txt"),
            size: 100,
            last_modified: 12346,
            category: FileCategory::Document,
            hash: Some("hash1".to_string()),  // Same hash as file1
        };
        
        let file3 = FileInfo {
            path: PathBuf::from("file3.txt"),
            size: 200,
            last_modified: 12347,
            category: FileCategory::Document,
            hash: Some("hash2".to_string()),  // Different hash
        };
        
        let files = vec![file1, file2, file3];
        
        // Find duplicates
        let duplicates = find_duplicates(&files);
        
        // Should find one group of duplicates (files 1 and 2)
        assert_eq!(duplicates.len(), 1);
        assert_eq!(duplicates.get("hash1").unwrap().len(), 2);
        
        // The group should contain the paths of files 1 and 2
        let duplicate_paths: Vec<String> = duplicates
            .get("hash1")
            .unwrap()
            .iter()
            .map(|f| f.path.to_string_lossy().to_string())
            .collect();
        
        assert!(duplicate_paths.contains(&"file1.txt".to_string()));
        assert!(duplicate_paths.contains(&"file2.txt".to_string()));
    }

    #[test]
    fn test_scan_directory() {
        let dir = tempdir().unwrap();
        
        // Create some test files
        create_test_file(&dir.path(), "doc.pdf", "pdf content");
        create_test_file(&dir.path(), "image.jpg", "jpg content");
        create_test_file(&dir.path(), "unknown.xyz", "unknown content");
        
        // Create a subdirectory with a file
        let subdir = dir.path().join("subdir");
        fs::create_dir(&subdir).unwrap();
        create_test_file(&subdir, "subfile.txt", "subfile content");
        
        // Scan without recursion
        let config = TidyConfig::default();
        let files = scan_directory(&dir.path(), &config, false, false).unwrap();
        
        // Should find 3 files (not including the file in the subdirectory)
        assert_eq!(files.len(), 3);
        
        // Scan with recursion
        let files_recursive = scan_directory(&dir.path(), &config, false, true).unwrap();
        
        // Should find 4 files (including the file in the subdirectory)
        assert_eq!(files_recursive.len(), 4);
        
        // Verify correct categorization
        let categories: Vec<String> = files
            .iter()
            .map(|f| match &f.category {
                FileCategory::Document => "Document".to_string(),
                FileCategory::Image => "Image".to_string(),
                FileCategory::Other(ext) => format!("Other({})", ext),
                _ => "Other".to_string(),
            })
            .collect();
        
        assert!(categories.contains(&"Document".to_string()));
        assert!(categories.contains(&"Image".to_string()));
        assert!(categories.contains(&"Other(xyz)".to_string()));
    }

    #[test]
    fn test_config_save_load() {
        let dir = tempdir().unwrap();
        let config_dir = dir.path().join("config");
        fs::create_dir_all(&config_dir).unwrap();
        
        // Create a custom config
        let mut config = TidyConfig::default();
        config.ignore_patterns.push("test_pattern".to_string());
        
        let mut custom_categories = HashMap::new();
        custom_categories.insert(
            "TestCategory".to_string(),
            vec!["test".to_string(), "example".to_string()],
        );
        config.custom_categories = custom_categories;
        
        // Mock the dirs::config_dir function to use our temp directory
        // In a real implementation, we would use dependency injection or a configurable
        // config directory path instead of directly mocking
        
        // For test purposes, we'll just use our own save/load functions
        
        let config_path = config_dir.join("config.json");
        let config_json = serde_json::to_string_pretty(&config).unwrap();
        let mut file = File::create(&config_path).unwrap();
        file.write_all(config_json.as_bytes()).unwrap();
        
        // Load the config back
        let mut file = File::open(&config_path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let loaded_config: TidyConfig = serde_json::from_str(&contents).unwrap();
        
        // Verify the loaded config matches what we saved
        assert_eq!(loaded_config.ignore_patterns.len(), config.ignore_patterns.len());
        assert!(loaded_config.ignore_patterns.contains(&"test_pattern".to_string()));
        
        assert_eq!(
            loaded_config.custom_categories.len(),
            config.custom_categories.len()
        );
        assert!(loaded_config.custom_categories.contains_key("TestCategory"));
        
        let test_category = loaded_config.custom_categories.get("TestCategory").unwrap();
        assert_eq!(test_category.len(), 2);
        assert!(test_category.contains(&"test".to_string()));
        assert!(test_category.contains(&"example".to_string()));
    }
}
