#!/bin/bash
# Setup lightweight pre-commit hooks for Grape project
# This version only runs rustfmt and clippy (no tests)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
HOOKS_DIR="$PROJECT_ROOT/.git/hooks"

echo "🔧 Setting up lightweight Git pre-commit hooks for Grape..."

# Create hooks directory if it doesn't exist
mkdir -p "$HOOKS_DIR"

# Create pre-commit hook
cat > "$HOOKS_DIR/pre-commit" << 'EOF'
#!/bin/bash
# Lightweight pre-commit hook for Grape
# Runs only rustfmt and clippy (no tests)

set -e

echo "🔍 Running pre-commit checks (light mode)..."

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo "❌ cargo not found. Please install Rust."
    exit 1
fi

# Run rustfmt
echo "📝 Running rustfmt..."
if ! cargo fmt --all -- --check; then
    echo "❌ Code formatting issues detected!"
    echo "💡 Run 'cargo fmt --all' to fix formatting"
    exit 1
fi
echo "✅ Formatting check passed"

# Run clippy (only on staged files, fast mode)
echo "🔎 Running clippy..."
if ! cargo clippy --all-targets --all-features -- -D warnings 2>&1 | head -n 50; then
    echo "❌ Clippy found issues!"
    echo "💡 Fix the issues or use 'git commit --no-verify' to skip (not recommended)"
    exit 1
fi
echo "✅ Clippy check passed"

echo "✅ All pre-commit checks passed!"
echo "💡 Remember to run tests manually: cargo test"
EOF

# Make the hook executable
chmod +x "$HOOKS_DIR/pre-commit"

echo "✅ Lightweight pre-commit hook installed successfully!"
echo ""
echo "The hook will run on every commit and check:"
echo "  - Code formatting (rustfmt)"
echo "  - Linting (clippy)"
echo ""
echo "⚠️  Tests are NOT run automatically with this light version"
echo "💡 Run tests manually: cargo test"
echo ""
echo "To skip the hook temporarily, use: git commit --no-verify"
echo ""
