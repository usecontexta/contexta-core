"""Result dataclasses for code analysis."""

from dataclasses import dataclass, field
from pathlib import Path
from typing import List, Optional, Dict, Any
from enum import Enum


class SymbolKind(str, Enum):
    """Symbol types identified during analysis."""

    FUNCTION = "function"
    CLASS = "class"
    METHOD = "method"
    VARIABLE = "variable"
    CONSTANT = "constant"
    MODULE = "module"
    INTERFACE = "interface"
    TYPE_ALIAS = "type_alias"
    ENUM = "enum"
    IMPORT = "import"
    EXPORT = "export"
    UNKNOWN = "unknown"


@dataclass
class Symbol:
    """A code symbol (function, class, variable, etc.) extracted from analysis.

    Attributes:
        name: Symbol name (e.g., 'calculate_total', 'UserModel')
        kind: Symbol type (function, class, method, etc.)
        file_path: Absolute path to the file containing this symbol
        line: Line number where symbol is defined (1-indexed)
        column: Column number where symbol starts (0-indexed)
        end_line: Line number where symbol ends (1-indexed)
        end_column: Column number where symbol ends (0-indexed)
        scope: Optional parent scope (e.g., 'MyClass.my_method')
        docstring: Optional documentation string
        metadata: Additional language-specific metadata
    """

    name: str
    kind: SymbolKind
    file_path: Path
    line: int
    column: int
    end_line: int
    end_column: int
    scope: Optional[str] = None
    docstring: Optional[str] = None
    metadata: Dict[str, Any] = field(default_factory=dict)

    def __post_init__(self):
        """Convert file_path to Path if it's a string."""
        if isinstance(self.file_path, str):
            self.file_path = Path(self.file_path)

        # Convert string kind to SymbolKind enum
        if isinstance(self.kind, str):
            try:
                self.kind = SymbolKind(self.kind)
            except ValueError:
                self.kind = SymbolKind.UNKNOWN


@dataclass
class Dependency:
    """A dependency relationship between symbols or modules.

    Attributes:
        source: Source symbol or module path
        target: Target symbol or module path
        kind: Dependency type ('import', 'call', 'inheritance', 'reference')
        line: Line number where dependency occurs (1-indexed)
        metadata: Additional dependency metadata
    """

    source: str
    target: str
    kind: str
    line: int
    metadata: Dict[str, Any] = field(default_factory=dict)


@dataclass
class AnalysisResult:
    """Complete analysis result for a codebase.

    Attributes:
        symbols: List of all symbols found in the analyzed code
        dependencies: List of all dependency relationships
        file_count: Number of files analyzed
        error_count: Number of files that failed to parse
        warnings: List of warning messages
        metadata: Additional analysis metadata (timing, config, etc.)
    """

    symbols: List[Symbol] = field(default_factory=list)
    dependencies: List[Dependency] = field(default_factory=list)
    file_count: int = 0
    error_count: int = 0
    warnings: List[str] = field(default_factory=list)
    metadata: Dict[str, Any] = field(default_factory=dict)

    def summary(self) -> str:
        """Generate a human-readable summary of the analysis."""
        return (
            f"Analysis Summary:\n"
            f"  Files analyzed: {self.file_count}\n"
            f"  Symbols found: {len(self.symbols)}\n"
            f"  Dependencies: {len(self.dependencies)}\n"
            f"  Errors: {self.error_count}\n"
            f"  Warnings: {len(self.warnings)}"
        )
