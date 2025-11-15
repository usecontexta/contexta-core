# Contexta Core

[![PyPI version](https://badge.fury.io/py/contexta-core.svg)](https://pypi.org/project/contexta-core/)
[![License](https://img.shields.io/badge/license-Apache%202.0%20OR%20MIT-blue.svg)](LICENSE)

Contexta code analysis core engine - Rust + Python bindings for fast, accurate code intelligence.

## Features

- 🚀 **Fast**: Rust-powered tree-sitter parsing
- 🔍 **Accurate**: Syntax-aware code analysis
- 🐍 **Pythonic**: Clean Python API via PyO3
- 📦 **Standalone**: Use as a library in any Python project
- 🌐 **Cross-platform**: Linux, macOS, Windows

## Installation

```bash
pip install contexta-core
```

## Quick Start

```python
from contexta_core import analyze, AnalysisConfig

# Analyze a Python file
result = analyze('/path/to/file.py')

print(f"Found {len(result.symbols)} symbols")
for symbol in result.symbols:
    print(f"  {symbol.kind}: {symbol.name}")
```

## API Reference

### `analyze(source, config=None)`

Analyze a source file or directory.

**Parameters:**
- `source` (str | Path): Path to file or directory
- `config` (AnalysisConfig, optional): Analysis configuration

**Returns:** `AnalysisResult` containing symbols and dependencies

**Example:**
```python
from contexta_core import analyze, AnalysisConfig

config = AnalysisConfig(enable_deep_mode=False)
result = analyze('/path/to/project', config)
```

### `capabilities()`

Returns list of available capabilities.

**Returns:** `List[str]` - Capability strings like `['analyze', 'search', 'deep-mode']`

### `check_compatibility(client_version)`

Check if a client version is compatible with this core version.

**Parameters:**
- `client_version` (str): Semantic version string (e.g., "0.1.0")

**Returns:** `bool` - True if compatible

## Development

### Building from Source

Requires Rust 1.75+ and Python 3.8+:

```bash
# Install Maturin
pip install maturin

# Build release wheel
maturin build --release

# Install locally
pip install target/wheels/*.whl
```

### Running Tests

```bash
# Rust tests
cargo test

# Python tests
pytest python/tests/
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Contributing

Contributions are welcome! Please see our [Contributing Guide](CONTRIBUTING.md).
