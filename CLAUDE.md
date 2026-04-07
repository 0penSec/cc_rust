# Claude Code 开发指南

> 本项目开发规范与工作流程文档

## 📚 文档导航

| 文档 | 内容 | 何时使用 |
|------|------|----------|
| [docs/claude/README.md](docs/claude/README.md) | 文档首页与导航 | 首次查看 |
| [docs/claude/comments.md](docs/claude/comments.md) | 代码注释规范 | 需要写注释时 |
| [docs/claude/workflow.md](docs/claude/workflow.md) | GitHub 提交流程 | 提交代码前 |
| [docs/claude/checklists.md](docs/claude/checklists.md) | 检查清单 | 提交前自查 |
| [docs/claude/templates.md](docs/claude/templates.md) | 代码模板 | 需要写新代码时 |

## 🚀 快速开始

### 常用指令

与 Claude 协作时可以使用：

- **"按照注释规范为这段代码添加中文注释"** → 参考 [comments.md](docs/claude/comments.md)
- **"执行提交前检查"** → 参考 [checklists.md](docs/claude/checklists.md)
- **"提交到 GitHub"** → 参考 [workflow.md](docs/claude/workflow.md)
- **"给我函数文档模板"** → 参考 [templates.md](docs/claude/templates.md)

### 提交前三步

```bash
# 1. 格式化
cargo fmt --all

# 2. 检查
cargo clippy --all-targets --all-features -- -D warnings

# 3. 测试
cargo test --all
```

## 📋 核心原则

1. **详细的中文注释** - 所有代码注释使用中文，解释"为什么"
2. **完整的文档** - 每个公共 API 都有文档注释和示例
3. **严格的检查** - 提交前必须通过 fmt、clippy、test 检查
4. **清晰的提交** - 遵循 Conventional Commits 规范

## 🔧 项目信息

- **语言**: Rust
- **版本管理**: Git + GitHub
- **CI/CD**: GitHub Actions
- **代码风格**: `rustfmt` + `clippy`

---

*详细内容请查看 [docs/claude/](docs/claude/) 目录下的子文档*
