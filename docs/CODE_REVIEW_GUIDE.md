# Claude Code RS 代码审查指南

## 开发工作流程

```
1. 编写代码 -> 2. 本地测试 -> 3. 静态分析 -> 4. PR 创建 -> 5. CI 检查 -> 6. Code Review -> 7. 合并
```

---

## 一、本地开发检查（提交前必须完成）

### 1.1 代码格式化

```bash
# 自动格式化代码
cargo fmt --all

# 检查格式（CI 使用）
cargo fmt --all -- --check
```

**配置** (`.rustfmt.toml`):
```toml
edition = "2021"
max_width = 100
tab_spaces = 4
use_small_heuristics = "Default"
reorder_imports = true
reorder_modules = true
```

### 1.2 静态分析 (Clippy)

```bash
# 基础检查
cargo clippy --all-targets --all-features

# 严格模式（推荐）
cargo clippy --all-targets --all-features -- -D warnings

# 检查特定 crate
cargo clippy --package claude-engine -- -D warnings
```

**必修复的 Lint**:
- `error` 级别：必须修复
- `warning` 级别：建议修复
- `nursery` 级别：选择性采纳

常用允许列表（如有需要）:
```rust
#![allow(clippy::too_many_arguments)]  // 构造函数参数多
#![allow(clippy::type_complexity)]      // 复杂类型暂时允许
```

### 1.3 编译检查

```bash
# 调试构建
cargo build --all

# 发布构建（优化检查）
cargo build --release --all

# 检查所有目标
cargo check --all-targets --all-features
```

### 1.4 测试运行

```bash
# 运行所有测试
cargo test --all

# 运行特定 crate 测试
cargo test --package claude-engine

# 运行特定测试
cargo test test_query_engine_basic

# 显示输出
cargo test -- --nocapture

# 并发测试（默认）
cargo test --all --jobs 4
```

---

## 二、单元测试编写规范

### 2.1 测试组织结构

```rust
// crates/engine/src/query_engine.rs

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    // ===== 基础测试 =====
    #[tokio::test]
    async fn test_query_engine_creation() {
        let config = QueryEngineConfig::default();
        let client_config = ClientConfig {
            api_key: "test-key".to_string(),
            ..Default::default()
        };
        
        let engine = QueryEngine::new(client_config, config);
        assert!(engine.is_ok());
    }

    // ===== 错误处理测试 =====
    #[tokio::test]
    async fn test_invalid_api_key() {
        // 使用 mock 或测试密钥验证错误处理
    }

    // ===== 边界条件测试 =====
    #[tokio::test]
    async fn test_max_turns_exceeded() {
        // 测试最大轮次限制
    }

    // ===== 并发测试 =====
    #[tokio::test]
    async fn test_concurrent_tool_execution() {
        // 测试工具并行执行
    }
}
```

### 2.2 测试覆盖率

```bash
# 安装 tarpaulin
cargo install cargo-tarpaulin

# 生成覆盖率报告
cargo tarpaulin --all --out Html --output-dir ./coverage

# 查看行覆盖率
cargo tarpaulin --all --verbose
```

**目标覆盖率**:
- 核心业务逻辑: > 80%
- 工具实现: > 70%
- 错误处理: > 90%

### 2.3 Mock 和 Stub

使用 `mockall` 进行接口模拟:

```rust
// crates/engine/src/client.rs

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait ApiClient {
    async fn stream_messages(&self, request: MessagesRequest) -> ClaudeResult<MessageStream>;
}

#[tokio::test]
async fn test_with_mock_client() {
    let mut mock = MockApiClient::new();
    mock.expect_stream_messages()
        .returning(|_| Ok(MessageStream::empty()));
    
    // 使用 mock 测试 QueryEngine
}
```

---

## 三、集成测试

### 3.1 测试结构

```
tests/
├── common/
│   ├── mod.rs          # 共享测试工具
│   └── fixtures.rs     # 测试数据
├── integration_test.rs # 端到端测试
├── cli_test.rs         # CLI 测试
└── tool_test.rs        # 工具集成测试
```

### 3.2 集成测试示例

```rust
// tests/query_engine_integration.rs

use claude_engine::{QueryEngine, QueryEngineConfig};
use std::time::Duration;

#[tokio::test]
async fn test_end_to_end_conversation() {
    // 设置测试环境
    let client_config = create_test_client_config();
    let engine_config = QueryEngineConfig {
        max_turns: 5,
        ..Default::default()
    };
    
    let engine = QueryEngine::new(client_config, engine_config).unwrap();
    
    // 执行对话
    let mut stream = engine.submit_message("Hello");
    let mut events = vec![];
    
    while let Some(event) = stream.next().await {
        events.push(event);
    }
    
    // 验证结果
    assert!(!events.is_empty());
    assert!(events.iter().any(|e| matches!(e, QueryEvent::Complete { .. })));
}

fn create_test_client_config() -> ClientConfig {
    ClientConfig {
        api_key: std::env::var("TEST_API_KEY")
            .unwrap_or_else(|_| "sk-test".to_string()),
        api_base: "https://api.anthropic.com".to_string(),
        ..Default::default()
    }
}
```

