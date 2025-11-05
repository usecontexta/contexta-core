// PyO3 Python bindings for Contexta analyzer-core
// Exposes Rust indexing functionality to Python with async support

use pyo3::prelude::*;
use pyo3::exceptions::PyRuntimeError;
use pyo3_async_runtimes::tokio::future_into_py;
use std::path::PathBuf;

mod bridge;

use bridge::{PyIndexer, PyIndexerConfig, PyFileMetadata};

/// Initialize the Contexta Python module
#[pymodule]
fn contexta_bindings(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyIndexer>()?;
    m.add_class::<PyIndexerConfig>()?;
    m.add_class::<PyFileMetadata>()?;
    Ok(())
}
