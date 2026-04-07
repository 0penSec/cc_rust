.PHONY: all build test check fmt lint clean doc install pre-commit help

# 默认目标
all: check build test

# 构建
build:
	cargo build --all

build-release:
	cargo build --release --all

# 测试
test:
	cargo test --all

test-lib:
	cargo test --lib --all

test-integration:
	cargo test --test '*' --all

# 代码检查
check: fmt lint

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

lint:
	cargo clippy --all-targets --all-features -- -D warnings

# 文档
doc:
	cargo doc --all --no-deps

doc-open:
	cargo doc --all --open

# 清理
clean:
	cargo clean

# 安装本地
install:
	cargo install --path crates/cli --force

# 预提交检查
pre-commit:
	./scripts/pre-commit.sh

# 代码覆盖率
coverage:
	cargo tarpaulin --all --out Html --output-dir ./coverage

# 安全检查
audit:
	cargo audit

# 发布检查
release-check:
	@echo "Checking release readiness..."
	cargo fmt --all -- --check
	cargo clippy --all-targets --all-features -- -D warnings
	cargo test --all
	cargo build --release --all
	@echo "✅ Ready for release!"

# 帮助
help:
	@echo "可用目标:"
	@echo "  make build          - 构建项目"
	@echo "  make build-release  - 发布构建"
	@echo "  make test           - 运行所有测试"
	@echo "  make check          - 运行代码检查 (fmt + lint)"
	@echo "  make fmt            - 格式化代码"
	@echo "  make lint           - 运行 Clippy"
	@echo "  make doc            - 生成文档"
	@echo "  make pre-commit     - 运行预提交检查"
	@echo "  make coverage       - 生成覆盖率报告"
	@echo "  make audit          - 安全审计"
	@echo "  make clean          - 清理构建产物"
