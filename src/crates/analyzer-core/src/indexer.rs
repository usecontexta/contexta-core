// Indexer module - File discovery and indexing logic
// Implements recursive directory walk, language detection, and progress reporting

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use rayon::prelude::*;

use crate::{detect_language, FileMetadata};

/// Callback for progress reporting during indexing
pub type ProgressCallback = Box<dyn Fn(usize, usize) + Send + Sync>;

/// Indexer configuration
#[derive(Debug, Clone)]
pub struct IndexerConfig {
    /// Root directory to index
    pub root_dir: PathBuf,

    /// File extensions to index (empty = all supported languages)
    pub extensions: Vec<String>,

    /// Directories to exclude
    pub exclude_dirs: Vec<String>,

    /// Maximum file size in bytes (skip larger files)
    pub max_file_size: u64,
}

impl Default for IndexerConfig {
    fn default() -> Self {
        Self {
            root_dir: PathBuf::from("."),
            extensions: vec![],
            exclude_dirs: vec![
                ".git".to_string(),
                "node_modules".to_string(),
                "target".to_string(),
                ".venv".to_string(),
                "venv".to_string(),
                "__pycache__".to_string(),
                "dist".to_string(),
                "build".to_string(),
                ".next".to_string(),
            ],
            max_file_size: 10 * 1024 * 1024, // 10 MB
        }
    }
}

/// Discover all indexable files in a directory
pub fn discover_files(config: &IndexerConfig) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    discover_files_recursive(&config.root_dir, config, &mut files)?;
    Ok(files)
}

fn discover_files_recursive(
    dir: &Path,
    config: &IndexerConfig,
    files: &mut Vec<PathBuf>,
) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }

    // Check if directory should be excluded
    if let Some(dir_name) = dir.file_name().and_then(|n| n.to_str()) {
        if config.exclude_dirs.contains(&dir_name.to_string()) {
            return Ok(());
        }
    }

    for entry in fs::read_dir(dir).context("Failed to read directory")? {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();

        if path.is_dir() {
            // Recursively index subdirectories
            discover_files_recursive(&path, config, files)?;
        } else if path.is_file() {
            // Check if file should be indexed
            if should_index_file(&path, config)? {
                files.push(path);
            }
        }
    }

    Ok(())
}

fn should_index_file(path: &Path, config: &IndexerConfig) -> Result<bool> {
    // Check file size
    let metadata = fs::metadata(path).context("Failed to read file metadata")?;
    if metadata.len() > config.max_file_size {
        return Ok(false);
    }

    // Check language support
    let path_str = path.to_str().unwrap_or("");
    if detect_language(path_str).is_none() {
        return Ok(false);
    }

    // If extensions filter is specified, check it
    if !config.extensions.is_empty() {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            return Ok(config.extensions.contains(&ext.to_string()));
        }
        return Ok(false);
    }

    Ok(true)
}

/// Create FileMetadata from a file path
pub fn create_file_metadata(path: &Path) -> Result<FileMetadata> {
    let metadata = fs::metadata(path).context("Failed to read file metadata")?;
    let path_str = path.to_str().context("Invalid UTF-8 in file path")?;
    let language = detect_language(path_str)
        .context("Unsupported file language")?
        .to_string();

    Ok(FileMetadata {
        id: None,
        path: path_str.to_string(),
        language,
        size: metadata.len(),
        last_indexed: None,
        parse_errors: 0,
    })
}

/// Index files with progress reporting (sequential)
pub fn index_files_with_progress(
    files: &[PathBuf],
    callback: ProgressCallback,
) -> Result<Vec<FileMetadata>> {
    let total = files.len();
    let mut file_metadata = Vec::with_capacity(total);

    for (idx, path) in files.iter().enumerate() {
        callback(idx + 1, total);

        match create_file_metadata(path) {
            Ok(metadata) => file_metadata.push(metadata),
            Err(e) => {
                eprintln!("Failed to index {}: {}", path.display(), e);
                continue;
            }
        }
    }

    Ok(file_metadata)
}

/// Index files with progress reporting (parallel with rayon)
pub fn index_files_with_progress_parallel(
    files: &[PathBuf],
    callback: ProgressCallback,
) -> Result<Vec<FileMetadata>> {
    let total = files.len();
    let counter = Arc::new(AtomicUsize::new(0));
    let callback = Arc::new(callback);

    // Process files in parallel using rayon
    let file_metadata: Vec<FileMetadata> = files
        .par_iter()
        .filter_map(|path| {
            // Update progress counter
            let current = counter.fetch_add(1, Ordering::Relaxed) + 1;
            callback(current, total);

            match create_file_metadata(path) {
                Ok(metadata) => Some(metadata),
                Err(e) => {
                    eprintln!("Failed to index {}: {}", path.display(), e);
                    None
                }
            }
        })
        .collect();

    Ok(file_metadata)
}

