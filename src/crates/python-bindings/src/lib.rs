// PyO3 Python bindings for Contexta analyzer-core
// Exposes Rust indexing functionality to Python with async support

use pyo3::prelude::*;
use pyo3::types::PyDict;

mod bridge;

use bridge::{PyFileMetadata, PyIndexer, PyIndexerConfig};

/// Placeholder analyze function - returns empty result for now
#[pyfunction]
fn analyze(py: Python, _source: String, _config: Option<PyObject>) -> PyResult<PyObject> {
    // Create empty result dict
    let result = PyDict::new(py);
    result.set_item("symbols", Vec::<String>::new())?;
    result.set_item("dependencies", Vec::<String>::new())?;
    Ok(result.into())
}

/// Return list of available analyzer capabilities
#[pyfunction]
fn capabilities() -> PyResult<Vec<String>> {
    let mut caps = vec![
        "analyze".to_string(),
        "python".to_string(),
        "typescript".to_string(),
        "javascript".to_string(),
        "rust".to_string(),
    ];

    #[cfg(feature = "deep-mode")]
    caps.push("deep-mode".to_string());

    Ok(caps)
}

/// Check if a client version is compatible with this core version
#[pyfunction]
fn check_compatibility(client_version: String) -> PyResult<bool> {
    // Simple version compatibility check
    // For now, accept any 0.1.x version
    Ok(client_version.starts_with("0.1."))
}

/// Initialize the Contexta Python module
#[pymodule]
fn _bindings(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Add classes
    m.add_class::<PyIndexer>()?;
    m.add_class::<PyIndexerConfig>()?;
    m.add_class::<PyFileMetadata>()?;

    // Add functions
    m.add_function(wrap_pyfunction!(analyze, m)?)?;
    m.add_function(wrap_pyfunction!(capabilities, m)?)?;
    m.add_function(wrap_pyfunction!(check_compatibility, m)?)?;

    Ok(())
}
