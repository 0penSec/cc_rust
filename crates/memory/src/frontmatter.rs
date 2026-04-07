//! Frontmatter 解析模块
//!
//! 这个模块负责解析和生成记忆文件的 frontmatter（前置元数据）。
//!
//! ## 什么是 Frontmatter？
//!
//! Frontmatter 是放在文件开头的元数据区块，被 `---` 或 `+++` 包围。
//! 它描述了文件的属性，而不是文件的内容。
//!
//! ## 为什么需要 Frontmatter？
//!
//! 1. **结构化存储**: 元数据和内容分离，便于程序读取
//! 2. **人类可读**: 用户可以直接用文本编辑器查看和编辑
//! 3. **版本控制友好**: Markdown 文件可以用 git 管理
//! 4. **兼容性好**: 许多静态网站生成器（如 Hugo、Jekyll）使用相同格式
//!
//! ## 示例
//!
//! ```markdown
//! ---
//! name: 用户角色
//! description: 用户是资深 Rust 工程师
//! type: user
//! ---
//!
//! 这是记忆的具体内容，可以很长...
//! 包含多行文本、代码块等。
//! ```

use regex::Regex;
use std::collections::HashMap;

/// 解析 frontmatter
///
/// ## 参数
///
/// - `content`: 文件的完整内容
///
/// ## 返回值
///
/// 返回一个元组 `(frontmatter_map, remaining_content)`:
/// - `frontmatter_map`: HashMap，包含所有 frontmatter 键值对
/// - `remaining_content`: 去掉 frontmatter 后的正文内容
///
/// ## 支持的格式
///
/// 1. **YAML 格式**: 用 `---` 包围，键值对用 `:` 分隔
///    ```yaml
///    ---
///    name: value
///    ---
///    ```
///
/// 2. **TOML 格式**: 用 `+++` 包围，键值对用 `=` 分隔
///    ```toml
///    +++
///    name = "value"///    +++
///    ```
///
/// ## 为什么支持两种格式？
///
/// - YAML 更简洁（不需要引号）
/// - TOML 更明确（类型更清晰）
/// - 让用户可以选择喜欢的格式
///
/// ## 使用示例
///
/// ```rust
/// use claude_memory::parse_frontmatter;
///
/// let content = r#"---
/// name: test
/// type: user
/// ---
///
/// Hello World"#;
///
/// let (frontmatter, body) = parse_frontmatter(content);
/// assert_eq!(frontmatter.get("name"), Some(&"test".to_string()));
/// assert_eq!(body, "Hello World");
/// ```
pub fn parse_frontmatter(content: &str) -> (HashMap<String, String>, String) {
    // (?s) 是正则表达式的 "dot-all" 模式，让 `.` 也能匹配换行符
    // 这样我们就可以匹配跨多行的 frontmatter
    let yaml_pattern = Regex::new(r"(?s)^---\s*\n(.*?)\n---\s*\n(.*)$").unwrap();
    let toml_pattern = Regex::new(r"(?s)^\+\+\+\s*\n(.*?)\n\+\+\+\s*\n(.*)$").unwrap();

    // 先尝试 YAML 格式
    if let Some(captures) = yaml_pattern.captures(content) {
        // captures.get(1) 是第一个捕获组：frontmatter 内容
        // captures.get(2) 是第二个捕获组：正文内容
        let frontmatter_text = captures.get(1).map(|m| m.as_str()).unwrap_or("");
        let remaining = captures.get(2).map(|m| m.as_str()).unwrap_or("");

        return (parse_yaml_like(frontmatter_text), remaining.to_string());
    }

    // 再尝试 TOML 格式
    if let Some(captures) = toml_pattern.captures(content) {
        let frontmatter_text = captures.get(1).map(|m| m.as_str()).unwrap_or("");
        let remaining = captures.get(2).map(|m| m.as_str()).unwrap_or("");

        return (parse_toml_like(frontmatter_text), remaining.to_string());
    }

    // 没有找到 frontmatter，返回空 map 和原内容
    (HashMap::new(), content.to_string())
}

/// 解析 YAML 风格的 frontmatter
///
/// ## 格式
///
/// 简单的 `key: value` 格式，每行一个键值对。
/// 支持字符串引号，但引号是可选的。
///
/// ## 示例
///
/// ```yaml
/// name: 测试
/// description: "带引号的描述"
/// type: user
/// ```
///
/// ## 为什么是 "YAML-like" 而不是完整 YAML？
///
/// 1. 完整 YAML 解析器很重（需要 serde_yaml 依赖）
/// 2. frontmatter 通常只有简单的键值对
/// 3. 自定义解析更轻量，足够满足需求
fn parse_yaml_like(text: &str) -> HashMap<String, String> {
    let mut result = HashMap::new();

    for line in text.lines() {
        let line = line.trim();
        // 跳过空行和注释行
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // 用冒号分割键和值
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim().to_string();
            // 去掉值两边的空白和引号
            let value = value.trim().trim_matches('"').trim_matches('\'').to_string();
            if !key.is_empty() {
                result.insert(key, value);
            }
        }
    }

    result
}