### 3.3 使用 wiremock 进行 HTTP 模拟

```rust
// tests/api_mock_test.rs

use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};

#[tokio::test]
async fn test_api_retry_logic() {
    let mock_server = MockServer::start().await;
    
    // 模拟失败响应
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(500))
        .up_to_n_times(2)
        .mount(&mock_server)
        .await;
    
    // 模拟成功响应
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(serde_json::json!({
                "id": "msg_123",
                "type": "message",
                "role": "assistant",
                "content": [{"type": "text", "text": "Hi"}],
                "model": "claude-sonnet-4-6",
                "stop_reason": "end_turn",
                "usage": {"input_tokens": 10, "output_tokens": 5}
            })))
        .mount(&mock_server)
        .await;
    
    // 测试重试逻辑
    let config = ClientConfig {
        api_base: mock_server.uri(),
        max_retries: 3,
        ..Default::default()
    };
    
    // 执行测试...
}
```

---

## 四、文档检查

### 4.1 文档测试

```bash
# 测试文档中的代码示例
cargo test --doc --all

# 生成文档
cargo doc --all --no-deps

# 打开文档
cargo doc --all --open
```

### 4.2 文档注释规范

```rust
/// 提交消息到 QueryEngine 并获取事件流
///
/// # Arguments
///
/// * `prompt` - 用户输入的提示文本
///
/// # Returns
///
/// 返回一个 `Stream`，产生 `QueryEvent` 事件
///
/// # Examples
///
/// ```rust,no_run
/// use claude_engine::{QueryEngine, QueryEngineConfig};
///
/// # async fn example() -> anyhow::Result<()> {
/// let engine = QueryEngine::new(Default::default(), Default::default())?;
/// let mut stream = engine.submit_message("Hello");
///
/// while let Some(event) = stream.next().await {
///     println!("{:?}", event);
/// }
/// # Ok(())
/// # }
/// ```
pub fn submit_message(&self, prompt: impl Into<String>) -> impl Stream<Item = QueryEvent> + '_ {
    // ...
}
```

---

## 五、性能测试

### 5.1 基准测试

```rust
// crates/engine/benches/query_benchmark.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use claude_engine::Conversation;

fn conversation_benchmark(c: &mut Criterion) {
    c.bench_function("add 100 messages", |b| {
        b.iter(|| {
            let mut conv = Conversation::builder().build();
            for i in 0..100 {
                conv.add_user_message(format!("Message {}", i));
            }
            black_box(conv);
        });
    });
}

criterion_group!(benches, conversation_benchmark);
criterion_main!(benches);
```

运行:
```bash
cargo bench
```

### 5.2 性能分析

```bash
# CPU 分析
cargo install flamegraph
cargo flamegraph --bin claude

# 内存检查
cargo install cargo-valgrind
cargo valgrind --bin claude
```

---

## 六、安全检查

### 6.1 依赖审计

```bash
# 安装 cargo-audit
cargo install cargo-audit

# 检查已知漏洞
cargo audit

# 检查许可证
cargo install cargo-deny
cargo deny check
```

### 6.2 不安全代码检查

```bash
# 检查 unsafe 代码
cargo geiger

# Miri 检查（未定义行为）
cargo miri test
```

---

## 七、GitHub Actions CI 配置

### 7.1 完整 CI 配置

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main, develop ]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  # ===== 基础检查 =====
  check:
    name: Check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      
      - name: Cache cargo
        uses: Swatinem/rust-cache@v2
      
      - name: Check formatting
        run: cargo fmt --all -- --check
      
      - name: Run clippy
        run: cargo clippy --all-targets --all-features -- -D warnings
      
      - name: Check build
        run: cargo check --all

  # ===== 测试矩阵 =====
  test:
    name: Test
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable, beta]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust ${{ matrix.rust }}
        uses: dtolnay/rust-toolchain@master
        with:
          toolchain: ${{ matrix.rust }}
      
      - name: Cache cargo
        uses: Swatinem/rust-cache@v2
      
      - name: Build
        run: cargo build --all --verbose
      
      - name: Run tests
        run: cargo test --all --verbose
        env:
          RUST_TEST_THREADS: 4

  # ===== 代码覆盖率 =====
  coverage:
    name: Code Coverage
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
      
      - name: Install tarpaulin
        run: cargo install cargo-tarpaulin
      
      - name: Generate coverage
        run: cargo tarpaulin --all --out Xml
      
      - name: Upload to Codecov
        uses: codecov/codecov-action@v3
        with:
          files: ./cobertura.xml
          fail_ci_if_error: false

  # ===== 安全检查 =====
  security:
    name: Security Audit
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
      
      - name: Install cargo-audit
        run: cargo install cargo-audit
      
      - name: Run audit
        run: cargo audit
      
      - name: Check dependencies
        run: |
          cargo tree --duplicates  # 检查重复依赖
          cargo outdated           # 检查过时依赖 (需安装)

  # ===== 文档检查 =====
  docs:
    name: Documentation
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
      
      - name: Build docs
        run: cargo doc --all --no-deps
      
      - name: Check doc tests
        run: cargo test --doc --all

  # ===== 发布构建测试 =====
  build-release:
    name: Release Build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
      
      - name: Build release
        run: cargo build --release --all
      
      - name: Check binary size
        run: ls -lh target/release/claude
```

