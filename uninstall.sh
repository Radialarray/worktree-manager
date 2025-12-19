#!/bin/bash
# Uninstall script for wt (worktree-manager) on macOS

set -e

echo "🗑️  Uninstalling wt (worktree-manager)..."
echo ""

# Track what was removed
REMOVED_ITEMS=()

# 1. Remove binary
if command -v wt &> /dev/null; then
    BINARY_PATH=$(which wt)
    echo "Removing binary: $BINARY_PATH"
    rm -f "$BINARY_PATH"
    REMOVED_ITEMS+=("binary: $BINARY_PATH")
else
    echo "⚠️  wt binary not found in PATH"
fi

# 2. Remove Homebrew installation (if installed via Homebrew)
if brew list worktree-manager &> /dev/null 2>&1; then
    echo "Removing Homebrew installation..."
    brew uninstall worktree-manager
    REMOVED_ITEMS+=("Homebrew package")
fi

# 3. Remove configuration directory
CONFIG_DIR="$HOME/.config/worktree-manager"
if [ -d "$CONFIG_DIR" ]; then
    echo "Removing configuration: $CONFIG_DIR"
    read -p "⚠️  This will delete your config.yaml. Continue? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        rm -rf "$CONFIG_DIR"
        REMOVED_ITEMS+=("config directory: $CONFIG_DIR")
    else
        echo "Skipped config directory removal"
    fi
fi

# 4. Remove shell integration
echo ""
echo "Checking shell integration..."

remove_from_file() {
    local file=$1
    local pattern=$2
    
    if [ -f "$file" ]; then
        if grep -q "$pattern" "$file"; then
            echo "Found wt integration in $file"
            # Create backup
            cp "$file" "${file}.backup"
            # Remove lines containing the pattern
            sed -i '' "/$pattern/d" "$file"
            REMOVED_ITEMS+=("shell integration: $file")
            echo "✓ Removed from $file (backup saved as ${file}.backup)"
        fi
    fi
}

# Check common shell config files
remove_from_file "$HOME/.zshrc" "wt init"
remove_from_file "$HOME/.bashrc" "wt init"
remove_from_file "$HOME/.bash_profile" "wt init"
remove_from_file "$HOME/.config/fish/config.fish" "wt init"

# 5. Remove cargo installation artifacts
CARGO_BIN="$HOME/.cargo/bin/wt"
if [ -f "$CARGO_BIN" ]; then
    echo "Removing cargo binary: $CARGO_BIN"
    rm -f "$CARGO_BIN"
    REMOVED_ITEMS+=("cargo binary: $CARGO_BIN")
fi

# Summary
echo ""
echo "=========================================="
if [ ${#REMOVED_ITEMS[@]} -eq 0 ]; then
    echo "❌ No wt installation found"
else
    echo "✅ Uninstall complete!"
    echo ""
    echo "Removed items:"
    for item in "${REMOVED_ITEMS[@]}"; do
        echo "  - $item"
    done
fi
echo "=========================================="
echo ""
echo "⚠️  Please restart your shell or run:"
echo "    source ~/.zshrc    # or ~/.bashrc"
echo ""
