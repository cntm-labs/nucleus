#!/bin/sh
# Install git hooks for Nucleus development
set -e

HOOKS_DIR="$(git rev-parse --show-toplevel)/.git/hooks"
SCRIPTS_DIR="$(git rev-parse --show-toplevel)/scripts"

cp "$SCRIPTS_DIR/pre-commit" "$HOOKS_DIR/pre-commit"
chmod +x "$HOOKS_DIR/pre-commit"

echo "✅ Git hooks installed successfully!"
echo "   pre-commit: fmt + clippy + test + typecheck"
