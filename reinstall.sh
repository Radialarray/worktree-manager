#!/usr/bin/env bash
set -e

echo "🔧 Reinstalling wt from source..."
cargo install --path . --force

echo ""
echo "✓ wt reinstalled successfully!"
echo ""
echo "📝 Next steps:"
echo "  1. Update your shell integration:"
echo "     - Remove the line with '# wt shell integration' from your shell config"
echo "     - Run: wt init"
echo ""
echo "  2. Reload your shell:"
echo "     source ~/.zshrc    # for zsh"
echo "     source ~/.bashrc   # for bash"
echo "     exec fish          # for fish"
echo ""
echo "  Or just open a new terminal window!"
