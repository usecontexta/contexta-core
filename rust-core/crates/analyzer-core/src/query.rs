// Query module - Symbol and file query engine
// Implements efficient SQLite queries for MCP protocol

use anyhow::{Context, Result};
use rusqlite::{Connection, params};
use serde_json::json;

use crate::{Symbol, SymbolKind, FileMetadata};

/// Query symbols by name
pub fn find_symbols_by_name(
    conn: &Connection,
    name: &str,
) -> Result<Vec<Symbol>> {
    let mut stmt = conn.prepare(
        "SELECT id, file_id, name, kind, line_start, line_end, scope, metadata
         FROM symbols WHERE name = ?1"
    )?;

    let symbols = stmt.query_map(params![name], |row| {
        Ok(Symbol {
            id: Some(row.get(0)?),
            file_id: row.get(1)?,
            name: row.get(2)?,
            kind: parse_symbol_kind(&row.get::<_, String>(3)?),
            line_start: row.get(4)?,
            line_end: row.get(5)?,
            scope: row.get(6)?,
            metadata: row.get(7)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(symbols)
}

/// Query symbols by kind
pub fn find_symbols_by_kind(
    conn: &Connection,
    kind: SymbolKind,
) -> Result<Vec<Symbol>> {
    let mut stmt = conn.prepare(
        "SELECT id, file_id, name, kind, line_start, line_end, scope, metadata
         FROM symbols WHERE kind = ?1"
    )?;

    let symbols = stmt.query_map(params![kind.to_string()], |row| {
        Ok(Symbol {
            id: Some(row.get(0)?),
            file_id: row.get(1)?,
            name: row.get(2)?,
            kind: parse_symbol_kind(&row.get::<_, String>(3)?),
            line_start: row.get(4)?,
            line_end: row.get(5)?,
            scope: row.get(6)?,
            metadata: row.get(7)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(symbols)
}

/// List all files in the index
pub fn list_files(conn: &Connection) -> Result<Vec<FileMetadata>> {
    let mut stmt = conn.prepare(
        "SELECT id, path, language, size, last_indexed, parse_errors FROM files"
    )?;

    let files = stmt.query_map([], |row| {
        Ok(FileMetadata {
            id: Some(row.get(0)?),
            path: row.get(1)?,
            language: row.get(2)?,
            size: row.get(3)?,
            last_indexed: row.get(4)?,
            parse_errors: row.get(5)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(files)
}

/// Get language statistics
pub fn get_language_stats(conn: &Connection) -> Result<serde_json::Value> {
    let mut stmt = conn.prepare(
        "SELECT language, COUNT(*) as count, SUM(size) as total_size
         FROM files GROUP BY language"
    )?;

    let mut stats = Vec::new();
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, i64>(2)?,
        ))
    })?;

    for row in rows {
        let (language, count, total_size) = row?;
        stats.push(json!({
            "language": language,
            "file_count": count,
            "total_size": total_size,
        }));
    }

    Ok(json!(stats))
}

/// Query symbols by file path
pub fn find_symbols_by_file_path(
    conn: &Connection,
    file_path: &str,
) -> Result<Vec<Symbol>> {
    // First, find the file_id for the given path
    let file_id: i64 = conn.query_row(
        "SELECT id FROM files WHERE path = ?1",
        params![file_path],
        |row| row.get(0)
    ).context("File not found in database")?;

    // Query all symbols for this file
    let mut stmt = conn.prepare(
        "SELECT id, file_id, name, kind, line_start, line_end, scope, metadata
         FROM symbols WHERE file_id = ?1 ORDER BY line_start"
    )?;

    let symbols = stmt.query_map(params![file_id], |row| {
        Ok(Symbol {
            id: Some(row.get(0)?),
            file_id: row.get(1)?,
            name: row.get(2)?,
            kind: parse_symbol_kind(&row.get::<_, String>(3)?),
            line_start: row.get(4)?,
            line_end: row.get(5)?,
            scope: row.get(6)?,
            metadata: row.get(7)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(symbols)
}

/// Query import symbols for a file
pub fn find_imports_by_file(
    conn: &Connection,
    file_path: &str,
) -> Result<Vec<Symbol>> {
    // First, find the file_id for the given path
    let file_id: i64 = conn.query_row(
        "SELECT id FROM files WHERE path = ?1",
        params![file_path],
        |row| row.get(0)
    ).context("File not found in database")?;

    // Query import symbols for this file
    let mut stmt = conn.prepare(
        "SELECT id, file_id, name, kind, line_start, line_end, scope, metadata
         FROM symbols WHERE file_id = ?1 AND kind = 'import' ORDER BY line_start"
    )?;

    let symbols = stmt.query_map(params![file_id], |row| {
        Ok(Symbol {
            id: Some(row.get(0)?),
            file_id: row.get(1)?,
            name: row.get(2)?,
            kind: parse_symbol_kind(&row.get::<_, String>(3)?),
            line_start: row.get(4)?,
            line_end: row.get(5)?,
            scope: row.get(6)?,
            metadata: row.get(7)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(symbols)
}

/// Query export symbols for a file
pub fn find_exports_by_file(
    conn: &Connection,
    file_path: &str,
) -> Result<Vec<Symbol>> {
    // First, find the file_id for the given path
    let file_id: i64 = conn.query_row(
        "SELECT id FROM files WHERE path = ?1",
        params![file_path],
        |row| row.get(0)
    ).context("File not found in database")?;

    // Query export symbols for this file
    let mut stmt = conn.prepare(
        "SELECT id, file_id, name, kind, line_start, line_end, scope, metadata
         FROM symbols WHERE file_id = ?1 AND kind = 'export' ORDER BY line_start"
    )?;

    let symbols = stmt.query_map(params![file_id], |row| {
        Ok(Symbol {
            id: Some(row.get(0)?),
            file_id: row.get(1)?,
            name: row.get(2)?,
            kind: parse_symbol_kind(&row.get::<_, String>(3)?),
            line_start: row.get(4)?,
            line_end: row.get(5)?,
            scope: row.get(6)?,
            metadata: row.get(7)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(symbols)
}

/// Get file path by file_id
pub fn get_file_path_by_id(
    conn: &Connection,
    file_id: i64,
) -> Result<String> {
    let path: String = conn.query_row(
        "SELECT path FROM files WHERE id = ?1",
        params![file_id],
        |row| row.get(0)
    ).context("File not found in database")?;

    Ok(path)
}

/// Analyze query plan for a given SQL statement
pub fn analyze_query_plan(
    conn: &Connection,
    query: &str,
) -> Result<String> {
    let mut stmt = conn.prepare(&format!("EXPLAIN QUERY PLAN {}", query))?;

    let mut plan = String::new();
    let rows = stmt.query_map([], |row| {
        let detail: String = row.get(3)?;
        Ok(detail)
    })?;

    for row in rows {
        let detail = row?;
        plan.push_str(&detail);
        plan.push('\n');
    }

    Ok(plan)
}

/// Run ANALYZE to update query planner statistics
pub fn update_query_statistics(conn: &Connection) -> Result<()> {
    conn.execute("ANALYZE", [])?;
    Ok(())
}

/// Optimize database by running VACUUM and ANALYZE
pub fn optimize_database(conn: &Connection) -> Result<()> {
    // VACUUM reclaims space from deleted records
    conn.execute("VACUUM", [])?;

    // ANALYZE updates query planner statistics
    conn.execute("ANALYZE", [])?;

    Ok(())
}

/// Query symbols by name and kind (optimized with composite index)
pub fn find_symbols_by_name_and_kind(
    conn: &Connection,
    name: &str,
    kind: SymbolKind,
) -> Result<Vec<Symbol>> {
    // Uses idx_symbols_name_kind composite index
    let mut stmt = conn.prepare(
        "SELECT id, file_id, name, kind, line_start, line_end, scope, metadata
         FROM symbols WHERE name = ?1 AND kind = ?2"
    )?;

    let symbols = stmt.query_map(params![name, kind.to_string()], |row| {
        Ok(Symbol {
            id: Some(row.get(0)?),
            file_id: row.get(1)?,
            name: row.get(2)?,
            kind: parse_symbol_kind(&row.get::<_, String>(3)?),
            line_start: row.get(4)?,
            line_end: row.get(5)?,
            scope: row.get(6)?,
            metadata: row.get(7)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(symbols)
}

/// Query symbols by file_id and kind (optimized with composite index)
pub fn find_symbols_by_file_and_kind(
    conn: &Connection,
    file_id: i64,
    kind: SymbolKind,
) -> Result<Vec<Symbol>> {
    // Uses idx_symbols_file_kind composite index
    let mut stmt = conn.prepare(
        "SELECT id, file_id, name, kind, line_start, line_end, scope, metadata
         FROM symbols WHERE file_id = ?1 AND kind = ?2 ORDER BY line_start"
    )?;

    let symbols = stmt.query_map(params![file_id, kind.to_string()], |row| {
        Ok(Symbol {
            id: Some(row.get(0)?),
            file_id: row.get(1)?,
            name: row.get(2)?,
            kind: parse_symbol_kind(&row.get::<_, String>(3)?),
            line_start: row.get(4)?,
            line_end: row.get(5)?,
            scope: row.get(6)?,
            metadata: row.get(7)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(symbols)
}

fn parse_symbol_kind(s: &str) -> SymbolKind {
    match s.to_lowercase().as_str() {
        "function" => SymbolKind::Function,
        "class" => SymbolKind::Class,
        "variable" => SymbolKind::Variable,
        "import" => SymbolKind::Import,
        "export" => SymbolKind::Export,
        "module" => SymbolKind::Module,
        "struct" => SymbolKind::Struct,
        "enum" => SymbolKind::Enum,
        "trait" => SymbolKind::Trait,
        "interface" => SymbolKind::Interface,
        "type" => SymbolKind::Type,
        _ => SymbolKind::Variable, // Default fallback
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{init_schema, upsert_file, insert_symbol};
    use tempfile::NamedTempFile;

    #[test]
    fn test_find_symbols_by_name() {
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
            name: "my_function".to_string(),
            kind: SymbolKind::Function,
            line_start: 10,
            line_end: 20,
            scope: None,
            metadata: None,
        };
        insert_symbol(&conn, &symbol).unwrap();

        let found = find_symbols_by_name(&conn, "my_function").unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].name, "my_function");
    }

    #[test]
    fn test_language_stats() {
        let temp_file = NamedTempFile::new().unwrap();
        let conn = init_schema(temp_file.path()).unwrap();

        let file1 = FileMetadata {
            id: None,
            path: "test1.py".to_string(),
            language: "python".to_string(),
            size: 1024,
            last_indexed: None,
            parse_errors: 0,
        };
        upsert_file(&conn, &file1).unwrap();

        let file2 = FileMetadata {
            id: None,
            path: "test2.py".to_string(),
            language: "python".to_string(),
            size: 2048,
            last_indexed: None,
            parse_errors: 0,
        };
        upsert_file(&conn, &file2).unwrap();

        let stats = get_language_stats(&conn).unwrap();
        let stats_array = stats.as_array().unwrap();
        assert_eq!(stats_array.len(), 1);
        assert_eq!(stats_array[0]["language"], "python");
        assert_eq!(stats_array[0]["file_count"], 2);
    }

    #[test]
    fn test_find_symbols_by_file_path() {
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

        let symbol1 = Symbol {
            id: None,
            file_id,
            name: "my_function".to_string(),
            kind: SymbolKind::Function,
            line_start: 10,
            line_end: 20,
            scope: None,
            metadata: None,
        };
        insert_symbol(&conn, &symbol1).unwrap();

        let symbol2 = Symbol {
            id: None,
            file_id,
            name: "MyClass".to_string(),
            kind: SymbolKind::Class,
            line_start: 25,
            line_end: 40,
            scope: None,
            metadata: None,
        };
        insert_symbol(&conn, &symbol2).unwrap();

        let found = find_symbols_by_file_path(&conn, "test.py").unwrap();
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].name, "my_function");
        assert_eq!(found[1].name, "MyClass");
    }

    #[test]
    fn test_find_imports_by_file() {
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

        let import1 = Symbol {
            id: None,
            file_id,
            name: "os".to_string(),
            kind: SymbolKind::Import,
            line_start: 1,
            line_end: 1,
            scope: None,
            metadata: None,
        };
        insert_symbol(&conn, &import1).unwrap();

        let import2 = Symbol {
            id: None,
            file_id,
            name: "sys".to_string(),
            kind: SymbolKind::Import,
            line_start: 2,
            line_end: 2,
            scope: None,
            metadata: None,
        };
        insert_symbol(&conn, &import2).unwrap();

        let function = Symbol {
            id: None,
            file_id,
            name: "my_function".to_string(),
            kind: SymbolKind::Function,
            line_start: 10,
            line_end: 20,
            scope: None,
            metadata: None,
        };
        insert_symbol(&conn, &function).unwrap();

        let imports = find_imports_by_file(&conn, "test.py").unwrap();
        assert_eq!(imports.len(), 2);
        assert_eq!(imports[0].name, "os");
        assert_eq!(imports[1].name, "sys");
    }

    #[test]
    fn test_get_file_path_by_id() {
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

        let path = get_file_path_by_id(&conn, file_id).unwrap();
        assert_eq!(path, "test.py");
    }

    #[test]
    fn test_find_symbols_by_name_and_kind() {
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

        // Insert function with same name
        let symbol1 = Symbol {
            id: None,
            file_id,
            name: "process".to_string(),
            kind: SymbolKind::Function,
            line_start: 10,
            line_end: 20,
            scope: None,
            metadata: None,
        };
        insert_symbol(&conn, &symbol1).unwrap();

        // Insert variable with same name
        let symbol2 = Symbol {
            id: None,
            file_id,
            name: "process".to_string(),
            kind: SymbolKind::Variable,
            line_start: 5,
            line_end: 5,
            scope: None,
            metadata: None,
        };
        insert_symbol(&conn, &symbol2).unwrap();

        // Query for function only
        let functions = find_symbols_by_name_and_kind(&conn, "process", SymbolKind::Function).unwrap();
        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].kind, SymbolKind::Function);

        // Query for variable only
        let variables = find_symbols_by_name_and_kind(&conn, "process", SymbolKind::Variable).unwrap();
        assert_eq!(variables.len(), 1);
        assert_eq!(variables[0].kind, SymbolKind::Variable);
    }

    #[test]
    fn test_optimize_database() {
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
            name: "test_func".to_string(),
            kind: SymbolKind::Function,
            line_start: 1,
            line_end: 10,
            scope: None,
            metadata: None,
        };
        insert_symbol(&conn, &symbol).unwrap();

        // Run optimization
        optimize_database(&conn).unwrap();

        // Verify database still works after optimization
        let symbols = find_symbols_by_name(&conn, "test_func").unwrap();
        assert_eq!(symbols.len(), 1);
    }

    #[test]
    fn test_analyze_query_plan() {
        let temp_file = NamedTempFile::new().unwrap();
        let conn = init_schema(temp_file.path()).unwrap();

        let query = "SELECT * FROM symbols WHERE name = 'test'";
        let plan = analyze_query_plan(&conn, query).unwrap();

        // Query plan should mention the index
        assert!(plan.contains("idx_symbols_name") || plan.contains("SEARCH"));
    }
}
