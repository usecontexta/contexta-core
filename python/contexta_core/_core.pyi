"""Type stubs for Rust FFI bindings.

This file provides type hints for functions implemented in Rust and exposed via PyO3.
"""

from pathlib import Path
from typing import Optional, List
from .config import AnalysisConfig
from .result import AnalysisResult

def analyze(
    source: str | Path, config: Optional[AnalysisConfig] = None
) -> AnalysisResult:
    """Analyze a source file or directory using the Rust analyzer core.

    This function is implemented in Rust and exposed via PyO3. It performs
    tree-sitter-based code analysis and returns structured results.

    Args:
        source: Path to file or directory to analyze (str or Path object)
        config: Optional analysis configuration (AnalysisConfig dataclass)

    Returns:
        AnalysisResult containing symbols, dependencies, and metadata

    Raises:
        ValueError: If source path doesn't exist or is invalid
        RuntimeError: If analysis fails due to internal error
        PermissionError: If source path is not readable

    Example:
        >>> from contexta_core import analyze, AnalysisConfig
        >>> result = analyze('/path/to/project')
        >>> print(f"Found {len(result.symbols)} symbols")
        >>>
        >>> # With configuration
        >>> config = AnalysisConfig(enable_deep_mode=False)
        >>> result = analyze('/path/to/project', config)
    """
    ...

def capabilities() -> List[str]:
    """Return list of available analyzer capabilities.

    This function queries the Rust core for supported features. Capabilities
    may vary based on compile-time features (e.g., deep-mode).

    Returns:
        List of capability strings, e.g.:
        ['analyze', 'search', 'python-support', 'typescript-support', 'rust-support']

    Example:
        >>> from contexta_core import capabilities
        >>> caps = capabilities()
        >>> if 'deep-mode' in caps:
        ...     print("Deep Mode is available")
    """
    ...

def check_compatibility(client_version: str) -> bool:
    """Check if a client version is compatible with this core version.

    Uses semantic versioning to determine compatibility. Minor and patch
    version differences are compatible, major version differences are not.

    Args:
        client_version: Semantic version string (e.g., "0.1.0", "1.2.3")

    Returns:
        True if client version is compatible, False otherwise

    Raises:
        ValueError: If version string is malformed

    Example:
        >>> from contexta_core import check_compatibility
        >>> if check_compatibility("0.1.5"):
        ...     print("Client version 0.1.5 is compatible")
        ... else:
        ...     print("Client version incompatible - please upgrade")
    """
    ...
