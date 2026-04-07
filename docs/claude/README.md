# Claude Code 开发指南

> 本项目开发规范与工作流程文档集合

## 文档结构

| 文件 | 内容 |
|------|------|
| [comments.md](./comments.md) | 代码注释规范 - 如何写详细的中文注释 |
| [workflow.md](./workflow.md) | GitHub 提交流程 - 从编码到推送的完整流程 |
| [checklists.md](./checklists.md) | 检查清单 - 提交前、代码审查的必查项 |
| [templates.md](./templates.md) | 模板示例 - 常用代码模板 |

## 快速开始

### 提交代码前的三步检查

```bash
# 1. 格式化
cargo fmt --all

# 2. 检查
cargo clippy --all-targets --all-features -- -D warnings

# 3. 测试
cargo test --all
```

### 与 Claude 协作的常用指令

- "按照 [comments.md](./comments.md) 规范为这段代码添加注释"
- "执行 [checklists.md](./checklists.md) 中的提交前检查"
- "按照 [workflow.md](./workflow.md) 提交到 GitHub"

---

*本文档目录用于指导 Claude Code 在本项目中的工作方式*
