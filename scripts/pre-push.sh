#!/bin/bash
# Pre-push hook - 在 git push 前运行
# 安装: cp scripts/pre-push.sh .git/hooks/pre-push && chmod +x .git/hooks/pre-push

set -e

echo "🚀 Running pre-push checks..."
echo ""

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 检查 cargo
check_cargo() {
    if ! command -v cargo &> /dev/null; then
        echo -e "${RED}❌ Error: cargo not found${NC}"
        exit 1
    fi
}

# 格式化检查
check_fmt() {
    echo "📐 Checking formatting..."
    if cargo fmt --all -- --check; then
        echo -e "${GREEN}✅ Formatting OK${NC}"
        return 0
    else
        echo -e "${RED}❌ Formatting issues found${NC}"
        echo "Run 'cargo fmt --all' to fix"
        return 1
    fi
}

# Clippy 检查
check_clippy() {
    echo "🔍 Running clippy..."
    if cargo clippy --all-targets --all-features -- -D warnings 2>&1; then
        echo -e "${GREEN}✅ Clippy OK${NC}"
        return 0
    else
        echo -e "${RED}❌ Clippy warnings found${NC}"
        return 1
    fi
}

# 测试
check_tests() {
    echo "🧪 Running tests..."
    if cargo test --all --quiet 2>&1; then
        echo -e "${GREEN}✅ Tests passed${NC}"
        return 0
    else
        echo -e "${RED}❌ Tests failed${NC}"
        return 1
    fi
}

# 编译检查
check_build() {
    echo "🔨 Checking build..."
    if cargo check --all 2>&1 | grep -q "error"; then
        echo -e "${RED}❌ Build failed${NC}"
        return 1
    else
        echo -e "${GREEN}✅ Build OK${NC}"
        return 0
    fi
}

# 文档检查
check_docs() {
    echo "📚 Checking documentation..."
    if RUSTDOCFLAGS="-D warnings" cargo doc --all --no-deps 2>&1 | grep -q "error"; then
        echo -e "${RED}❌ Documentation warnings found${NC}"
        return 1
    else
        echo -e "${GREEN}✅ Documentation OK${NC}"
        return 0
    fi
}

# 主流程
main() {
    check_cargo
    
    local failed=0
    
    check_fmt || failed=1
    echo ""
    
    check_build || failed=1
    echo ""
    
    check_clippy || failed=1
    echo ""
    
    check_tests || failed=1
    echo ""
    
    check_docs || failed=1
    echo ""
    
    if [ $failed -eq 0 ]; then
        echo -e "${GREEN}✨ All checks passed! Pushing...${NC}"
        exit 0
    else
        echo -e "${RED}❌ Some checks failed. Fix before pushing.${NC}"
        echo ""
        echo "You can bypass this hook with: git push --no-verify"
        echo -e "${YELLOW}⚠️  WARNING: Bypassing checks is not recommended!${NC}"
        exit 1
    fi
}

main "$@"
