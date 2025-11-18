"""Unit tests for analyze() function."""

import pytest
from pathlib import Path
import tempfile
import os

from contexta_core import analyze, AnalysisConfig, AnalysisResult, SymbolKind


@pytest.fixture
def temp_python_file():
    """Create a temporary Python file for testing."""
    with tempfile.NamedTemporaryFile(mode="w", suffix=".py", delete=False) as f:
        f.write(
            """
def hello_world():
    '''Say hello to the world.'''
    return "Hello, World!"

class Calculator:
    '''Simple calculator class.'''

    def add(self, a: int, b: int) -> int:
        '''Add two numbers.'''
        return a + b

    def subtract(self, a: int, b: int) -> int:
        '''Subtract b from a.'''
        return a - b

PI = 3.14159
"""
        )
        temp_path = f.name

    yield temp_path

    # Cleanup
    if os.path.exists(temp_path):
        os.unlink(temp_path)


@pytest.fixture
def temp_directory():
    """Create a temporary directory with multiple Python files."""
    temp_dir = tempfile.mkdtemp()

    # Create main.py
    with open(os.path.join(temp_dir, "main.py"), "w") as f:
        f.write(
            """
def main():
    print("Hello from main")

if __name__ == "__main__":
    main()
"""
        )

    # Create utils.py
    with open(os.path.join(temp_dir, "utils.py"), "w") as f:
        f.write(
            """
def helper():
    return "Helper function"
"""
        )

    yield temp_dir

    # Cleanup
    import shutil

    if os.path.exists(temp_dir):
        shutil.rmtree(temp_dir)


class TestAnalyzeBasic:
    """Test basic analyze() functionality."""

    def test_analyze_returns_result(self, temp_python_file):
        """Test that analyze() returns an AnalysisResult."""
        result = analyze(temp_python_file)
        assert isinstance(result, AnalysisResult)

    def test_analyze_with_path_object(self, temp_python_file):
        """Test that analyze() accepts Path objects."""
        result = analyze(Path(temp_python_file))
        assert isinstance(result, AnalysisResult)

    def test_analyze_with_string_path(self, temp_python_file):
        """Test that analyze() accepts string paths."""
        result = analyze(str(temp_python_file))
        assert isinstance(result, AnalysisResult)

    def test_analyze_finds_symbols(self, temp_python_file):
        """Test that analyze() finds symbols in the file."""
        result = analyze(temp_python_file)
        assert len(result.symbols) > 0
        assert result.file_count >= 1

    def test_analyze_finds_function(self, temp_python_file):
        """Test that analyze() finds function definitions."""
        result = analyze(temp_python_file)
        function_names = [
            s.name for s in result.symbols if s.kind == SymbolKind.FUNCTION
        ]
        assert "hello_world" in function_names

    def test_analyze_finds_class(self, temp_python_file):
        """Test that analyze() finds class definitions."""
        result = analyze(temp_python_file)
        class_names = [s.name for s in result.symbols if s.kind == SymbolKind.CLASS]
        assert "Calculator" in class_names

    def test_analyze_finds_methods(self, temp_python_file):
        """Test that analyze() finds class methods."""
        result = analyze(temp_python_file)
        method_names = [s.name for s in result.symbols if s.kind == SymbolKind.METHOD]
        assert "add" in method_names
        assert "subtract" in method_names

    def test_analyze_finds_variable(self, temp_python_file):
        """Test that analyze() finds module-level variables."""
        result = analyze(temp_python_file)
        var_names = [s.name for s in result.symbols if s.kind == SymbolKind.VARIABLE]
        assert "PI" in var_names


class TestAnalyzeDirectory:
    """Test analyze() with directories."""

    def test_analyze_directory(self, temp_directory):
        """Test that analyze() can process a directory."""
        result = analyze(temp_directory)
        assert isinstance(result, AnalysisResult)
        assert result.file_count >= 2

    def test_analyze_directory_finds_multiple_functions(self, temp_directory):
        """Test that analyze() finds functions across multiple files."""
        result = analyze(temp_directory)
        function_names = [
            s.name for s in result.symbols if s.kind == SymbolKind.FUNCTION
        ]
        assert "main" in function_names
        assert "helper" in function_names


