#!/bin/bash
# Setup pre-commit hooks for Grape project

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
HOOKS_DIR="$PROJECT_ROOT/.git/hooks"

echo "🔧 Setting up Git pre-commit hooks for Grape..."

# Create hooks directory if it doesn't exist
mkdir -p "$HOOKS_DIR"

# Create pre-commit hook
cat > "$HOOKS_DIR/pre-commit" << 'EOF'
#!/bin/bash
# Pre-commit hook for Grape
# Runs rustfmt and clippy before allowing commit

set -e

echo "🔍 Running pre-commit checks..."

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

# Run clippy (only on staged files)
echo "🔎 Running clippy..."
if ! cargo clippy --all-targets --all-features -- -D warnings; then
    echo "❌ Clippy found issues!"
    echo "💡 Fix the issues or use 'git commit --no-verify' to skip (not recommended)"
    exit 1
fi
echo "✅ Clippy check passed"

# Run tests
echo "🧪 Running tests..."
if ! cargo test --all-features; then
    echo "❌ Tests failed!"
    echo "💡 Fix the failing tests or use 'git commit --no-verify' to skip (not recommended)"
    exit 1
fi
echo "✅ Tests passed"

echo "✅ All pre-commit checks passed!"
EOF

# Make the hook executable
chmod +x "$HOOKS_DIR/pre-commit"

echo "✅ Pre-commit hook installed successfully!"
echo ""
echo "The hook will run on every commit and check:"
echo "  - Code formatting (rustfmt)"
echo "  - Linting (clippy)"
echo "  - Tests (cargo test)"
echo ""
echo "To skip the hook temporarily, use: git commit --no-verify"
echo ""
