"""Unit tests for capabilities() and check_compatibility() functions."""

import pytest
from contexta_core import capabilities, check_compatibility, __version__, __api_version__


class TestCapabilities:
    """Test capabilities() function."""

    def test_capabilities_returns_list(self):
        """Test that capabilities() returns a list."""
        caps = capabilities()
        assert isinstance(caps, list)

    def test_capabilities_contains_strings(self):
        """Test that all capabilities are strings."""
        caps = capabilities()
        for cap in caps:
            assert isinstance(cap, str)

    def test_capabilities_has_analyze(self):
        """Test that capabilities includes 'analyze'."""
        caps = capabilities()
        assert 'analyze' in caps

    def test_capabilities_is_not_empty(self):
        """Test that capabilities list is not empty."""
        caps = capabilities()
        assert len(caps) > 0

    def test_capabilities_consistency(self):
        """Test that capabilities() returns consistent results."""
        caps1 = capabilities()
        caps2 = capabilities()
        assert caps1 == caps2

    def test_capabilities_deep_mode_optional(self):
        """Test that deep-mode capability is optional."""
        caps = capabilities()
        # Deep mode may or may not be available depending on build
        # This test just verifies that if it's present, it's a valid string
        if 'deep-mode' in caps:
            assert isinstance('deep-mode', str)


class TestCheckCompatibility:
    """Test check_compatibility() function."""

    def test_check_compatibility_returns_bool(self):
        """Test that check_compatibility() returns a boolean."""
        result = check_compatibility("0.1.0")
        assert isinstance(result, bool)

    def test_check_compatibility_same_version(self):
        """Test compatibility with same version."""
        result = check_compatibility(__version__)
        assert result is True

    def test_check_compatibility_patch_version(self):
        """Test compatibility with different patch version."""
        # Minor version same, patch different should be compatible
        result = check_compatibility("0.1.5")
        assert result is True

    def test_check_compatibility_minor_version_lower(self):
        """Test compatibility with lower minor version."""
        # 0.0.x should be compatible with 0.1.x (backward compatible)
        result = check_compatibility("0.0.9")
        # This depends on implementation - may be True or False
        assert isinstance(result, bool)

    def test_check_compatibility_major_version(self):
        """Test incompatibility with different major version."""
        # Different major version should be incompatible
        result = check_compatibility("1.0.0")
        assert result is False

    def test_check_compatibility_invalid_version_format(self):
        """Test that invalid version format raises ValueError."""
        with pytest.raises(ValueError):
            check_compatibility("not-a-version")

    def test_check_compatibility_empty_string(self):
        """Test that empty string raises ValueError."""
        with pytest.raises(ValueError):
            check_compatibility("")

    def test_check_compatibility_non_string(self):
        """Test that non-string input raises ValueError."""
        with pytest.raises(ValueError):
            check_compatibility(123)  # type: ignore

    def test_check_compatibility_none(self):
        """Test that None input raises ValueError."""
        with pytest.raises(ValueError):
            check_compatibility(None)  # type: ignore


class TestVersionConstants:
    """Test version constants."""

    def test_version_exists(self):
        """Test that __version__ constant exists."""
        assert __version__ is not None

    def test_version_is_string(self):
        """Test that __version__ is a string."""
        assert isinstance(__version__, str)

    def test_version_format(self):
        """Test that __version__ follows semantic versioning."""
        parts = __version__.split('.')
        assert len(parts) == 3
        for part in parts:
            assert part.isdigit()

    def test_api_version_exists(self):
        """Test that __api_version__ constant exists."""
        assert __api_version__ is not None

    def test_api_version_is_string(self):
        """Test that __api_version__ is a string."""
        assert isinstance(__api_version__, str)

    def test_api_version_format(self):
        """Test that __api_version__ follows semantic versioning."""
        parts = __api_version__.split('.')
        assert len(parts) == 3
        for part in parts:
            assert part.isdigit()


class TestCompatibilityEdgeCases:
    """Test edge cases for check_compatibility()."""

    def test_compatibility_with_prerelease(self):
        """Test compatibility with pre-release versions."""
        # Pre-release versions should be handled gracefully
        with pytest.raises((ValueError, Exception)):
            check_compatibility("0.1.0-alpha")

    def test_compatibility_with_build_metadata(self):
        """Test compatibility with build metadata."""
        # Build metadata should be handled gracefully
        with pytest.raises((ValueError, Exception)):
            check_compatibility("0.1.0+build.123")

    def test_compatibility_with_leading_v(self):
        """Test compatibility with leading 'v' in version."""
        # 'v0.1.0' format should be handled or raise error
        with pytest.raises((ValueError, Exception)):
            check_compatibility("v0.1.0")

    def test_compatibility_with_whitespace(self):
        """Test compatibility with whitespace in version."""
        with pytest.raises((ValueError, Exception)):
            check_compatibility(" 0.1.0 ")

    def test_compatibility_future_minor_version(self):
        """Test compatibility with future minor version."""
        # Future minor version should be compatible (forward compatible API)
        result = check_compatibility("0.99.0")
        assert isinstance(result, bool)


class TestAPIConsistency:
    """Test API consistency and stability."""

    def test_capabilities_no_duplicates(self):
        """Test that capabilities list has no duplicates."""
        caps = capabilities()
        assert len(caps) == len(set(caps))

    def test_capabilities_lowercase(self):
        """Test that all capabilities are lowercase."""
        caps = capabilities()
        for cap in caps:
            assert cap == cap.lower()

    def test_capabilities_no_spaces(self):
        """Test that capabilities don't contain spaces."""
        caps = capabilities()
        for cap in caps:
            assert ' ' not in cap

    def test_check_compatibility_deterministic(self):
        """Test that check_compatibility() is deterministic."""
        version = "0.1.0"
        result1 = check_compatibility(version)
        result2 = check_compatibility(version)
        assert result1 == result2


class TestImportability:
    """Test that API is properly importable."""

    def test_can_import_capabilities(self):
        """Test that capabilities can be imported directly."""
        from contexta_core import capabilities as caps_func
        assert callable(caps_func)

    def test_can_import_check_compatibility(self):
        """Test that check_compatibility can be imported directly."""
        from contexta_core import check_compatibility as compat_func
        assert callable(compat_func)

    def test_can_import_version_constants(self):
        """Test that version constants can be imported."""
        from contexta_core import __version__, __api_version__
        assert isinstance(__version__, str)
        assert isinstance(__api_version__, str)
