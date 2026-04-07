# 贡献指南

感谢您对 Claude Code RS 的兴趣！本文档将帮助您快速开始贡献代码。

## 🚀 快速开始

### 1. 克隆仓库

```bash
git clone https://github.com/yourusername/claude-code-rs.git
cd claude-code-rs
```

### 2. 安装依赖

```bash
# 安装 Rust (如果还没有)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装必要工具
cargo install cargo-audit cargo-tarpaulin
```

### 3. 构建项目

```bash
cargo build --all
```

### 4. 运行测试

```bash
cargo test --all
```

## 📝 开发流程

### 1. 创建分支

```bash
git checkout -b feature/your-feature-name
```

### 2. 编写代码

- 遵循 [Rust API 指南](https://rust-lang.github.io/api-guidelines/)
- 使用 `cargo fmt` 格式化代码
- 使用 `cargo clippy` 检查代码

### 3. 添加测试

```bash
# 运行测试确保通过
cargo test --all

# 检查代码覆盖率
cargo tarpaulin --all
```

### 4. 预提交检查

```bash
# 运行预提交脚本
./scripts/pre-commit.sh
```

或使用 Makefile:

```bash
make pre-commit
```

### 5. 提交代码

```bash
git add .
git commit -m "feat: 添加新功能"
git push origin feature/your-feature-name
```

## 🔄 提交信息规范

我们使用 [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(scope): <subject>

[optional body]

[optional footer]
```

### 类型

- `feat`: 新功能
- `fix`: Bug 修复
- `docs`: 文档更新
- `style`: 代码格式（不影响功能）
- `refactor`: 重构
- `perf`: 性能优化
- `test`: 测试
- `chore`: 构建/工具

### 示例

```
feat(engine): 添加流式响应支持

实现了 SSE 流式解析，支持实时显示 AI 回复。

Closes #123
```

## 🧪 测试指南

### 单元测试

放在被测试代码的 `#[cfg(test)]` 模块中:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_feature() {
        // 准备数据
        let input = ...;
        
        // 执行操作
        let result = function(input).await;
        
        // 验证结果
        assert!(result.is_ok());
    }
}
```

### 集成测试

放在 `tests/` 目录:

```rust
// tests/integration_test.rs
use claude_engine::QueryEngine;

#[tokio::test]
async fn test_end_to_end() {
    // 测试完整流程
}
```

### 运行测试

```bash
# 所有测试
cargo test --all

# 特定 crate
cargo test --package claude-engine

# 特定测试
cargo test test_name -- --nocapture

# 显示输出
cargo test -- --nocapture
```

## 🔍 代码审查流程

1. **创建 PR**: 推送到 GitHub 并创建 Pull Request
2. **CI 检查**: 等待 GitHub Actions 检查通过
3. **代码审查**: 等待维护者审查
4. **修复反馈**: 根据评论修改代码
5. **合并**: 获得批准后合并到 main

## 🛠️ 常用命令

```bash
# 格式化代码
cargo fmt --all

# 静态分析
cargo clippy --all-targets --all-features -- -D warnings

# 生成文档
cargo doc --all --open

# 检查依赖
cargo audit

# 性能分析
cargo bench

# 构建发布版本
cargo build --release
```

或使用 Makefile:

```bash
make build      # 构建
make test       # 测试
make check      # 检查
make fmt        # 格式化
make lint       # Clippy
make doc        # 文档
make pre-commit # 预提交检查
```

## 📋 代码规范

### 格式化

项目使用 `rustfmt`，配置在 `.rustfmt.toml`:

```bash
cargo fmt --all
```

### Clippy

项目使用严格模式:

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

### 文档

- 公共 API 必须有文档注释
- 使用 `///` 添加文档
- 包含示例代码

```rust
/// 提交消息到 QueryEngine
///
/// # 示例
///
/// ```rust,no_run
/// let engine = QueryEngine::new(...)?;
/// let stream = engine.submit_message("Hello");
/// ```
pub fn submit_message(...) { ... }
```

## 🐛 调试技巧

### 日志输出

```bash
# 启用日志
RUST_LOG=debug cargo run

# 特定模块
RUST_LOG=claude_engine=trace cargo run
```

### 调试测试

```bash
# 单线程运行
cargo test --test test_name -- --nocapture --test-threads=1

# 使用 debugger
cargo test --no-run  # 编译但不运行
gdb target/debug/deps/test_name
```

## 🆘 获取帮助

- 查看 [文档](./ARCHITECTURE.md)
- 查看 [API 文档](../target/doc/claude_engine/index.html)
- 在 GitHub Issues 中提问

## 📝 许可

通过贡献代码，您同意您的贡献将在 MIT 许可下发布。
