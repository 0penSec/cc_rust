#!/bin/bash
set -e

echo "🔍 Running pre-commit checks..."

echo "1️⃣  Formatting check..."
cargo fmt --all -- --check
echo "   ✅ Formatting OK"

echo "2️⃣  Clippy check..."
cargo clippy --all-targets --all-features -- -D warnings
echo "   ✅ Clippy OK"

echo "3️⃣  Running tests..."
cargo test --all
echo "   ✅ Tests passed"

echo "4️⃣  Building release..."
cargo build --release --all
echo "   ✅ Release build OK"

echo "5️⃣  Checking documentation..."
cargo doc --all --no-deps
echo "   ✅ Documentation OK"

echo ""
echo "✨ All checks passed! Ready to commit."
