# Claude Code 开发指南

> 本项目开发规范与工作流程文档，供 Claude Code 加载使用。

---

## 1. 代码注释规范

### 1.1 注释原则

- **详细解释**：每个函数、结构体、枚举都需要文档注释
- **解释"为什么"**：不仅说明功能，更要解释设计决策的原因
- **示例代码**：重要的函数应包含使用示例

### 1.2 注释格式

```rust
/// 函数功能简述
///
/// ## 参数
///
/// - `param1`: 参数说明
/// - `param2`: 参数说明
///
/// ## 返回值
///
/// 返回值说明
///
/// ## 为什么这样设计
///
/// 解释设计决策、权衡考虑
///
/// ## 使用示例
///
/// ```rust
/// let result = function_name(arg1, arg2);
/// ```
pub fn function_name(param1: Type1, param2: Type2) -> ReturnType {
    // 实现代码...
}
```

### 1.3 模块级注释

每个模块文件开头应包含：

```rust
//! 模块名称
//!
//! 模块功能概述
//!
//! ## 核心设计思路
//!
//! 1. **设计点1**: 解释
//! 2. **设计点2**: 解释
//!
//! ## 使用示例
//!
//! ```rust
//! // 示例代码
//! ```
```

### 1.4 测试注释

```rust
/// 测试功能名称
///
/// ## 验证点
///
/// 1. 验证点1
/// 2. 验证点2
///
/// ## 为什么重要
///
/// 解释测试的必要性
#[test]
fn test_feature() {
    // ...
}
```

---

## 2. 语言规范

### 2.1 使用中文

- **所有注释使用中文**：包括模块文档、函数文档、测试说明
- **代码保持英文**：变量名、函数名、类型名使用英文
- **混合场景**：在中文注释中可以包含英文术语，无需翻译

### 2.2 示例

```rust
/// 解析 frontmatter
///
/// ## 为什么需要这个方法？
///
/// Frontmatter 是放在文件开头的元数据区块，
/// 它描述了文件的属性，而不是文件的内容。
pub fn parse_frontmatter(content: &str) -> (HashMap<String, String>, String) {
    // 实现...
}
```

---

## 3. GitHub 提交流程

### 3.1 提交前检查清单

```bash
# 1. 格式化代码
cargo fmt --all

# 2. 检查格式
cargo fmt --all -- --check

# 3. 运行 clippy 检查
cargo clippy --all-targets --all-features -- -D warnings

# 4. 运行测试
cargo test --all

# 5. 构建文档
cargo doc --all --no-deps
```

### 3.2 提交流程

```bash
# 1. 查看变更
git status
git diff

# 2. 暂存文件
git add <files>

# 3. 提交（遵循提交信息规范）
git commit -m "提交标题

详细说明（可选）

- 变更点1
- 变更点2"

# 4. 推送到 GitHub
git push origin main

# 5. 如需创建标签
git tag -a v0.1.0 -m "版本说明"
git push origin --tags
```

### 3.3 提交信息规范

**格式：**
```
<type>: <subject>

<body>

<footer>
```

**类型（type）：**
- `feat`: 新功能
- `fix`: 修复 bug
- `docs`: 文档更新
- `style`: 代码格式（不影响功能）
- `refactor`: 重构
- `test`: 测试相关
- `chore`: 构建/工具相关

**示例：**
```bash
git commit -m "feat: 添加记忆系统模块

- 实现四种记忆类型（user, feedback, project, reference）
- 添加文件存储和索引管理
- 包含 34 个单元测试

Closes #123"
```

### 3.4 CI 失败处理

当 CI 失败时：

1. **查看错误日志**：定位具体错误
2. **本地复现**：在本地运行相同的检查
3. **修复问题**：
   - 格式化问题：`cargo fmt --all`
   - Clippy 警告：根据提示修复
   - 测试失败：修复代码或更新测试
   - 文档错误：修复文档注释中的链接
4. **重新提交**：修复后再次推送

---

## 4. 代码审查要点

### 4.1 自我审查清单

- [ ] 所有公共 API 都有文档注释
- [ ] 注释解释了"为什么"而不仅是"做什么"
- [ ] 代码通过 `cargo fmt` 格式化
- [ ] 代码通过 `cargo clippy` 检查
- [ ] 所有测试通过
- [ ] 新增功能有对应的测试
- [ ] 文档可以正常构建（`cargo doc`）

### 4.2 常见 CI 错误及修复

| 错误类型 | 修复命令 |
|---------|---------|
| 格式错误 | `cargo fmt --all` |
| Clippy 警告 | 根据提示修改代码 |
| 文档链接错误 | 移除或修复 `[text]` 格式的链接 |
| 测试失败 | `cargo test` 查看详情并修复 |

---

## 5. 项目结构规范

### 5.1 目录组织

```
crates/
├── crate-name/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs          # 模块入口
│       ├── types.rs        # 类型定义
│       ├── storage.rs      # 存储逻辑
│       └── ...
```

### 5.2 文件命名

- 使用 `snake_case` 命名文件
- 测试文件：`module_name_test.rs` 或内联 `#[cfg(test)]` 模块

---

## 6. 版本管理

### 6.1 版本号规则（SemVer）

- **MAJOR**: 破坏性变更
- **MINOR**: 新增功能（向后兼容）
- **PATCH**: 修复 bug（向后兼容）

### 6.2 发布流程

1. 更新 `Cargo.toml` 中的版本号
2. 更新 `CHANGELOG.md`（如有）
3. 提交变更：`git commit -m "Bump version to x.x.x"`
4. 创建标签：`git tag -a vx.x.x -m "Release x.x.x"`
5. 推送：`git push origin main --tags`

---

## 7. 与 Claude 协作提示

当与 Claude 协作时，可以引用本文件中的规范：

- "为这段代码添加详细的中文注释"
- "按照提交流程检查代码"
- "修复 CI 失败的格式问题"
- "准备 GitHub 提交"

---

*最后更新：2026-04-07*
