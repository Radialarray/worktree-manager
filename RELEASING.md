# Release Process

This project uses [cargo-dist](https://opensource.axo.dev/cargo-dist/) for automated binary releases.

## Creating a Release

1. **Update version** in `Cargo.toml`:
   ```toml
   [package]
   version = "0.2.0"
   ```

2. **Commit the version change**:
   ```bash
   git add Cargo.toml
   git commit -m "chore: bump version to 0.2.0"
   git push
   ```

3. **Create and push a version tag**:
   ```bash
   git tag v0.2.0
   git push origin v0.2.0
   ```

4. **GitHub Actions will automatically**:
   - Build binaries for all platforms (macOS, Linux, Windows)
   - Create a GitHub Release
   - Upload pre-built binaries
   - Generate shell installer script
   - Generate Homebrew formula
   - Calculate SHA256 checksums

## Installation Methods (After Release)

### Shell Installer (Recommended)
```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/Radialarray/worktree-manager/releases/latest/download/worktree-manager-installer.sh | sh
```

### Homebrew (After tap setup)
```bash
# Once Homebrew tap is configured
brew install radialarray/tap/worktree-manager
```

### Manual Download
Download pre-built binaries from:
https://github.com/Radialarray/worktree-manager/releases

## Supported Platforms

- **macOS**: x86_64 (Intel), aarch64 (Apple Silicon)
- **Linux**: x86_64, aarch64
- **Windows**: x86_64

## Build Configuration

The release builds use the `[profile.dist]` in `Cargo.toml`:
- Inherits from `release` profile
- Uses thin LTO for smaller binaries
- Optimized for size and performance

## Testing Before Release

Test the release build locally:
```bash
cargo install cargo-dist
cargo dist build --artifacts=global
```

Check generated artifacts in `target/distrib/`.