### 7.2 PR 检查清单模板

```markdown
<!-- .github/pull_request_template.md -->

## 描述
<!-- 描述这个 PR 的改动 -->

## 检查清单

### 代码质量
- [ ] 代码已格式化 (`cargo fmt`)
- [ ] Clippy 无警告 (`cargo clippy`)
- [ ] 所有测试通过 (`cargo test`)
- [ ] 新增功能有对应测试
- [ ] 文档注释完整

### 安全
- [ ] 无 `unwrap()` 或 `expect()` 用于用户输入
- [ ] 敏感数据（API key）已正确处理
- [ ] 依赖项已审计 (`cargo audit`)

### 性能
- [ ] 无明显的性能回归
- [ ] 大数据集已测试（如有）

### 兼容性
- [ ] 向后兼容（如适用）
- [ ] 新功能已记录在 CHANGELOG.md

## 测试方式
<!-- 描述如何测试这些改动 -->
```

---

## 八、代码审查检查表

### 8.1 审查者检查表

```markdown
## 代码审查检查表

### 正确性
- [ ] 逻辑正确，边界条件处理完善
- [ ] 错误处理完整，使用正确的错误类型
- [ ] 无竞态条件（并发代码）
- [ ] 资源正确释放（Drop 实现）

### 代码风格
- [ ] 命名符合 Rust 惯例 (snake_case, CamelCase)
- [ ] 函数长度合理（< 50 行）
- [ ] 模块组织清晰
- [ ] 导入分组合理（std, external, crate）

### 性能
- [ ] 无不必要的克隆
- [ ] 使用正确的集合类型（Vec vs HashMap）
- [ ] 异步代码使用正确的运行时
- [ ] 锁使用合理（避免死锁）

### 可维护性
- [ ] 文档清晰完整
- [ ] 测试覆盖关键路径
- [ ] 魔法数字有常量定义
- [ ] 复杂逻辑有注释

### 安全性
- [ ] 无 unsafe 代码（或有充分理由）
- [ ] 用户输入已验证
- [ ] 敏感数据未泄露
```

### 8.2 常见代码问题清单

| 问题 | 示例 | 建议 |
|------|------|------|
| 过度使用 unwrap | `file.read().unwrap()` | 使用 `?` 或 `match` |
| 不必要的 clone | `data.clone()` | 使用引用或 `Arc` |
| 阻塞操作在 async | `std::fs::read` | 使用 `tokio::fs::read` |
| 大类型传值 | `fn process(data: Vec<u8>)` | `fn process(data: &[u8])` |
| 忽略 Result | `file.write(buf);` | `file.write(buf).await?;` |
| 不必要 mut | `let mut x = 5;` | `let x = 5;` |
| 冗长匹配 | `match opt { Some(v) => ..., None => None }` | `opt.map(\|v\| ...)` |

---

## 九、提交前最终检查脚本

创建 `scripts/pre-commit.sh`:

```bash
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

echo "6️⃣  Security audit..."
cargo audit || echo "   ⚠️  Audit skipped (cargo-audit not installed)"

echo ""
echo "✨ All checks passed! Ready to commit."
```

配置 git hook:

```bash
chmod +x scripts/pre-commit.sh
ln -s ../../scripts/pre-commit.sh .git/hooks/pre-commit
```

---

## 十、发布流程

### 版本发布检查表

```markdown
## 发布检查表

### 准备工作
- [ ] 更新版本号 (Cargo.toml)
- [ ] 更新 CHANGELOG.md
- [ ] 所有测试通过
- [ ] 文档已更新

### 构建
- [ ] Linux x86_64 构建
- [ ] macOS x86_64 构建
- [ ] macOS ARM64 构建
- [ ] Windows x86_64 构建

### 验证
- [ ] 二进制可执行
- [ ] 基本功能测试
- [ ] 安装脚本测试

### 发布
- [ ] Git tag 创建
- [ ] GitHub Release 创建
- [ ] Release notes 编写
- [ ] 二进制上传到 Release
```

---

## 总结

每次提交前必须运行:

```bash
# 快速检查（< 30 秒）
cargo fmt && cargo check && cargo test --lib

# 完整检查（< 5 分钟）
./scripts/pre-commit.sh

# PR 提交前
./scripts/pre-commit.sh && cargo tarpaulin --all
```
