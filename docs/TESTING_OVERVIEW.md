# 测试和代码审查概览

## 📊 代码质量工具链

```
代码编写
    │
    ▼
┌─────────────────┐
│  cargo fmt      │  ◀── 自动格式化
└─────────────────┘
    │
    ▼
┌─────────────────┐
│  cargo clippy   │  ◀── 静态分析
└─────────────────┘
    │
    ▼
┌─────────────────┐
│  cargo test     │  ◀── 单元/集成测试
└─────────────────┘
    │
    ▼
┌─────────────────┐
│  cargo doc      │  ◀── 文档测试
└─────────────────┘
    │
    ▼
┌─────────────────┐
│  cargo audit    │  ◀── 安全检查
└─────────────────┘
    │
    ▼
提交到 GitHub
```

## 🔄 CI/CD 流程

### Pull Request 流程

1. **开发者推送代码**
   ```bash
   git push origin feature/my-feature
   ```

2. **GitHub Actions 自动触发**
   - ✅ 代码格式化检查
   - ✅ Clippy 静态分析
   - ✅ 单元测试（Linux/macOS/Windows）
   - ✅ 集成测试
   - ✅ 代码覆盖率
   - ✅ 安全审计
   - ✅ 文档构建

3. **代码审查**
   - 维护者审查代码
   - 检查是否通过所有 CI
   - 验证测试覆盖

4. **合并到 main**
   - 获得批准后合并
   - 自动删除分支

## 🛠️ 本地开发工具

### 快速命令

| 命令 | 说明 | 时间 |
|------|------|------|
| `cargo fmt` | 格式化代码 | < 1s |
| `cargo check` | 编译检查 | < 10s |
| `cargo test --lib` | 运行单元测试 | < 30s |
| `make pre-commit` | 完整预提交检查 | < 5min |
| `cargo tarpaulin` | 生成覆盖率 | < 5min |

### 使用 Makefile

```bash
# 最常用的命令
make build      # 构建项目
make test       # 运行所有测试
make check      # 格式化 + Clippy
make pre-commit # 提交前完整检查

# 其他命令
make fmt        # 仅格式化
make lint       # 仅 Clippy
make doc        # 生成文档
make clean      # 清理
make coverage   # 覆盖率报告
```

## 📝 测试策略

### 1. 单元测试

位置：`src/` 文件内的 `#[cfg(test)]` 模块

覆盖目标：
- 核心业务逻辑 > 80%
- 工具实现 > 70%
- 错误处理 > 90%

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_feature() {
        // given - 准备数据
        let input = ...;
        
        // when - 执行操作
        let result = function(input).await;
        
        // then - 验证结果
        assert!(result.is_ok());
    }
}
```

### 2. 集成测试

位置：`tests/` 目录

覆盖目标：
- 端到端流程
- API 集成
- 工具链集成

```rust
// tests/integration_test.rs
#[tokio::test]
async fn test_end_to_end() {
    let engine = setup_engine().await;
    let result = engine.process("Hello").await;
    assert!(result.is_ok());
}
```

### 3. 文档测试

代码示例自动测试：

```rust
/// # 示例
///
/// ```rust
/// let result = add(1, 2);
/// assert_eq!(result, 3);
/// ```
pub fn add(a: i32, b: i32) -> i32 { a + b }
```

## 🔍 代码审查要点

### 自动检查（CI）

| 检查项 | 工具 | 要求 |
|--------|------|------|
| 格式化 | rustfmt | 必须通过 |
| 静态分析 | clippy | 无警告 |
| 单元测试 | cargo test | 100% 通过 |
| 覆盖率 | tarpaulin | > 70% |
| 安全审计 | cargo audit | 无高危漏洞 |
| 文档 | cargo doc | 无警告 |

### 人工审查

- **正确性**：逻辑正确、边界处理
- **性能**：无性能回归
- **安全**：无安全问题
- **可维护性**：代码清晰、文档完整

## 📈 代码质量指标

### 当前目标

```yaml
代码覆盖率:
  核心逻辑: > 80%
  工具实现: > 70%
  错误处理: > 90%

代码规范:
  格式化: 100%
  Clippy: 0 warnings
  文档: 100% 公共 API

安全检查:
  audit: 0 高危漏洞
  unsafe: 最小化
```

### 查看报告

```bash
# 覆盖率报告（HTML）
cargo tarpaulin --all --out Html --output-dir ./coverage
open coverage/tarpaulin-report.html

# 依赖树
cargo tree
cargo tree --duplicates  # 检查重复依赖

# 代码统计
tokei .
```

## 🚨 常见问题

### CI 失败

| 问题 | 解决方案 |
|------|----------|
| 格式化失败 | 运行 `cargo fmt --all` |
| Clippy 警告 | 修复警告或 `#![allow(...)]` |
| 测试失败 | 本地运行 `cargo test --all` 调试 |
| 覆盖率下降 | 添加更多测试 |

### 本地测试失败

```bash
# 清理重新构建
cargo clean
cargo build --all

# 更新依赖
cargo update

# 检查 Rust 版本
rustc --version  # 需要 >= 1.80
```

## 📚 相关文档

- [API 文档](../target/doc/claude_engine/index.html)
- [架构说明](./ARCHITECTURE.md)
- [贡献指南](./CONTRIBUTING.md)
- [代码审查指南](./CODE_REVIEW_GUIDE.md)
- [审查检查清单](./CODE_REVIEW_CHECKLIST.md)

## 🎯 快速开始

新贡献者的前 5 分钟：

```bash
# 1. 克隆仓库
git clone https://github.com/yourusername/claude-code-rs.git
cd claude-code-rs

# 2. 构建项目
cargo build --all

# 3. 运行测试
cargo test --all

# 4. 验证设置
make pre-commit

# 5. 开始开发！
```
