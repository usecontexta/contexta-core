"""Contexta code analysis core engine."""

# Import Rust bindings
try:
    from ._bindings import *  # noqa: F401, F403
except ImportError as e:
    raise ImportError(
        "Failed to import Rust bindings. "
        "Make sure contexta-core is properly installed. "
        f"Error: {e}"
    ) from e

__version__ = "0.1.0"
__all__ = ["analyze", "capabilities", "check_compatibility", "__version__"]
