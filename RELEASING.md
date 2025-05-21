# Release Automation and Versioning

Commitaura uses [cargo-release](https://github.com/crate-ci/cargo-release) to automate versioning, changelog generation, and publishing.

## 🚀 How to Release a New Version

1. **Install cargo-release** (one-time):
   ```sh
   cargo install cargo-release
   ```

2. **Release a new version:**
   - For a patch release:
     ```sh
     cargo release patch
     ```
   - For a minor release:
     ```sh
     cargo release minor
     ```
   - For a major release:
     ```sh
     cargo release major
     ```

   This will:
   - Bump the version in `Cargo.toml` and `Cargo.lock`
   - Generate or update `CHANGELOG.md`
   - Commit and tag the release
   - Push the tag to GitHub
   - Optionally publish to crates.io (if you confirm)

3. **GitHub Actions Release Automation:**
   - When you push a tag (e.g., `v1.2.3`), GitHub Actions will:
     - Build the release binary
     - Upload it as a GitHub Release asset
     - Publish to crates.io (if configured)

## Example Workflow

```sh
# Make sure your working directory is clean
cargo release minor
# Follow the prompts (edit changelog, confirm, etc.)
# Push the tag if not done automatically
# GitHub Actions will handle the rest!
```

## Configuration

You can customize release behavior with a `release.toml` file. See the [cargo-release docs](https://github.com/crate-ci/cargo-release) for advanced options.

---

For more details, see the main README.md and DEV_SETUP.md.
