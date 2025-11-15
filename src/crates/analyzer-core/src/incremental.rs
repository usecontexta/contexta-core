// Incremental module - Incremental parsing and update logic
// Implements file change detection and efficient re-indexing

use anyhow::{Context, Result};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver};
use std::time::SystemTime;
use std::fs;

use crate::FileMetadata;

/// Check if a file has been modified since last index
pub fn is_file_modified(
    file_path: &Path,
    last_indexed: Option<&str>,
) -> Result<bool> {
    let metadata = fs::metadata(file_path)
        .context("Failed to read file metadata")?;

    let modified_time = metadata
        .modified()
        .context("Failed to get file modified time")?;

    if let Some(last_indexed_str) = last_indexed {
        // Parse last_indexed timestamp
        // For simplicity, we'll use SystemTime comparison
        // In production, parse the ISO timestamp properly
        let _ = last_indexed_str; // Suppress warning

        // For now, always return true for MVP
        // TODO: Implement proper timestamp parsing
        Ok(true)
    } else {
        // Never indexed before
        Ok(true)
    }
}

/// Calculate file hash for change detection (simple size-based for MVP)
pub fn calculate_file_hash(file_path: &Path) -> Result<u64> {
    let metadata = fs::metadata(file_path)
        .context("Failed to read file metadata")?;

    // For MVP, use file size as a simple hash
    // In production, use a proper hash algorithm (SHA256, etc.)
    Ok(metadata.len())
}

/// Detect which files need re-indexing
pub fn detect_changed_files(
    files: &[FileMetadata],
) -> Result<Vec<String>> {
    let mut changed = Vec::new();

    for file in files {
        let path = Path::new(&file.path);
        if !path.exists() {
            // File was deleted
            continue;
        }

        if is_file_modified(path, file.last_indexed.as_deref())? {
            changed.push(file.path.clone());
        }
    }

    Ok(changed)
}

/// File watcher for detecting changes in real-time
pub struct FileWatcher {
    watcher: RecommendedWatcher,
    receiver: Receiver<Result<Event, notify::Error>>,
}

impl FileWatcher {
    /// Create a new file watcher for the given root directory
    pub fn new(root: &Path) -> Result<Self> {
        let (tx, rx) = channel();

        let watcher = RecommendedWatcher::new(
            move |res| {
                if let Err(e) = tx.send(res) {
                    eprintln!("Failed to send file event: {}", e);
                }
            },
            Config::default(),
        )
        .context("Failed to create file watcher")?;

        Ok(Self {
            watcher,
            receiver: rx,
        })
    }

    /// Start watching a directory
    pub fn watch(&mut self, path: &Path) -> Result<()> {
        self.watcher
            .watch(path, RecursiveMode::Recursive)
            .context("Failed to watch directory")?;
        Ok(())
    }

    /// Get the next file change event (blocking)
    pub fn next_event(&self) -> Option<FileChangeEvent> {
        match self.receiver.recv() {
            Ok(Ok(event)) => Some(FileChangeEvent::from_notify_event(event)),
            Ok(Err(e)) => {
                eprintln!("File watch error: {}", e);
                None
            }
            Err(_) => None, // Channel closed
        }
    }

    /// Try to get the next event without blocking
    pub fn try_next_event(&self) -> Option<FileChangeEvent> {
        match self.receiver.try_recv() {
            Ok(Ok(event)) => Some(FileChangeEvent::from_notify_event(event)),
            Ok(Err(e)) => {
                eprintln!("File watch error: {}", e);
                None
            }
            Err(_) => None,
        }
    }
}

/// File change event with simplified interface
#[derive(Debug, Clone)]
pub struct FileChangeEvent {
    pub kind: FileChangeKind,
    pub paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileChangeKind {
    Create,
    Modify,
    Delete,
    Rename,
    Other,
}

impl FileChangeEvent {
    fn from_notify_event(event: Event) -> Self {
        let kind = match event.kind {
            EventKind::Create(_) => FileChangeKind::Create,
            EventKind::Modify(_) => FileChangeKind::Modify,
            EventKind::Remove(_) => FileChangeKind::Delete,
            EventKind::Any => FileChangeKind::Other,
            _ => FileChangeKind::Other,
        };

        Self {
            kind,
            paths: event.paths,
        }
    }

    /// Check if the event affects a file we care about (by extension)
    pub fn is_relevant_file(&self) -> bool {
        self.paths.iter().any(|path| {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                matches!(ext, "py" | "pyi" | "ts" | "tsx" | "js" | "jsx" | "rs")
            } else {
                false
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_is_file_modified() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "test content").unwrap();

        // Never indexed before
        let result = is_file_modified(temp_file.path(), None).unwrap();
        assert!(result);

        // With last_indexed (always returns true for MVP)
        let result = is_file_modified(
            temp_file.path(),
            Some("2025-01-01T00:00:00Z"),
        )
        .unwrap();
        assert!(result);
    }

    #[test]
    fn test_calculate_file_hash() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "test content").unwrap();
        temp_file.flush().unwrap();

        let hash = calculate_file_hash(temp_file.path()).unwrap();
        assert!(hash > 0);
    }
}
