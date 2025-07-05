#!/bin/bash

# Exit on any error
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_step() {
    echo -e "${YELLOW}==> $1${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

# Check if cargo-readme is installed
if ! command -v cargo-readme &> /dev/null; then
    print_error "cargo-readme is not installed. Please install it with: cargo install cargo-readme"
    exit 1
fi

# Main execution
print_step "Running local CI checks..."

# Test Suite
print_step "Running tests..."
if cargo test --all-features --workspace; then
    print_success "Tests passed"
else
    print_error "Tests failed"
    exit 1
fi

# Rustfmt
print_step "Checking formatting..."
if cargo fmt --all --check; then
    print_success "Formatting check passed"
else
    print_error "Formatting check failed. Run 'cargo fmt' to fix."
    exit 1
fi

# Clippy
print_step "Running clippy..."
if cargo clippy --all-targets --all-features --workspace -- -D warnings; then
    print_success "Clippy check passed"
else
    print_error "Clippy check failed"
    exit 1
fi

# Documentation
print_step "Checking documentation..."
if RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --document-private-items --all-features --workspace --examples; then
    print_success "Documentation check passed"
else
    print_error "Documentation check failed"
    exit 1
fi

# Update README
print_step "Updating README.md..."
if cargo readme --no-license --no-title > README.md; then
    print_success "README.md updated successfully"
else
    print_error "Failed to update README.md"
    exit 1
fi

echo
print_success "All checks passed! 🎉"
echo
echo "Next steps:"
echo "  - Review the changes to README.md"
echo "  - Commit your changes if everything looks good"
echo "  - Push to trigger CI on GitHub"