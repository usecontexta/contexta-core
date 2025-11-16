"""Contexta code analysis core engine.

This package provides fast, accurate code analysis powered by Rust and tree-sitter.
It can be used standalone or as a dependency in other projects.

Example usage:
    >>> from contexta_core import analyze, AnalysisConfig
    >>> result = analyze('/path/to/project')
    >>> print(f"Found {len(result.symbols)} symbols")

Public API:
    - analyze(): Analyze source code and extract symbols/dependencies
    - capabilities(): List available analyzer features
    - check_compatibility(): Check version compatibility
    - AnalysisConfig: Configuration dataclass
    - AnalysisResult: Result dataclass with symbols and dependencies
"""

from pathlib import Path
from typing import Optional, List

# Import configuration and result types
from .config import AnalysisConfig
from .result import AnalysisResult, Symbol, Dependency, SymbolKind

# Version constants
__version__ = "0.1.0"
__api_version__ = "1.0.0"  # Semantic versioning for API compatibility

# Import Rust bindings
try:
    from ._bindings import (
        analyze as _rust_analyze,
        capabilities as _rust_capabilities,
        check_compatibility as _rust_check_compatibility,
    )
except ImportError as e:
    raise ImportError(
        "Failed to import Rust bindings. "
        "Make sure contexta-core is properly installed. "
        f"Error: {e}"
    ) from e


def analyze(
    source: str | Path,
    config: Optional[AnalysisConfig] = None
) -> AnalysisResult:
    """Analyze source code and extract symbols and dependencies.

    Args:
        source: Path to file or directory to analyze
        config: Optional analysis configuration

    Returns:
        AnalysisResult containing symbols, dependencies, and metadata

    Raises:
        ValueError: If source path is invalid
        RuntimeError: If analysis fails

    Example:
        >>> result = analyze('/path/to/project')
        >>> print(result.summary())
    """
    if config is None:
        config = AnalysisConfig()

    # Convert Path to string for Rust FFI
    source_str = str(source) if isinstance(source, Path) else source

    # Call Rust implementation
    return _rust_analyze(source_str, config)


def capabilities() -> List[str]:
    """Return list of available analyzer capabilities.

    Returns:
        List of capability strings

    Example:
        >>> caps = capabilities()
        >>> if 'deep-mode' in caps:
        ...     print("Deep Mode available")
    """
    return _rust_capabilities()


def check_compatibility(client_version: str) -> bool:
    """Check if a client version is compatible with this core version.

    Args:
        client_version: Semantic version string (e.g., "0.1.0")

    Returns:
        True if compatible, False otherwise

    Raises:
        ValueError: If version string is malformed

    Example:
        >>> if check_compatibility("0.1.5"):
        ...     print("Compatible")
    """
    if not isinstance(client_version, str):
        raise ValueError("client_version must be a string")

    return _rust_check_compatibility(client_version)


__all__ = [
    # Main functions
    "analyze",
    "capabilities",
    "check_compatibility",
    # Configuration
    "AnalysisConfig",
    # Results
    "AnalysisResult",
    "Symbol",
    "Dependency",
    "SymbolKind",
    # Version info
    "__version__",
    "__api_version__",
]
