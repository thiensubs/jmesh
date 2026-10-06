# Release Process

This document outlines the steps to release a new version of jmesh.

## Prerequisites

1. You need write access to the thiensubs/jmesh repository on GitHub
2. You need a crates.io account with permission to publish to the jmesh crate
3. You need to have set up the CARGO_REGISTRY_TOKEN secret in the GitHub repository

## Steps

### 1. Prepare the release

Ensure all changes are merged into main and the code is in a releasable state:

```bash
git checkout main
git pull
```

### 2. Update version and changelog

Determine the next version number following Semantic Versioning:
- MAJOR version when you make incompatible API changes
- MINOR version when you add functionality in a backward compatible manner
- PATCH version when you make backward compatible bug fixes

Update Cargo.toml with the new version:
```toml
[package]
version = "X.Y.Z"  # <-- update this
```

Update CHANGELOG.md:
- Change the "## [Unreleased]" header to "## [X.Y.Z] - YYYY-MM-DD"
- Add a new "## [Unreleased]" section at the top for future changes

Commit these changes:
```bash
git add Cargo.toml CHANGELOG.md
git commit -m "release: prepare version X.Y.Z"
```

### 3. Create and push tag

Create an annotated tag and push it:

```bash
git tag -a vX.Y.Z -m "Release version X.Y.Z"
git push origin main --tags
```

### 4. GitHub Actions workflow

Pushing the tag will trigger the release workflow (.github/workflows/release.yml) which will:

1. Verify the tag matches Cargo.toml version
2. Perform a dry-run publish to ensure the package is valid
3. Build binaries for multiple platforms:
   - Linux (glibc and musl)
   - macOS (Apple Silicon and Intel)
   - Windows
4. Create a GitHub Release with the built binaries
5. Publish the crate to crates.io

### 5. Post-release verification

After the workflow completes:

1. Check the GitHub Release page for the uploaded assets
2. Verify the crate is available on crates.io: https://crates.io/crates/jmesh
3. Test installation from crates.io:
   ```bash
   cargo install jmesh
   jmesh --version
   ```
4. Announce the release if desired

## Troubleshooting

### Publish fails
If the cargo publish step fails, you can try manually:
```bash
cargo login  # if needed
cargo publish
```

### Build failures
Check the workflow logs for build errors. Common issues:
- Missing dependencies for cross-compilation
- Feature flag incompatibilities
- Rust version requirements

### Tag already exists
If you accidentally reuse a tag:
```bash
git tag -d vX.Y.Z
git push origin :refs/tags/vX.Y.Z
```
Then recreate with the correct version.

## Automation

The release is fully automated via GitHub Actions. Maintainers only need to:
1. Ensure code is ready on main
2. Update version in Cargo.toml
3. Update CHANGELOG.md
4. Create and push the tag

Everything else (building, packaging, publishing) happens automatically.