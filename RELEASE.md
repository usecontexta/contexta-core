# Release Process for contexta-core

This document describes the release process for publishing new versions of `contexta-core` to PyPI.

## Overview

The release process is **fully automated** via GitHub Actions using PyPI Trusted Publishing. The workflow is triggered by creating and pushing a git tag.

## Release Workflow

```mermaid
graph LR
    A[Create Tag] --> B[Push Tag]
    B --> C[GitHub Actions]
    C --> D[Build Wheels]
    D --> E[PyPI Publish]
    E --> F[Verify Package]
    F --> G[Update Downstream]
```

## Prerequisites

Before releasing, ensure:

1. ✅ All CI checks are passing on `main` branch
2. ✅ Version number updated in:
   - `Cargo.toml` (Rust package version)
   - `python/contexta_core/__init__.py` (__version__ constant)
3. ✅ CHANGELOG.md updated with release notes
4. ✅ All tests pass locally:
   ```bash
   cargo test --all-features
   maturin develop
   pytest python/tests/
   ```
5. ✅ Code reviewed and merged to `main`

## Versioning Strategy

We follow [Semantic Versioning (SemVer)](https://semver.org/):

- **MAJOR** (X.0.0): Breaking changes to public API
- **MINOR** (0.X.0): New features, backward compatible
- **PATCH** (0.0.X): Bug fixes, backward compatible

### Version Compatibility

- CLI and Cloud worker depend on core via: `contexta-core>=0.1.0,<0.2.0`
- **MINOR** version bumps are safe for existing users
- **MAJOR** version bumps require downstream updates

## Step-by-Step Release Process

### 1. Prepare Release Branch (Optional for Major/Minor)

```bash
# For major/minor releases, create a release branch
git checkout -b release/v0.2.0
```

### 2. Update Version Numbers

```bash
# Update Cargo.toml
vim Cargo.toml
# [package]
# version = "0.2.0"

# Update Python version
vim python/contexta_core/__init__.py
# __version__ = "0.2.0"

# Commit changes
git add Cargo.toml python/contexta_core/__init__.py
git commit -m "chore(release): bump version to 0.2.0"
```

### 3. Update CHANGELOG

```bash
vim CHANGELOG.md
```

Add release notes:

```markdown
## [0.2.0] - 2024-11-18

### Added
- New feature X
- Capability Y

### Changed
- Updated behavior Z

### Fixed
- Bug fix A
- Security fix B

### Breaking Changes
- Removed deprecated API method
```

Commit:

```bash
git add CHANGELOG.md
git commit -m "docs: update CHANGELOG for v0.2.0"
```

### 4. Create and Push Tag

```bash
# Create annotated tag with release notes
git tag -a v0.2.0 -m "Release v0.2.0

Features:
- Add capability X
- Improve performance Y

Bug fixes:
- Fix issue Z

See CHANGELOG.md for full details."

# Push tag to trigger release workflow
git push origin v0.2.0
```

**Note**: Only tags matching `v*.*.*` pattern trigger the release workflow.

### 5. Monitor GitHub Actions

1. Go to: https://github.com/{owner}/contexta-core/actions
2. Find the "Release" workflow run for your tag
3. Monitor the build matrix:
   - ✅ Ubuntu (linux-x86_64, linux-aarch64)
   - ✅ macOS (darwin-x86_64, darwin-aarch64)
   - ✅ Windows (win-amd64)
4. Verify all wheel builds succeed
5. Check PyPI publish step completes

**Typical build time**: 10-15 minutes

### 6. Verify PyPI Publication

Once the workflow completes:

```bash
# Visit PyPI project page
open https://pypi.org/project/contexta-core/

# Or check via API
curl https://pypi.org/pypi/contexta-core/json | jq '.releases | keys | .[-5:]'
```

Verify:
- ✅ New version appears in release list
- ✅ All platform wheels are present (linux, macos, windows)
- ✅ Source distribution (sdist) is present
- ✅ Metadata is correct (description, license, homepage)

### 7. Test Installation

Test the published package in a fresh environment:

```bash
# Create fresh virtual environment
python -m venv test-env
source test-env/bin/activate  # On Windows: test-env\Scripts\activate

# Install from PyPI
pip install contexta-core==0.2.0

# Verify version
python -c "import contexta_core; print(contexta_core.__version__)"
# Expected: 0.2.0

# Test basic functionality
python -c "
from contexta_core import analyze, capabilities
print('Capabilities:', capabilities())
result = analyze('test.py', 'print(\"hello\")')
print('Analysis result:', result)
"

# Deactivate and cleanup
deactivate
rm -rf test-env
```

### 8. Update Downstream Packages

After successful release, update dependent packages:

#### contexta-cli

```bash
cd ../contexta-cli-migration
vim pyproject.toml
# Update: contexta-core = ">=0.2.0,<0.3.0"  # If minor/patch
# Or:     contexta-core = ">=1.0.0,<2.0.0"  # If major

git add pyproject.toml
git commit -m "deps: update contexta-core to v0.2.0"
git push
```

#### contexta-cloud

```bash
cd ../contexta-cloud-migration/worker
vim pyproject.toml
# Update: contexta-core = ">=0.2.0,<0.3.0"

git add pyproject.toml
git commit -m "deps: update contexta-core to v0.2.0"
git push
```

### 9. Create GitHub Release

1. Go to: https://github.com/{owner}/contexta-core/releases
2. Click "Draft a new release"
3. Select the tag: `v0.2.0`
4. Title: `contexta-core v0.2.0`
5. Copy release notes from CHANGELOG.md
6. Attach artifacts (optional):
   - Download wheels from GitHub Actions artifacts
   - Upload to release
7. Click "Publish release"

## Hotfix Releases

For urgent bug fixes on a released version:

```bash
# Create hotfix branch from tag
git checkout -b hotfix/v0.2.1 v0.2.0

# Apply fixes
git cherry-pick <commit-hash>
# Or make manual fixes

# Update version to 0.2.1
# Update CHANGELOG
# Create tag v0.2.1
# Push tag to trigger release
```

## Rollback Procedure

If a release has critical issues:

### Option 1: Yank from PyPI (Recommended)

```bash
# Install twine
pip install twine

# Authenticate (use PyPI API token)
export TWINE_USERNAME=__token__
export TWINE_PASSWORD=pypi-...

# Yank the broken version
twine yank contexta-core 0.2.0 -r pypi -m "Critical bug in feature X"
```

**Note**: Yanked versions can still be installed explicitly but won't be selected by pip by default.

### Option 2: Release New Patch Version

```bash
# Fix the issue
# Bump to v0.2.1
# Release following normal process
```

## Testing Release Workflow (Dry Run)

To test the release workflow without publishing to PyPI:

### Option A: Use TestPyPI

1. Create TestPyPI account: https://test.pypi.org
2. Configure Trusted Publishing for TestPyPI
3. Modify `.github/workflows/release.yml`:
   ```yaml
   repository-url: https://test.pypi.org/legacy/
   ```
4. Create test tag: `v0.2.0-rc1`
5. Monitor workflow
6. Verify on TestPyPI: https://test.pypi.org/project/contexta-core/

### Option B: Manual Local Build

```bash
# Build wheels locally
maturin build --release

# Check dist/ directory
ls -lh dist/

# Test wheel installation
pip install dist/contexta_core-*.whl
```

## Troubleshooting

### Build Fails on Specific Platform

```bash
# Check platform-specific Rust tooling
# Check Cargo.lock is committed
# Review error logs in GitHub Actions
```

### PyPI Publish Fails

```bash
# Check Trusted Publishing is configured correctly
# Verify workflow has permissions: id-token: write
# Check PyPI project name matches: contexta-core
```

### Version Already Exists on PyPI

```bash
# Cannot republish same version
# Must bump version and create new tag
# Delete local tag: git tag -d v0.2.0
# Delete remote tag: git push origin :refs/tags/v0.2.0
# Create new tag with incremented version
```

## Monitoring and Notifications

- **GitHub Actions**: Email notifications on workflow failure
- **PyPI Download Stats**: https://pypistats.org/packages/contexta-core
- **Dependabot**: Automatically opens PRs for downstream updates

## Release Checklist

Use this checklist for each release:

- [ ] All CI checks passing on `main`
- [ ] Version bumped in `Cargo.toml` and `__init__.py`
- [ ] CHANGELOG.md updated with release notes
- [ ] All tests pass locally (`cargo test && pytest`)
- [ ] Code merged to `main`
- [ ] Tag created and pushed (`git tag -a v0.X.Y -m "..."`)
- [ ] GitHub Actions workflow completed successfully
- [ ] All platform wheels published to PyPI
- [ ] PyPI package page verified
- [ ] Package tested in fresh virtualenv
- [ ] Downstream packages updated (CLI, Cloud)
- [ ] GitHub Release created with notes
- [ ] Team notified via Slack/Discord

## References

- [Semantic Versioning](https://semver.org/)
- [PyPI Trusted Publishing](https://docs.pypi.org/trusted-publishers/)
- [GitHub Actions - Building with maturin](https://github.com/PyO3/maturin-action)
- [Python Packaging Guide](https://packaging.python.org/)

---

**Last Updated**: 2024-11-18
**Maintainers**: contexta-core team
