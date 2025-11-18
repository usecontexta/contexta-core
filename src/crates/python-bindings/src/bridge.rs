// PyO3 bridge module - Exposes Rust analyzer functions to Python
// Implements async bridge with error propagation

use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3_async_runtimes::tokio::future_into_py;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use analyzer_core::{
    indexer::{discover_files, IndexerConfig},
    query::{
        find_exports_by_file, find_imports_by_file, find_symbols_by_file_path,
        find_symbols_by_name, get_file_path_by_id, get_language_stats as query_language_stats,
        list_files as query_list_files,
    },
    storage::{delete_file_symbols, get_file_by_path, init_schema, insert_symbol, upsert_file},
    FileMetadata, Symbol,
};
use analyzer_python::analyze_python;
use analyzer_rust::analyze_rust;
use analyzer_typescript::analyze_typescript;

/// Python wrapper for IndexerConfig
#[pyclass]
#[derive(Clone)]
pub struct PyIndexerConfig {
    #[pyo3(get, set)]
    pub root_dir: String,

    #[pyo3(get, set)]
    pub extensions: Vec<String>,

    #[pyo3(get, set)]
    pub exclude_dirs: Vec<String>,

    #[pyo3(get, set)]
    pub max_file_size: u64,
}

#[pymethods]
impl PyIndexerConfig {
    #[new]
    fn new(root_dir: String) -> Self {
        Self {
            root_dir,
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

    fn __repr__(&self) -> String {
        format!(
            "PyIndexerConfig(root_dir='{}', extensions={:?}, exclude_dirs={:?})",
            self.root_dir, self.extensions, self.exclude_dirs
        )
    }
}

impl From<&PyIndexerConfig> for IndexerConfig {
    fn from(py_config: &PyIndexerConfig) -> Self {
        IndexerConfig {
            root_dir: PathBuf::from(&py_config.root_dir),
            extensions: py_config.extensions.clone(),
            exclude_dirs: py_config.exclude_dirs.clone(),
            max_file_size: py_config.max_file_size,
        }
    }
}

/// Python wrapper for Symbol
#[pyclass]
#[derive(Clone)]
pub struct PySymbol {
    #[pyo3(get)]
    pub id: Option<i64>,

    #[pyo3(get)]
    pub file_id: i64,

    #[pyo3(get)]
    pub name: String,

    #[pyo3(get)]
    pub kind: String,

    #[pyo3(get)]
    pub line_start: usize,

    #[pyo3(get)]
    pub line_end: usize,

    #[pyo3(get)]
    pub scope: Option<String>,
}

#[pymethods]
impl PySymbol {
    fn __repr__(&self) -> String {
        format!(
            "PySymbol(name='{}', kind='{}', lines={}-{})",
            self.name, self.kind, self.line_start, self.line_end
        )
    }

    fn to_dict(&self) -> PyResult<std::collections::HashMap<String, String>> {
        let mut map = std::collections::HashMap::new();
        if let Some(id) = self.id {
            map.insert("id".to_string(), id.to_string());
        }
        map.insert("file_id".to_string(), self.file_id.to_string());
        map.insert("name".to_string(), self.name.clone());
        map.insert("kind".to_string(), self.kind.clone());
        map.insert("line_start".to_string(), self.line_start.to_string());
        map.insert("line_end".to_string(), self.line_end.to_string());
        if let Some(ref scope) = self.scope {
            map.insert("scope".to_string(), scope.clone());
        }
        Ok(map)
    }
}

impl From<Symbol> for PySymbol {
    fn from(symbol: Symbol) -> Self {
        Self {
            id: symbol.id,
            file_id: symbol.file_id,
            name: symbol.name,
            kind: symbol.kind.to_string(),
            line_start: symbol.line_start,
            line_end: symbol.line_end,
            scope: symbol.scope,
        }
    }
}

/// Python wrapper for FileMetadata
#[pyclass]
#[derive(Clone)]
pub struct PyFileMetadata {
    #[pyo3(get)]
    pub path: String,

    #[pyo3(get)]
    pub language: String,

    #[pyo3(get)]
    pub size: u64,

    #[pyo3(get)]
    pub parse_errors: i32,

    #[pyo3(get)]
    pub last_indexed: Option<String>,
}

#[pymethods]
impl PyFileMetadata {
    fn __repr__(&self) -> String {
        format!(
            "PyFileMetadata(path='{}', language='{}', size={})",
            self.path, self.language, self.size
        )
    }

    fn to_dict(&self) -> PyResult<std::collections::HashMap<String, String>> {
        let mut map = std::collections::HashMap::new();
        map.insert("path".to_string(), self.path.clone());
        map.insert("language".to_string(), self.language.clone());
        map.insert("size".to_string(), self.size.to_string());
        map.insert("parse_errors".to_string(), self.parse_errors.to_string());
        if let Some(ref last_indexed) = self.last_indexed {
            map.insert("last_indexed".to_string(), last_indexed.clone());
        }
        Ok(map)
    }
}

impl From<FileMetadata> for PyFileMetadata {
    fn from(metadata: FileMetadata) -> Self {
        Self {
            path: metadata.path,
            language: metadata.language,
            size: metadata.size,
            parse_errors: metadata.parse_errors,
            last_indexed: metadata.last_indexed,
        }
    }
}

/// Main Indexer class for Python
#[pyclass]
pub struct PyIndexer {
    db_path: PathBuf,
    runtime: Arc<tokio::runtime::Runtime>,
}

#[pymethods]
impl PyIndexer {
    #[new]
    fn new(db_path: String) -> PyResult<Self> {
        let runtime = tokio::runtime::Runtime::new().map_err(|e| {
            PyRuntimeError::new_err(format!("Failed to create Tokio runtime: {}", e))
        })?;

        Ok(Self {
            db_path: PathBuf::from(db_path),
            runtime: Arc::new(runtime),
        })
    }

    /// Initialize database schema
    fn init_database(&self) -> PyResult<()> {
        init_schema(&self.db_path).map_err(|e| {
            PyRuntimeError::new_err(format!("Failed to initialize database: {}", e))
        })?;
        Ok(())
    }

    /// Discover files in a directory (synchronous)
    fn discover_files(&self, config: &PyIndexerConfig) -> PyResult<Vec<String>> {
        let rust_config: IndexerConfig = config.into();

        let files = discover_files(&rust_config)
            .map_err(|e| PyRuntimeError::new_err(format!("File discovery failed: {}", e)))?;

        Ok(files
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect())
    }

    /// Index files with progress reporting (async)
    fn index_files<'py>(
        &self,
        py: Python<'py>,
        config: &PyIndexerConfig,
        progress_callback: Option<PyObject>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let rust_config: IndexerConfig = config.into();
        let db_path = self.db_path.clone();
        let runtime = self.runtime.clone();

        future_into_py(py, async move {
            // Run blocking file indexing in Tokio thread pool
            let files = tokio::task::spawn_blocking({
                let config = rust_config.clone();
                move || discover_files(&config)
            })
            .await
            .map_err(|e| PyRuntimeError::new_err(format!("Task join error: {}", e)))?
            .map_err(|e| PyRuntimeError::new_err(format!("File discovery failed: {}", e)))?;

            let total = files.len();
            let mut indexed_files = Vec::new();

            // Index files with progress reporting
            for (index, file_path) in files.iter().enumerate() {
                // Call progress callback if provided
                if let Some(ref callback) = progress_callback {
                    Python::with_gil(|py| {
                        let _ = callback.call1(py, (index + 1, total));
                    });
                }

                // Get file metadata
                let metadata = tokio::task::spawn_blocking({
                    let file_path = file_path.clone();
                    move || {
                        let size = std::fs::metadata(&file_path)?.len();
                        let language = analyzer_core::detect_language(&file_path.to_string_lossy())
                            .unwrap_or("unknown");

                        Ok::<FileMetadata, anyhow::Error>(FileMetadata {
                            id: None,
                            path: file_path.to_string_lossy().to_string(),
                            language: language.to_string(),
                            size,
                            last_indexed: None,
                            parse_errors: 0,
                        })
                    }
                })
                .await
                .map_err(|e| PyRuntimeError::new_err(format!("Task join error: {}", e)))?
                .map_err(|e: anyhow::Error| {
                    PyRuntimeError::new_err(format!("Metadata error: {}", e))
                })?;

                indexed_files.push(PyFileMetadata::from(metadata));
            }

            // Store in database and populate symbols via Tree-sitter analyzers
            tokio::task::spawn_blocking({
                let db_path = db_path.clone();
                let files_to_store = indexed_files.clone();
                move || {
                    let conn = init_schema(&db_path)?;

                    for py_file in &files_to_store {
                        let file_metadata = FileMetadata {
                            id: None,
                            path: py_file.path.clone(),
                            language: py_file.language.clone(),
                            size: py_file.size,
                            last_indexed: py_file.last_indexed.clone(),
                            parse_errors: py_file.parse_errors,
                        };
                        upsert_file(&conn, &file_metadata)?;

                        // Resolve file_id reliably and refresh symbols
                        if let Some(db_file) = get_file_by_path(&conn, &py_file.path)? {
                            let file_id = db_file.id.unwrap_or(0);
                            if file_id > 0 {
                                // Clear old symbols for re-indexing
                                let _ = delete_file_symbols(&conn, file_id);

                                // Read file content
                                let source = std::fs::read_to_string(&py_file.path)
                                    .unwrap_or_else(|_| String::new());

                                // Select analyzer by language
                                let mut extracted: Vec<Symbol> = Vec::new();
                                match py_file.language.as_str() {
                                    "python" => {
                                        if let Ok(mut syms) = analyze_python(&source) {
                                            extracted.append(&mut syms);
                                        }
                                    }
                                    "typescript" | "javascript" => {
                                        if let Ok(mut syms) = analyze_typescript(&source) {
                                            extracted.append(&mut syms);
                                        }
                                    }
                                    "rust" => {
                                        if let Ok(mut syms) = analyze_rust(&source) {
                                            extracted.append(&mut syms);
                                        }
                                    }
                                    _ => {}
                                }

                                // Persist extracted symbols
                                for mut sym in extracted {
                                    sym.file_id = file_id;
                                    let _ = insert_symbol(&conn, &sym);
                                }
                            }
                        }
                    }

                    Ok::<(), anyhow::Error>(())
                }
            })
            .await
            .map_err(|e| PyRuntimeError::new_err(format!("Task join error: {}", e)))?
            .map_err(|e: anyhow::Error| {
                PyRuntimeError::new_err(format!("Database error: {}", e))
            })?;

            Ok(indexed_files)
        })
    }

    /// List all indexed files
    fn list_files(&self) -> PyResult<Vec<PyFileMetadata>> {
        let conn = init_schema(&self.db_path)
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to open database: {}", e)))?;

        let files = query_list_files(&conn)
            .map_err(|e| PyRuntimeError::new_err(format!("Query failed: {}", e)))?;

        Ok(files.into_iter().map(PyFileMetadata::from).collect())
    }

    /// Get language statistics as JSON string
    fn get_language_stats(&self) -> PyResult<String> {
        let conn = init_schema(&self.db_path)
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to open database: {}", e)))?;

        let stats = query_language_stats(&conn)
            .map_err(|e| PyRuntimeError::new_err(format!("Query failed: {}", e)))?;

        Ok(stats.to_string())
    }

    /// Find symbols by name
    fn find_symbols(&self, name: String) -> PyResult<Vec<PySymbol>> {
        let conn = init_schema(&self.db_path)
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to open database: {}", e)))?;

        let symbols = find_symbols_by_name(&conn, &name)
            .map_err(|e| PyRuntimeError::new_err(format!("Query failed: {}", e)))?;

        Ok(symbols.into_iter().map(PySymbol::from).collect())
    }

    /// List all symbols in a specific file
    fn list_symbols_in_file(&self, file_path: String) -> PyResult<Vec<PySymbol>> {
        let conn = init_schema(&self.db_path)
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to open database: {}", e)))?;

        let symbols = find_symbols_by_file_path(&conn, &file_path)
            .map_err(|e| PyRuntimeError::new_err(format!("Query failed: {}", e)))?;

        Ok(symbols.into_iter().map(PySymbol::from).collect())
    }

    /// Find import symbols for a file
    fn find_imports(&self, file_path: String) -> PyResult<Vec<PySymbol>> {
        let conn = init_schema(&self.db_path)
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to open database: {}", e)))?;

        let symbols = find_imports_by_file(&conn, &file_path)
            .map_err(|e| PyRuntimeError::new_err(format!("Query failed: {}", e)))?;

        Ok(symbols.into_iter().map(PySymbol::from).collect())
    }

    /// Find export symbols for a file
    fn find_exports(&self, file_path: String) -> PyResult<Vec<PySymbol>> {
        let conn = init_schema(&self.db_path)
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to open database: {}", e)))?;

        let symbols = find_exports_by_file(&conn, &file_path)
            .map_err(|e| PyRuntimeError::new_err(format!("Query failed: {}", e)))?;

        Ok(symbols.into_iter().map(PySymbol::from).collect())
    }

    /// Get file path by file_id
    fn get_file_path(&self, file_id: i64) -> PyResult<String> {
        let conn = init_schema(&self.db_path)
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to open database: {}", e)))?;

        let path = get_file_path_by_id(&conn, file_id)
            .map_err(|e| PyRuntimeError::new_err(format!("Query failed: {}", e)))?;

        Ok(path)
    }

    fn __repr__(&self) -> String {
        format!("PyIndexer(db_path='{}')", self.db_path.display())
    }
}
