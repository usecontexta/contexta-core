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
from typing import Optional, List, Union
import ast
import os

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
    source: Union[str, Path], config: Optional[AnalysisConfig] = None
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

    # Normalize source to Path
    if isinstance(source, Path):
        source_path = source
    elif isinstance(source, str):
        source_path = Path(source)
    else:
        raise TypeError("source must be a str or Path")

    # Basic existence check to produce clear errors for tests/clients
    if not source_path.exists():
        raise FileNotFoundError(f"Source path does not exist: {source_path}")

    # For now we implement analysis in Python using the stdlib AST module.
    # This keeps tests and clients working while the Rust engine is evolving.
    return _analyze_python_source(source_path, config)


def _analyze_python_source(source_path: Path, config: AnalysisConfig) -> AnalysisResult:
    """Very simple Python-only analyzer used for tests and local dev.

    It walks .py files under the given path and extracts:
    - functions
    - classes
    - methods
    - module-level variables
    """
    python_files: List[Path] = []
    if source_path.is_file():
        if source_path.suffix == ".py":
            python_files.append(source_path)
    elif source_path.is_dir():
        for root, _dirs, files in os.walk(source_path):
            for name in files:
                if name.endswith(".py"):
                    python_files.append(Path(root) / name)
    else:
        raise ValueError(f"Unsupported source path type: {source_path}")

    result = AnalysisResult()
    result.file_count = len(python_files)

    for file_path in python_files:
        try:
            with open(file_path, "r", encoding="utf-8") as f:
                code = f.read()
            tree = ast.parse(code, filename=str(file_path))
        except SyntaxError as exc:
            result.error_count += 1
            result.warnings.append(f"Syntax error in {file_path}: {exc}")
            continue

        result.symbols.extend(_collect_symbols_from_ast(tree, file_path))

    return result


def _collect_symbols_from_ast(node: ast.AST, file_path: Path, scope: Optional[str] = None) -> List[Symbol]:
    """Recursively collect symbols from an AST tree."""
    symbols: List[Symbol] = []

    if isinstance(node, ast.FunctionDef):
        kind = SymbolKind.METHOD if scope is not None else SymbolKind.FUNCTION
        end_line = getattr(node, "end_lineno", node.lineno)
        end_col = getattr(node, "end_col_offset", node.col_offset)
        symbols.append(
            Symbol(
                name=node.name,
                kind=kind,
                file_path=file_path,
                line=node.lineno,
                column=node.col_offset,
                end_line=end_line,
                end_column=end_col,
                scope=scope,
                docstring=ast.get_docstring(node),
            )
        )

    elif isinstance(node, ast.ClassDef):
        end_line = getattr(node, "end_lineno", node.lineno)
        end_col = getattr(node, "end_col_offset", node.col_offset)
        symbols.append(
            Symbol(
                name=node.name,
                kind=SymbolKind.CLASS,
                file_path=file_path,
                line=node.lineno,
                column=node.col_offset,
                end_line=end_line,
                end_column=end_col,
                scope=scope,
                docstring=ast.get_docstring(node),
            )
        )
        # Recurse into class body with class scope for methods
        for child in node.body:
            symbols.extend(_collect_symbols_from_ast(child, file_path, scope=node.name))
        return symbols

    elif isinstance(node, (ast.Assign, ast.AnnAssign)) and scope is None:
        # Module-level variables
        target_names: List[str] = []
        if isinstance(node, ast.Assign):
            for t in node.targets:
                if isinstance(t, ast.Name):
                    target_names.append(t.id)
        else:
            if isinstance(node.target, ast.Name):
                target_names.append(node.target.id)

        for name in target_names:
            line = getattr(node, "lineno", 1)
            col = getattr(node, "col_offset", 0)
            end_line = getattr(node, "end_lineno", line)
            end_col = getattr(node, "end_col_offset", col)
            symbols.append(
                Symbol(
                    name=name,
                    kind=SymbolKind.VARIABLE,
                    file_path=file_path,
                    line=line,
                    column=col,
                    end_line=end_line,
                    end_column=end_col,
                    scope=None,
                    docstring=None,
                )
            )

    # Recurse into children (for non-class nodes or non-body attributes)
    for child in ast.iter_child_nodes(node):
        symbols.extend(_collect_symbols_from_ast(child, file_path, scope=scope))

    return symbols


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

    # Preserve original value for error messages and validation; do not
    # silently accept surrounding whitespace.
    if client_version != client_version.strip():
        raise ValueError(f"Invalid version string (whitespace): {client_version!r}")

    version = client_version
    parts = version.split(".")
    if len(parts) != 3 or not all(part.isdigit() for part in parts):
        # Invalid semantic version format
        raise ValueError(f"Invalid version string: {client_version!r}")

    major, minor, _patch = (int(p) for p in parts)

    # Different major version is incompatible
    if major != 0:
        return False

    # For now we only consider 0.1.x compatible; other 0.x.y are treated as
    # potentially incompatible and return False.
    if minor != 1:
        return False

    # Delegate final decision to the Rust implementation for 0.1.x range
    return _rust_check_compatibility(version)


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
