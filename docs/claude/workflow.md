# GitHub 提交流程

> 从编码到推送到 GitHub 的完整工作流程

---

## 提交前检查（必做）

按照以下顺序执行：

### 1. 格式化代码

```bash
cargo fmt --all
```

### 2. 检查格式

```bash
cargo fmt --all -- --check
```

如果输出为空，说明格式正确。如果有输出，第一步会自动修复。

### 3. Clippy 检查

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

`-D warnings` 将警告视为错误。

### 4. 运行测试

```bash
cargo test --all
```

### 5. 构建文档

```bash
cargo doc --all --no-deps
```

检查是否有文档链接错误。

---

## 提交流程

### 步骤 1：查看变更

```bash
git status
git diff
```

确认变更内容正确。

### 步骤 2：暂存文件

```bash
# 暂存特定文件
git add <file1> <file2>

# 或暂存所有变更
git add -A
```

### 步骤 3：提交

提交信息格式（Conventional Commits）：

```bash
git commit -m "type: 简短描述

详细说明（可选）

- 变更点 1
- 变更点 2

Closes #issue-number"
```

**类型（type）：**

| 类型 | 说明 |
|------|------|
| `feat` | 新功能 |
| `fix` | 修复 bug |
| `docs` | 仅文档更新 |
| `style` | 代码格式调整（不影响功能） |
| `refactor` | 重构（既不是 feat 也不是 fix） |
| `test` | 添加或修改测试 |
| `chore` | 构建过程或辅助工具的变动 |

**示例：**

```bash
git commit -m "feat: 添加记忆检索功能

- 实现关键词匹配搜索
- 添加相关性评分
- 包含 8 个单元测试

Closes #45"
```

### 步骤 4：推送到 GitHub

```bash
# 推送到 main 分支
git push origin main

# 推送标签（如果创建了）
git push origin --tags
```

### 步骤 5：验证 CI

访问 https://github.com/0penSec/cc_rust/actions 查看构建状态。

---

## 版本发布流程

### 1. 更新版本号

编辑 `Cargo.toml`：

```toml
[package]
name = "crate-name"
version = "0.2.0"  # 更新这里
```

版本号遵循 SemVer：
- **MAJOR**: 破坏性变更
- **MINOR**: 新增功能（向后兼容）
- **PATCH**: 修复 bug（向后兼容）

### 2. 提交版本变更

```bash
git add Cargo.toml
git commit -m "chore: Bump version to 0.2.0"
```

### 3. 创建标签

```bash
git tag -a v0.2.0 -m "Release v0.2.0

Features:
- 新增记忆检索功能
- 优化存储性能
- 添加中文文档"
```

### 4. 推送

```bash
git push origin main --tags
```

---

## CI 失败处理

### 常见错误及修复

#### 1. 格式错误

```
Diff in ...
-    let x = 1;
+    let x = 1;
```

**修复：**
```bash
cargo fmt --all
git add -A
git commit --amend --no-edit
git push origin main --force-with-lease
```

#### 2. Clippy 警告

```
error: unused import
  --> src/lib.rs:3:14
```

**修复：**
根据提示移除未使用的导入，然后重新提交。

#### 3. 文档链接错误

```
error: unresolved link to `系统提示词构建`
```

**修复：**
移除中文方括号链接，改为纯文本：
```rust
// 之前
//! - [`prompt`]: 系统提示词构建

// 之后
//! - `prompt`: 系统提示词构建
```

#### 4. 测试失败

```
test storage::tests::test_save ... FAILED
```

**修复：**
```bash
# 本地复现
cargo test test_save -- --nocapture

# 修复代码
# ...

# 重新提交
git add -A
git commit -m "fix: 修复存储测试"
git push origin main
```

---

## 快速参考

```bash
# 完整提交流程
cargo fmt --all \
  && cargo clippy --all-targets --all-features -- -D warnings \
  && cargo test --all \
  && cargo doc --all --no-deps \
  && git add -A \
  && git commit -m "type: 描述" \
  && git push origin main

# 修复后强制推送（仅用于个人分支）
git add -A
git commit --amend --no-edit
git push origin main --force-with-lease
```

---

*参考主文档：[../CLAUDE.md](../CLAUDE.md)*
