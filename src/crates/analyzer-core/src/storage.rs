// Storage module - SQLite database for symbol indexing
// Implements schema initialization, WAL mode, and connection management

use anyhow::{Context, Result};
use rusqlite::{Connection, params};
use std::path::Path;

use crate::{FileMetadata, Symbol};

/// Initialize SQLite database schema with WAL mode
pub fn init_schema(db_path: &Path) -> Result<Connection> {
    let conn = Connection::open(db_path)
        .context("Failed to open SQLite database")?;

    // ========================================
    // Performance Optimizations (PRAGMA settings)
    // ========================================

    // Enable Write-Ahead Logging (WAL) mode for better concurrency
    // Allows multiple readers + 1 writer simultaneously
    conn.pragma_update(None, "journal_mode", "WAL")
        .context("Failed to enable WAL mode")?;

    // Increase cache size to 100MB (negative = kibibytes)
    // More cache = fewer disk I/O operations
    conn.pragma_update(None, "cache_size", -102400)
        .context("Failed to set cache size")?;

    // Use memory-mapped I/O for reads (256MB)
    // Faster reads by mapping DB pages into memory
    conn.pragma_update(None, "mmap_size", 268435456)
        .context("Failed to set mmap size")?;

    // Synchronous = NORMAL (faster writes, still safe with WAL)
    // FULL is slower but safer, NORMAL is good balance with WAL
    conn.pragma_update(None, "synchronous", "NORMAL")
        .context("Failed to set synchronous mode")?;

    // Temp store in memory (faster temp tables/indexes)
    conn.pragma_update(None, "temp_store", "MEMORY")
        .context("Failed to set temp store")?;

    // Auto vacuum incremental (reclaim space gradually)
    conn.pragma_update(None, "auto_vacuum", "INCREMENTAL")
        .context("Failed to set auto vacuum")?;

    // Page size = 4KB (optimal for modern SSDs)
    // Must be set before creating tables
    conn.pragma_update(None, "page_size", 4096)
        .context("Failed to set page size")?;

    // Larger WAL checkpoint threshold (10000 pages ~= 40MB)
    // Fewer checkpoints = better write performance
    conn.pragma_update(None, "wal_autocheckpoint", 10000)
        .context("Failed to set WAL autocheckpoint")?;

    // Optimize for multi-threaded access
    conn.pragma_update(None, "locking_mode", "NORMAL")
        .context("Failed to set locking mode")?;

    // Create tables
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS files (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            path TEXT UNIQUE NOT NULL,
            language TEXT NOT NULL,
            size INTEGER NOT NULL,
            last_indexed TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            parse_errors INTEGER DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS symbols (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            file_id INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
            name TEXT NOT NULL,
            kind TEXT NOT NULL,
            line_start INTEGER NOT NULL,
            line_end INTEGER NOT NULL,
            scope TEXT,
            metadata TEXT,
            UNIQUE(file_id, name, line_start)
        );

        CREATE TABLE IF NOT EXISTS dependencies (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            file_id INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
            import_path TEXT NOT NULL,
            imported_symbols TEXT,
            line_number INTEGER
        );

        -- Indexes for efficient queries
        CREATE INDEX IF NOT EXISTS idx_symbols_name ON symbols(name);
        CREATE INDEX IF NOT EXISTS idx_symbols_kind ON symbols(kind);
        CREATE INDEX IF NOT EXISTS idx_symbols_file_id ON symbols(file_id);
        CREATE INDEX IF NOT EXISTS idx_files_path ON files(path);
        CREATE INDEX IF NOT EXISTS idx_files_language ON files(language);

        -- Composite indexes for common query patterns
        CREATE INDEX IF NOT EXISTS idx_symbols_file_kind ON symbols(file_id, kind);
        CREATE INDEX IF NOT EXISTS idx_symbols_name_kind ON symbols(name, kind);
        CREATE INDEX IF NOT EXISTS idx_symbols_file_line ON symbols(file_id, line_start);

        -- Dependency indexes
        CREATE INDEX IF NOT EXISTS idx_dependencies_file_id ON dependencies(file_id);
        CREATE INDEX IF NOT EXISTS idx_dependencies_import_path ON dependencies(import_path);
        "#,
    )
    .context("Failed to create database schema")?;

    Ok(conn)
}

