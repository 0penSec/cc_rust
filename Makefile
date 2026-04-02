# Claude Code RS - Makefile

.PHONY: build test check fmt clippy clean run install dev

# 默认目标
all: check build test

# 构建
build:
	cargo build --release

# 开发构建
dev:
	cargo build

# 运行测试
test:
	cargo test --workspace

# 运行特定 crate 的测试
test-core:
	cargo test -p claude-core -- --nocapture

test-tools:
	cargo test -p claude-tools -- --nocapture

test-engine:
	cargo test -p claude-engine -- --nocapture

# 代码检查
check:
	cargo check --all-targets --all-features

# 格式化
fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

# Clippy 检查
clippy:
	cargo clippy --all-targets --all-features -- -D warnings

# 清理
clean:
	cargo clean

# 运行 CLI
run:
	cargo run --

# 交互式模式
chat:
	cargo run -- chat

# 安装到本地
install:
	cargo install --path crates/cli

# 文档
doc:
	cargo doc --no-deps --open

# 性能分析
bench:
	cargo bench

# 检查编译时间
timings:
	cargo build --release --timings

# 发布准备
release-check:
	cargo fmt --all -- --check
	cargo clippy --all-targets --all-features -- -D warnings
	cargo test --workspace
	cargo build --release

# Docker 构建
docker-build:
	docker build -t claude-code-rs .

# 更新依赖
update:
	cargo update

# 检查安全漏洞
audit:
	cargo audit
