# 代码注释规范

> 详细说明如何为代码编写清晰、有用的中文注释

---

## 核心原则

### 1. 解释"为什么"而不仅是"做什么"

```rust
// ❌ 不好的注释
// 增加计数器
counter += 1;

// ✅ 好的注释
// 用户每完成一个任务，增加积分
// 用于后续等级计算
counter += 1;
```

### 2. 所有公共 API 必须有文档

使用 `///` 为公共函数、结构体、枚举添加文档。

### 3. 使用中文

所有注释、文档使用中文，代码保持英文。

---

## 函数文档模板

```rust
/// 功能简述（一句话）
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
/// 解释设计决策的原因，包括：
/// - 为什么选择这个方案
/// - 考虑过哪些替代方案
/// - 有什么权衡取舍
///
/// ## 使用示例
///
/// ```rust
/// let result = function_name(arg1, arg2);
/// assert_eq!(result, expected);
/// ```
///
/// ## 注意事项
///
/// - 边界情况
/// - 性能考虑
/// - 线程安全等
pub fn function_name(param1: Type1, param2: Type2) -> ReturnType {
    // ...
}
```

---

## 结构体/枚举文档

```rust
/// 记忆存储管理器
///
/// ## 为什么需要这个结构？
///
/// 封装存储配置和项目隔离，提供统一的文件操作接口。
///
/// ## 使用场景
///
/// - 保存记忆文件
/// - 扫描记忆目录
/// - 管理 MEMORY.md 索引
///
/// ## 线程安全
///
/// `MemoryStorage` 实现了 `Clone` 和 `Send`，
/// 可以在多线程/多任务间安全共享。
#[derive(Debug, Clone)]
pub struct MemoryStorage {
    /// 存储配置（路径、限制等）
    config: MemoryConfig,
    /// 项目标识符（用于目录隔离）
    project_slug: String,
}
```

---

## 模块文档

每个文件开头使用 `//!` 添加模块级文档：

```rust
//! 记忆存储操作模块
//!
//! 这个模块提供了记忆文件的存储、读取、扫描和管理功能。
//!
//! ## 核心设计思路
//!
//! 1. **基于文件的存储**: 每条记忆是一个独立的 Markdown 文件
//!    - 优点：人类可读、可用 git 管理、不依赖数据库
//!    - 缺点：大量小文件可能影响性能
//!
//! 2. **项目隔离**: 每个项目有独立的记忆目录
//!
//! ## 使用示例
//!
//! ```rust
//! let storage = MemoryStorage::new(config, "my-project");
//! storage.save_memory(...).await?;
//! ```
```

---

## 测试文档

```rust
/// 测试保存和读取记忆
///
/// ## 验证点
///
/// 1. 文件被正确创建
/// 2. 读取返回内容
/// 3. frontmatter 被正确解析分离
///
/// ## 为什么重要
///
/// 这是记忆系统的基础功能，如果失败，整个系统无法工作。
#[tokio::test]
async fn test_save_and_read_memory() {
    // ...
}
```

---

## 行内注释

用于解释复杂的逻辑：

```rust
// (?s) 是正则表达式的 "dot-all" 模式，让 . 也能匹配换行符
let pattern = Regex::new(r"(?s)^---\s*\n(.*?)\n---\s*\n(.*)$").unwrap();

// 限制递归深度为 2，避免扫描过深的目录
// 经验值：大多数项目的记忆文件都在根目录或一级子目录
for entry in WalkDir::new(&dir).max_depth(2) {
```

---

## 避免的注释

```rust
// ❌ 不添加显而易见的注释
// 将 x 设置为 1
let x = 1;

// ❌ 不添加与代码不符的注释
// 检查用户是否成年
if age > 18 {  // 实际是 > 18，不是 >= 18

// ❌ 不使用无意义的占位符
// TODO: 这里需要优化
```

---

## 文档测试

Rust 会自动运行文档中的代码示例：

```rust
/// 解析 frontmatter
///
/// ## 示例
///
/// ```rust
/// use claude_memory::parse_frontmatter;
///
/// let content = "---\nname: test\n---\n\nHello";
/// let (frontmatter, body) = parse_frontmatter(content);
/// assert_eq!(frontmatter.get("name"), Some(&"test".to_string()));
/// ```
pub fn parse_frontmatter(content: &str) -> (HashMap<String, String>, String) {
    // ...
}
```

运行 `cargo test --doc` 会执行这些示例。

---

*参考主文档：[../CLAUDE.md](../CLAUDE.md)*