/// 解析 TOML 风格的 frontmatter
///
/// ## 格式
///
/// `key = "value"` 格式，每行一个键值对。
/// 字符串必须用引号包围。
///
/// ## 示例
///
/// ```toml
/// name = "测试"
/// description = "带引号的描述"
/// type = "user"
/// ```
fn parse_toml_like(text: &str) -> HashMap<String, String> {
    let mut result = HashMap::new();

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // TOML 使用等号分割
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim().to_string();
            let value = value.trim().trim_matches('"').trim_matches('\'').to_string();
            if !key.is_empty() {
                result.insert(key, value);
            }
        }
    }

    result
}

/// 生成 frontmatter
///
/// ## 参数
///
/// - `name`: 记忆的名称（简短标题）
/// - `description`: 一句话描述
/// - `memory_type`: 记忆类型（user/feedback/project/reference）
///
/// ## 返回值
///
/// 返回格式化好的 frontmatter 字符串，包含 YAML 分隔符和换行。
///
/// ## 为什么需要这个方法？
///
/// 保证所有记忆文件的 frontmatter 格式统一，用户不需要手动写格式。
///
/// ## 使用示例
///
/// ```rust
/// use claude_memory::format_frontmatter;
///
/// let frontmatter = format_frontmatter(
///     "用户角色",
///     "用户是资深工程师",
///     "user"
/// );
///
/// println!("{}", frontmatter);
/// // 输出:
/// // ---
/// // name: 用户角色
/// // description: 用户是资深工程师
/// // type: user
/// // ---
/// //
/// ```
pub fn format_frontmatter(
    name: &str,
    description: &str,
    memory_type: &str,
) -> String {
    format!(
        r#"---
name: {}
description: {}
type: {}
---

"#,
        name, description, memory_type
    )
}

/// 测试模块
///
/// ## 测试覆盖
///
/// 1. **YAML 解析**: 基本的 `---` 包围的 frontmatter
/// 2. **TOML 解析**: `+++` 包围的 frontmatter
/// 3. **无 frontmatter**: 普通文本处理
/// 4. **空 frontmatter**: `---` 后立即 `---`
/// 5. **多行内容**: frontmatter 后的内容包含多行和代码块
#[cfg(test)]
mod tests {
    use super::*;

    /// 测试 YAML frontmatter 解析
    ///
    /// ## 验证点
    ///
    /// 1. name 字段正确提取
    /// 2. type 字段正确提取
    /// 3. 正文内容保留（"This is the content"）
    #[test]
    fn test_parse_yaml_frontmatter() {
        let content = r#"---
name: user_role
description: User is a senior engineer
type: user
---

This is the content."#;

        let (frontmatter, remaining) = parse_frontmatter(content);

        assert_eq!(frontmatter.get("name"), Some(&"user_role".to_string()));
        assert_eq!(frontmatter.get("type"), Some(&"user".to_string()));
        assert!(remaining.contains("This is the content"));
    }

    /// 测试无 frontmatter 的内容
    ///
    /// ## 验证点
    ///
    /// 1. 返回空的 frontmatter map
    /// 2. 原内容完整保留
    #[test]
    fn test_no_frontmatter() {
        let content = "Just regular content without frontmatter";
        let (frontmatter, remaining) = parse_frontmatter(content);

        assert!(frontmatter.is_empty());
        assert_eq!(remaining, content);
    }

    /// 测试 frontmatter 格式化
    ///
    /// ## 验证点
    ///
    /// 1. 包含 name 字段
    /// 2. 包含 type 字段
    /// 3. 包含 YAML 分隔符 `---`
    #[test]
    fn test_format_frontmatter() {
        let result = format_frontmatter("test", "A test memory", "project");
        assert!(result.contains("name: test"));
        assert!(result.contains("type: project"));
        assert!(result.contains("---"));
    }

    /// 测试 TOML frontmatter 解析
    ///
    /// ## 验证点
    ///
    /// 1. `+++` 分隔符被正确识别
    /// 2. `key = "value"` 格式被正确解析
    /// 3. 正文内容保留
    #[test]
    fn test_parse_toml_frontmatter() {
        let content = r#"+++
name = "user_role"
description = "User is a senior engineer"
type = "user"
+++

This is the content."#;

        let (frontmatter, remaining) = parse_frontmatter(content);

        assert_eq!(frontmatter.get("name"), Some(&"user_role".to_string()));
        assert_eq!(frontmatter.get("type"), Some(&"user".to_string()));
        assert!(remaining.contains("This is the content"));
    }

    /// 测试空 frontmatter
    ///
    /// ## 验证点
    ///
    /// `---` 之间没有内容时，应返回空 map 并保留正文。
    #[test]
    fn test_empty_frontmatter() {
        let content = r#"---
---

Content after empty frontmatter."#;

        let (frontmatter, remaining) = parse_frontmatter(content);
        assert!(frontmatter.is_empty());
        assert!(remaining.contains("Content after empty frontmatter"));
    }

    /// 测试多行正文内容
    ///
    /// ## 验证点
    ///
    /// frontmatter 后的内容可以包含：
    /// - 多个段落
    /// - Markdown 格式（粗体）
    /// - 代码块
    #[test]
    fn test_multiline_content_frontmatter() {
        let content = r#"---
name: test
type: feedback
---

First paragraph.

Second paragraph with **bold** text.

```rust
let x = 1;
```

Final paragraph."#;

        let (frontmatter, remaining) = parse_frontmatter(content);
        assert_eq!(frontmatter.get("name"), Some(&"test".to_string()));
        assert!(remaining.contains("First paragraph"));
        assert!(remaining.contains("```rust"));
        assert!(remaining.contains("Final paragraph"));
    }
}
