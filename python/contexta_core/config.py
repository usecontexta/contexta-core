"""Configuration dataclasses for code analysis."""

from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional, List, Callable, Any


@dataclass
class AnalysisConfig:
    """Configuration for code analysis.

    Attributes:
        enable_deep_mode: Enable Deep Mode analysis (requires enterprise license)
        max_file_size: Maximum file size to analyze in bytes (default: 10MB)
        exclude_patterns: List of glob patterns to exclude from analysis
        include_patterns: List of glob patterns to include (if set, only these are analyzed)
        max_depth: Maximum directory depth to traverse (default: unlimited)
        follow_symlinks: Whether to follow symbolic links (default: False)
        audit_callback: Optional callback for Deep Mode audit events
        language_filters: Optional list of language extensions to analyze (e.g., ['.py', '.ts'])
    """

    enable_deep_mode: bool = False
    max_file_size: int = 10 * 1024 * 1024  # 10MB
    exclude_patterns: List[str] = field(default_factory=lambda: [
        "**/node_modules/**",
        "**/.git/**",
        "**/__pycache__/**",
        "**/target/**",
        "**/.venv/**",
        "**/venv/**",
        "**/.pytest_cache/**",
        "**/*.min.js",
    ])
    include_patterns: Optional[List[str]] = None
    max_depth: Optional[int] = None
    follow_symlinks: bool = False
    audit_callback: Optional[Callable[[str, Any], None]] = None
    language_filters: Optional[List[str]] = None

    def __post_init__(self):
        """Validate configuration after initialization."""
        if self.max_file_size <= 0:
            raise ValueError("max_file_size must be positive")

        if self.max_depth is not None and self.max_depth <= 0:
            raise ValueError("max_depth must be positive or None")

        if self.enable_deep_mode and self.audit_callback is None:
            # Deep Mode requires audit callback for compliance
            raise ValueError(
                "Deep Mode requires audit_callback for compliance tracking. "
                "Set audit_callback to a function(event_type: str, data: dict) -> None"
            )
