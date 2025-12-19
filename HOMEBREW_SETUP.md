# Homebrew Distribution Guide

**Issue ID:** `worktree-manager-d66`

This guide contains everything needed to set up Homebrew formula distribution for `wt`.

## Prerequisites

✅ **Completed:**
- Binary releases via GitHub Actions (worktree-manager-4mv)
- cargo-dist configured with `installers = ["shell", "homebrew"]`
- GitHub release workflow at `.github/workflows/release.yml`

## Overview

cargo-dist automatically generates a Homebrew formula (`worktree-manager.rb`) as part of each release. We need to:
1. Create a Homebrew tap repository
2. Set up auto-publishing to the tap

## Option 1: Create Custom Homebrew Tap (Recommended)

### Step 1: Create Tap Repository

Create a new GitHub repository: `homebrew-tap`
- Repository URL: `https://github.com/Radialarray/homebrew-tap`
- Initialize with README
- Make it public

### Step 2: Configure cargo-dist to publish to tap

Add to `dist-workspace.toml`:

```toml
[dist]
# ... existing config ...
installers = ["shell", "homebrew"]

# Publish Homebrew formula to custom tap
[dist.tap]
tap = "Radialarray/homebrew-tap"
# Optional: customize formula name if needed
# name = "wt"
```

### Step 3: Add GitHub Token for Publishing

The release workflow needs permission to push to your tap repository.

1. **Create a Personal Access Token (Classic)**:
   - Go to: https://github.com/settings/tokens
   - Click "Generate new token (classic)"
   - Scopes needed:
     - `repo` (full control of private repositories)
   - Expiration: Set as desired (recommend 1 year)
   - Generate token and copy it

2. **Add token as repository secret**:
   - Go to: https://github.com/Radialarray/worktree-manager/settings/secrets/actions
   - Click "New repository secret"
   - Name: `HOMEBREW_TAP_TOKEN`
   - Value: (paste your token)
   - Add secret

3. **Update release workflow** (`.github/workflows/release.yml`):
   
   Find the `publish` job and add:
   ```yaml
   publish:
     needs: [plan, host, upload-artifacts]
     runs-on: "ubuntu-22.04"
     env:
       GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
       HOMEBREW_TAP_TOKEN: ${{ secrets.HOMEBREW_TAP_TOKEN }}  # Add this line
   ```

### Step 4: Test the Setup

1. **Create a test release**:
   ```bash
   # Update version in Cargo.toml
   git add Cargo.toml
   git commit -m "chore: bump version to 0.1.1"
   git tag v0.1.1
   git push && git push --tags
   ```

2. **Verify**:
   - GitHub Actions completes successfully
   - Formula appears in `homebrew-tap` repository
   - Formula has correct SHA256 hashes for all platforms

### Step 5: Usage

Users can now install via:
```bash
brew tap radialarray/tap
brew install worktree-manager

# Or in one line:
brew install radialarray/tap/worktree-manager
```

---

## Option 2: Submit to Homebrew Core (Long-term Goal)

For wider distribution, submit to official Homebrew:

### Requirements for homebrew-core:
- Stable release (not 0.x version)
- Established user base
- Active maintenance
- No external dependencies beyond what Homebrew provides

### Steps:
1. Wait until version 1.0.0 or higher
2. Build community adoption
3. Fork https://github.com/Homebrew/homebrew-core
4. Add formula to `Formula/w/worktree-manager.rb`
5. Submit PR following: https://docs.brew.sh/How-To-Open-a-Homebrew-Pull-Request

---

## Post-Installation Message

cargo-dist automatically includes this in the formula:

```ruby
def caveats
  <<~EOS
    To enable shell integration (cd and editor actions):
      echo 'eval "$(wt init)"' >> ~/.zshrc
      # or for bash: echo 'eval "$(wt init)"' >> ~/.bashrc
  EOS
end
```

This is generated from `Cargo.toml` metadata and shown after `brew install`.

---

## Testing on Different Architectures

### Test on Apple Silicon:
```bash
brew install --build-from-source radialarray/tap/worktree-manager
wt --version
```

### Test on Intel Mac:
Same command on Intel hardware

### Verify both architectures work:
```bash
file $(which wt)
# Should show correct architecture
```

---

## Troubleshooting

### Formula not updating after release:
```bash
brew update
brew upgrade worktree-manager
```

### SHA256 mismatch errors:
- cargo-dist calculates these automatically
- If mismatch occurs, re-run release or regenerate manually:
  ```bash
  cargo dist build --artifacts=global
  ```

### Binary not found errors:
- Ensure binary name matches in Cargo.toml: `name = "wt"`
- Check formula `bin.install` path

---

## Checklist

- [ ] Create `homebrew-tap` repository
- [ ] Generate Personal Access Token
- [ ] Add `HOMEBREW_TAP_TOKEN` secret to worktree-manager repo
- [ ] Configure `dist.tap` in `dist-workspace.toml`
- [ ] Regenerate workflow: `cargo dist generate`
- [ ] Commit and push changes
- [ ] Create test release (v0.1.1 or similar)
- [ ] Verify formula appears in tap repository
- [ ] Test installation: `brew install radialarray/tap/worktree-manager`
- [ ] Test on Intel Mac (if available)
- [ ] Test on Apple Silicon Mac
- [ ] Update README.md with Homebrew installation instructions
- [ ] Close issue worktree-manager-d66

---

## References

- [cargo-dist Homebrew docs](https://opensource.axo.dev/cargo-dist/book/installers/homebrew.html)
- [Homebrew Formula Cookbook](https://docs.brew.sh/Formula-Cookbook)
- [Creating a Homebrew Tap](https://docs.brew.sh/How-to-Create-and-Maintain-a-Tap)