/// Partial re-indexing: only index changed files (sequential)
pub fn reindex_files(
    changed_files: &[PathBuf],
    callback: Option<ProgressCallback>,
) -> Result<Vec<FileMetadata>> {
    let total = changed_files.len();
    let mut file_metadata = Vec::with_capacity(total);

    for (idx, path) in changed_files.iter().enumerate() {
        if let Some(ref cb) = callback {
            cb(idx + 1, total);
        }

        match create_file_metadata(path) {
            Ok(metadata) => file_metadata.push(metadata),
            Err(e) => {
                eprintln!("Failed to re-index {}: {}", path.display(), e);
                continue;
            }
        }
    }

    Ok(file_metadata)
}

/// Partial re-indexing: only index changed files (parallel with rayon)
pub fn reindex_files_parallel(
    changed_files: &[PathBuf],
    callback: Option<ProgressCallback>,
) -> Result<Vec<FileMetadata>> {
    let total = changed_files.len();

    if let Some(callback) = callback {
        let counter = Arc::new(AtomicUsize::new(0));
        let callback = Arc::new(callback);

        // Process files in parallel using rayon
        let file_metadata: Vec<FileMetadata> = changed_files
            .par_iter()
            .filter_map(|path| {
                // Update progress counter
                let current = counter.fetch_add(1, Ordering::Relaxed) + 1;
                callback(current, total);

                match create_file_metadata(path) {
                    Ok(metadata) => Some(metadata),
                    Err(e) => {
                        eprintln!("Failed to re-index {}: {}", path.display(), e);
                        None
                    }
                }
            })
            .collect();

        Ok(file_metadata)
    } else {
        // No callback, just process in parallel
        let file_metadata: Vec<FileMetadata> = changed_files
            .par_iter()
            .filter_map(|path| {
                match create_file_metadata(path) {
                    Ok(metadata) => Some(metadata),
                    Err(e) => {
                        eprintln!("Failed to re-index {}: {}", path.display(), e);
                        None
                    }
                }
            })
            .collect();

        Ok(file_metadata)
    }
}

/// Handle a single file change event for incremental indexing
pub fn handle_file_change(path: &Path, config: &IndexerConfig) -> Result<Option<FileMetadata>> {
    // Check if file should be indexed
    if !should_index_file(path, config)? {
        return Ok(None);
    }

    // Create metadata for the changed file
    create_file_metadata(path).map(Some)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::TempDir;

    #[test]
    fn test_discover_files() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create test files
        File::create(temp_path.join("test.py")).unwrap();
        File::create(temp_path.join("test.ts")).unwrap();
        File::create(temp_path.join("test.rs")).unwrap();
        File::create(temp_path.join("test.txt")).unwrap();

        let config = IndexerConfig {
            root_dir: temp_path.to_path_buf(),
            ..Default::default()
        };

        let files = discover_files(&config).unwrap();

        // Should find 3 files (py, ts, rs) but not txt
        assert_eq!(files.len(), 3);
    }

    #[test]
    fn test_exclude_directories() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create test structure
        fs::create_dir(temp_path.join("node_modules")).unwrap();
        File::create(temp_path.join("node_modules/test.js")).unwrap();
        File::create(temp_path.join("main.js")).unwrap();

        let config = IndexerConfig {
            root_dir: temp_path.to_path_buf(),
            ..Default::default()
        };

        let files = discover_files(&config).unwrap();

        // Should only find main.js, not node_modules/test.js
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("main.js"));
    }

    #[test]
    fn test_create_file_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        let file_path = temp_path.join("test.py");

        File::create(&file_path).unwrap();

        let metadata = create_file_metadata(&file_path).unwrap();

        assert_eq!(metadata.language, "python");
        assert!(metadata.path.ends_with("test.py"));
    }

    #[test]
    fn test_parallel_indexing() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create multiple test files
        for i in 0..10 {
            File::create(temp_path.join(format!("test{}.py", i))).unwrap();
        }

        let config = IndexerConfig {
            root_dir: temp_path.to_path_buf(),
            ..Default::default()
        };

        let files = discover_files(&config).unwrap();
        assert_eq!(files.len(), 10);

        // Test parallel indexing
        let callback = Box::new(|_current: usize, _total: usize| {
            // Progress callback
        });

        let metadata = index_files_with_progress_parallel(&files, callback).unwrap();
        assert_eq!(metadata.len(), 10);

        // All files should be Python
        assert!(metadata.iter().all(|m| m.language == "python"));
    }

    #[test]
    fn test_parallel_reindexing() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create test files
        let files: Vec<PathBuf> = (0..5)
            .map(|i| {
                let path = temp_path.join(format!("test{}.rs", i));
                File::create(&path).unwrap();
                path
            })
            .collect();

        // Test parallel re-indexing with callback
        let callback = Box::new(|_current: usize, _total: usize| {
            // Progress callback
        });

        let metadata = reindex_files_parallel(&files, Some(callback)).unwrap();
        assert_eq!(metadata.len(), 5);

        // All files should be Rust
        assert!(metadata.iter().all(|m| m.language == "rust"));

        // Test without callback
        let metadata = reindex_files_parallel(&files, None).unwrap();
        assert_eq!(metadata.len(), 5);
    }
}