class TestAnalyzeConfig:
    """Test analyze() with configuration options."""

    def test_analyze_with_default_config(self, temp_python_file):
        """Test analyze() with default configuration."""
        config = AnalysisConfig()
        result = analyze(temp_python_file, config)
        assert isinstance(result, AnalysisResult)

    def test_analyze_with_custom_config(self, temp_python_file):
        """Test analyze() with custom configuration."""
        config = AnalysisConfig(
            enable_deep_mode=False,
            max_file_size=5 * 1024 * 1024,
            exclude_patterns=["**/test_*.py"],
        )
        result = analyze(temp_python_file, config)
        assert isinstance(result, AnalysisResult)

    def test_analyze_without_config(self, temp_python_file):
        """Test analyze() without explicit config (uses default)."""
        result = analyze(temp_python_file)
        assert isinstance(result, AnalysisResult)


class TestAnalyzeErrors:
    """Test error handling in analyze()."""

    def test_analyze_nonexistent_path(self):
        """Test analyze() with non-existent path."""
        with pytest.raises((ValueError, RuntimeError, FileNotFoundError)):
            analyze("/nonexistent/path/to/file.py")

    def test_analyze_invalid_path_type(self):
        """Test analyze() with invalid path type."""
        with pytest.raises((TypeError, ValueError)):
            analyze(12345)  # type: ignore

    def test_deep_mode_without_callback(self):
        """Test that Deep Mode requires audit callback."""
        with pytest.raises(ValueError, match="Deep Mode requires audit_callback"):
            _ = AnalysisConfig(enable_deep_mode=True)


class TestAnalysisResult:
    """Test AnalysisResult structure."""

    def test_result_has_symbols(self, temp_python_file):
        """Test that result contains symbols list."""
        result = analyze(temp_python_file)
        assert hasattr(result, "symbols")
        assert isinstance(result.symbols, list)

    def test_result_has_dependencies(self, temp_python_file):
        """Test that result contains dependencies list."""
        result = analyze(temp_python_file)
        assert hasattr(result, "dependencies")
        assert isinstance(result.dependencies, list)

    def test_result_has_file_count(self, temp_python_file):
        """Test that result contains file count."""
        result = analyze(temp_python_file)
        assert hasattr(result, "file_count")
        assert isinstance(result.file_count, int)
        assert result.file_count >= 1

    def test_result_has_error_count(self, temp_python_file):
        """Test that result contains error count."""
        result = analyze(temp_python_file)
        assert hasattr(result, "error_count")
        assert isinstance(result.error_count, int)

    def test_result_has_warnings(self, temp_python_file):
        """Test that result contains warnings list."""
        result = analyze(temp_python_file)
        assert hasattr(result, "warnings")
        assert isinstance(result.warnings, list)

    def test_result_has_metadata(self, temp_python_file):
        """Test that result contains metadata dict."""
        result = analyze(temp_python_file)
        assert hasattr(result, "metadata")
        assert isinstance(result.metadata, dict)


class TestSymbolStructure:
    """Test Symbol dataclass structure."""

    def test_symbol_has_name(self, temp_python_file):
        """Test that symbols have name attribute."""
        result = analyze(temp_python_file)
        for symbol in result.symbols:
            assert hasattr(symbol, "name")
            assert isinstance(symbol.name, str)

    def test_symbol_has_kind(self, temp_python_file):
        """Test that symbols have kind attribute."""
        result = analyze(temp_python_file)
        for symbol in result.symbols:
            assert hasattr(symbol, "kind")
            assert isinstance(symbol.kind, SymbolKind)

    def test_symbol_has_location(self, temp_python_file):
        """Test that symbols have location attributes."""
        result = analyze(temp_python_file)
        for symbol in result.symbols:
            assert hasattr(symbol, "file_path")
            assert hasattr(symbol, "line")
            assert hasattr(symbol, "column")
            assert hasattr(symbol, "end_line")
            assert hasattr(symbol, "end_column")
            assert isinstance(symbol.line, int)
            assert isinstance(symbol.column, int)

    def test_symbol_has_optional_fields(self, temp_python_file):
        """Test that symbols have optional fields."""
        result = analyze(temp_python_file)
        for symbol in result.symbols:
            assert hasattr(symbol, "scope")
            assert hasattr(symbol, "docstring")
            assert hasattr(symbol, "metadata")
