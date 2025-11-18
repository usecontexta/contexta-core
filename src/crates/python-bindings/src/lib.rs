// PyO3 Python bindings for Contexta analyzer-core
// Exposes Rust indexing functionality to Python with async support

use pyo3::prelude::*;

mod bridge;

use bridge::{PyFileMetadata, PyIndexer, PyIndexerConfig};

/// Initialize the Contexta Python module
#[pymodule]
fn _bindings(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyIndexer>()?;
    m.add_class::<PyIndexerConfig>()?;
    m.add_class::<PyFileMetadata>()?;
    Ok(())
}
