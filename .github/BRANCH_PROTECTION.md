# Branch Protection Configuration

This document describes the required branch protection rules for the `main` branch.

## Required Settings

### Branch Protection Rule for `main`

Navigate to: Settings → Branches → Add branch protection rule

**Branch name pattern**: `main`

#### Protection Settings

- ✅ **Require a pull request before merging**
  - Require approvals: **1**
  - Dismiss stale pull request approvals when new commits are pushed
  - Require review from Code Owners (if CODEOWNERS file exists)

- ✅ **Require status checks to pass before merging**
  - Require branches to be up to date before merging
  - **Required status checks**:
    - `test / ubuntu-latest / Python 3.8`
    - `test / ubuntu-latest / Python 3.9`
    - `test / ubuntu-latest / Python 3.10`
    - `test / ubuntu-latest / Python 3.11`
    - `test / ubuntu-latest / Python 3.12`
    - `test / macos-latest / Python 3.8`
    - `test / macos-latest / Python 3.9`
    - `test / macos-latest / Python 3.10`
    - `test / macos-latest / Python 3.11`
    - `test / macos-latest / Python 3.12`
    - `test / windows-latest / Python 3.8`
    - `test / windows-latest / Python 3.9`
    - `test / windows-latest / Python 3.10`
    - `test / windows-latest / Python 3.11`
    - `test / windows-latest / Python 3.12`
    - `lint`

- ✅ **Require conversation resolution before merging**

- ✅ **Do not allow bypassing the above settings**
  - Applies to administrators

#### Optional Settings

- ⚠️ **Require signed commits** (recommended for releases)
- ⚠️ **Include administrators** (recommended but may block urgent fixes)

## Automated Configuration (GitHub CLI)

You can configure branch protection using the GitHub CLI:

```bash
# Install GitHub CLI if not already installed
# brew install gh  # macOS
# See: https://cli.github.com/

# Authenticate
gh auth login

# Enable branch protection
gh api repos/{owner}/contexta-core/branches/main/protection \
  --method PUT \
  --field required_status_checks[strict]=true \
  --field required_status_checks[contexts][]="test / ubuntu-latest / Python 3.8" \
  --field required_status_checks[contexts][]="test / ubuntu-latest / Python 3.9" \
  --field required_status_checks[contexts][]="test / ubuntu-latest / Python 3.10" \
  --field required_status_checks[contexts][]="test / ubuntu-latest / Python 3.11" \
  --field required_status_checks[contexts][]="test / ubuntu-latest / Python 3.12" \
  --field required_status_checks[contexts][]="test / macos-latest / Python 3.8" \
  --field required_status_checks[contexts][]="test / macos-latest / Python 3.9" \
  --field required_status_checks[contexts][]="test / macos-latest / Python 3.10" \
  --field required_status_checks[contexts][]="test / macos-latest / Python 3.11" \
  --field required_status_checks[contexts][]="test / macos-latest / Python 3.12" \
  --field required_status_checks[contexts][]="test / windows-latest / Python 3.8" \
  --field required_status_checks[contexts][]="test / windows-latest / Python 3.9" \
  --field required_status_checks[contexts][]="test / windows-latest / Python 3.10" \
  --field required_status_checks[contexts][]="test / windows-latest / Python 3.11" \
  --field required_status_checks[contexts][]="test / windows-latest / Python 3.12" \
  --field required_status_checks[contexts][]="lint" \
  --field required_pull_request_reviews[required_approving_review_count]=1 \
  --field required_pull_request_reviews[dismiss_stale_reviews]=true \
  --field enforce_admins=true \
  --field required_conversation_resolution=true
```

## Verification

After configuring, verify the settings:

```bash
gh api repos/{owner}/contexta-core/branches/main/protection | jq
```

You should see:
- `required_status_checks.strict: true`
- `required_status_checks.contexts` containing all CI job names
- `required_pull_request_reviews.required_approving_review_count: 1`
- `enforce_admins: true`

## References

- [GitHub Branch Protection Documentation](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/about-protected-branches)
- [GitHub REST API - Branch Protection](https://docs.github.com/en/rest/branches/branch-protection)