/// Insert or update file metadata
pub fn upsert_file(conn: &Connection, file: &FileMetadata) -> Result<i64> {
    conn.execute(
        r#"
        INSERT INTO files (path, language, size, last_indexed, parse_errors)
        VALUES (?1, ?2, ?3, CURRENT_TIMESTAMP, ?4)
        ON CONFLICT(path) DO UPDATE SET
            language = excluded.language,
            size = excluded.size,
            last_indexed = CURRENT_TIMESTAMP,
            parse_errors = excluded.parse_errors
        "#,
        params![file.path, file.language, file.size, file.parse_errors],
    )
    .context("Failed to upsert file metadata")?;

    Ok(conn.last_insert_rowid())
}

/// Insert symbol
pub fn insert_symbol(conn: &Connection, symbol: &Symbol) -> Result<i64> {
    conn.execute(
        r#"
        INSERT INTO symbols (file_id, name, kind, line_start, line_end, scope, metadata)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        ON CONFLICT(file_id, name, line_start) DO UPDATE SET
            kind = excluded.kind,
            line_end = excluded.line_end,
            scope = excluded.scope,
            metadata = excluded.metadata
        "#,
        params![
            symbol.file_id,
            symbol.name,
            symbol.kind.to_string(),
            symbol.line_start,
            symbol.line_end,
            symbol.scope,
            symbol.metadata,
        ],
    )
    .context("Failed to insert symbol")?;

    Ok(conn.last_insert_rowid())
}

/// Delete all symbols for a file (used during re-indexing)
pub fn delete_file_symbols(conn: &Connection, file_id: i64) -> Result<()> {
    conn.execute("DELETE FROM symbols WHERE file_id = ?1", params![file_id])
        .context("Failed to delete file symbols")?;
    Ok(())
}

/// Get file by path
pub fn get_file_by_path(conn: &Connection, path: &str) -> Result<Option<FileMetadata>> {
    let mut stmt = conn
        .prepare("SELECT id, path, language, size, last_indexed, parse_errors FROM files WHERE path = ?1")
        .context("Failed to prepare statement")?;

    let mut rows = stmt
        .query(params![path])
        .context("Failed to query file")?;

    if let Some(row) = rows.next().context("Failed to fetch row")? {
        Ok(Some(FileMetadata {
            id: Some(row.get(0)?),
            path: row.get(1)?,
            language: row.get(2)?,
            size: row.get(3)?,
            last_indexed: row.get(4)?,
            parse_errors: row.get(5)?,
        }))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_schema_initialization() {
        let temp_file = NamedTempFile::new().unwrap();
        let conn = init_schema(temp_file.path()).unwrap();

        // Verify WAL mode is enabled
        let journal_mode: String = conn
            .pragma_query_value(None, "journal_mode", |row| row.get(0))
            .unwrap();
        assert_eq!(journal_mode.to_lowercase(), "wal");
    }

    #[test]
    fn test_file_upsert() {
        let temp_file = NamedTempFile::new().unwrap();
        let conn = init_schema(temp_file.path()).unwrap();

        let file = FileMetadata {
            id: None,
            path: "test.py".to_string(),
            language: "python".to_string(),
            size: 1024,
            last_indexed: None,
            parse_errors: 0,
        };

        let file_id = upsert_file(&conn, &file).unwrap();
        assert!(file_id > 0);

        // Verify file was inserted
        let retrieved = get_file_by_path(&conn, "test.py").unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().path, "test.py");
    }

    #[test]
    fn test_symbol_insert() {
        let temp_file = NamedTempFile::new().unwrap();
        let conn = init_schema(temp_file.path()).unwrap();

        let file = FileMetadata {
            id: None,
            path: "test.py".to_string(),
            language: "python".to_string(),
            size: 1024,
            last_indexed: None,
            parse_errors: 0,
        };

        let file_id = upsert_file(&conn, &file).unwrap();

        let symbol = Symbol {
            id: None,
            file_id,
            name: "test_function".to_string(),
            kind: crate::SymbolKind::Function,
            line_start: 10,
            line_end: 20,
            scope: None,
            metadata: None,
        };

        let symbol_id = insert_symbol(&conn, &symbol).unwrap();
        assert!(symbol_id > 0);
    }
}
