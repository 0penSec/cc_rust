//! Memory type definitions
//!
//! 这个模块定义了记忆系统的核心数据类型。
//!
//! ## 为什么需要这些类型？
//!
//! 记忆系统需要存储不同类型的信息，每种信息有不同的用途。
//! 比如：用户信息（User）、反馈意见（Feedback）、项目状态（Project）、
//! 外部资源（Reference）。通过分类，AI可以更好地理解和使用这些信息。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 记忆的四种类型
///
/// ## 为什么要分类？
///
/// 就像人类会把工作笔记、个人日记、参考资料分开存放一样，
/// AI也需要区分不同类型的信息，这样才能：
/// 1. 知道什么时候该保存什么类型的记忆
/// 2. 知道什么时候该使用什么类型的记忆
/// 3. 给用户更清晰的组织结构
///
/// ## 四种类型详解
///
/// - **User**: 用户信息。比如"用户是前端专家"、"用户喜欢简洁的代码"
///   → 帮助AI理解用户的背景和偏好
///
/// - **Feedback**: 反馈。比如"不要添加不必要的注释"、"保持这样的代码风格"
///   → 帮助AI记住用户喜欢/不喜欢的工作方式
///
/// - **Project**: 项目信息。比如"我们在重构登录模块"、"周五要发布"
///   → 帮助AI理解当前工作的上下文
///
/// - **Reference**: 参考资料。比如"Bug记录在Jira的PROJECT-123"、"API文档在xxx.com"
///   → 帮助AI知道去哪里找外部信息
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryType {
    /// 用户信息：关于用户的角色、目标、知识水平
    ///
    /// 例子："用户是Rust专家"、"用户是初学者"、"用户负责后端开发"
    User,

    /// 反馈信息：用户对工作方式的指导
    ///
    /// 例子："不要添加没必要的错误处理"、"保持函数简短"
    Feedback,

    /// 项目信息：关于当前工作的上下文
    ///
    /// 例子："我们在做v2.0重构"、"周三要交付"
    Project,

    /// 参考资料：指向外部系统的链接
    ///
    /// 例子："Bug追踪在Jira"、"文档在Confluence"
    Reference,
}

impl MemoryType {
    /// 获取所有记忆类型
    ///
    /// ## 为什么需要这个方法？
    ///
    /// 当我们需要遍历所有类型（比如在生成提示词时），
    /// 不需要手动列出四种类型，直接调用这个方法即可。
    ///
    /// ## 使用示例
    ///
    /// ```rust
    /// use claude_memory::MemoryType;
    ///
    /// for memory_type in MemoryType::all() {
    ///     println!("类型: {}", memory_type.as_str());
    /// }
    /// ```
    pub fn all() -> &'static [MemoryType] {
        &[
            MemoryType::User,
            MemoryType::Feedback,
            MemoryType::Project,
            MemoryType::Reference,
        ]
    }

    /// 将类型转换为字符串
    ///
    /// ## 为什么需要这个方法？
    ///
    /// 1. 保存到文件时，类型要以字符串形式存储（如 `type: user`）
    /// 2. 显示给用户看时，需要可读的文字
    /// 3. 与前端或其他系统交互时，需要标准格式
    ///
    /// ## 返回值
    ///
    /// - `MemoryType::User` → `"user"`
    /// - `MemoryType::Feedback` → `"feedback"`
    /// - `MemoryType::Project` → `"project"`
    /// - `MemoryType::Reference` → `"reference"`
    pub fn as_str(&self) -> &'static str {
        match self {
            MemoryType::User => "user",
            MemoryType::Feedback => "feedback",
            MemoryType::Project => "project",
            MemoryType::Reference => "reference",
        }
    }
}

impl MemoryType {
    /// 获取"什么时候应该保存这种记忆"的说明
    ///
    /// ## 为什么需要这个方法？
    ///
    /// 当AI不确定是否要保存某条信息时，可以查看这个说明。
    /// 这个说明会被包含在系统提示词中，指导AI的行为。
    ///
    /// ## 使用场景
    ///
    /// 在生成系统提示词时调用，告诉AI：
    /// "当你了解到用户的角色信息时，保存为 user 类型"
    pub fn when_to_save(&self) -> &'static str {
        match self {
            MemoryType::User => {
                "When you learn any details about the user's role, preferences, responsibilities, or knowledge"
            }
            MemoryType::Feedback => {
                "Any time the user corrects your approach OR confirms a non-obvious approach worked"
            }
            MemoryType::Project => {
                "When you learn who is doing what, why, or by when"
            }
            MemoryType::Reference => {
                "When you learn about resources in external systems and their purpose"
            }
        }
    }

    /// 获取"如何使用这种记忆"的说明
    ///
    /// ## 为什么需要这个方法？
    ///
    /// 不同类型的记忆有不同的使用场景。
    /// 这个说明告诉AI什么时候应该参考这些记忆。
    ///
    /// ## 使用场景
    ///
    /// 在生成系统提示词时调用，告诉AI：
    /// "当你需要了解用户背景时，查看 user 类型的记忆"
    pub fn how_to_use(&self) -> &'static str {
        match self {
            MemoryType::User => {
                "When your work should be informed by the user's profile or perspective"
            }
            MemoryType::Feedback => {
                "Let these memories guide your behavior so that the user does not need to offer the same guidance twice"
            }
            MemoryType::Project => {
                "Use these memories to more fully understand the details and nuance behind the user's request"
            }
            MemoryType::Reference => {
                "When the user references an external system or information that may be in an external system"
            }
        }
    }
}

/// 从字符串解析记忆类型
///
/// ## 为什么实现这个trait？
///
/// 当从文件读取 `type: user` 这样的内容时，需要把它转换成 `MemoryType::User`。
/// 实现标准库的 `FromStr` trait 后，可以使用 `.parse()` 方法。
///
/// ## 使用示例
///
/// ```rust
/// use claude_memory::MemoryType;
/// use std::str::FromStr;
///
/// let ty: MemoryType = "user".parse().unwrap();
/// assert_eq!(ty, MemoryType::User);
/// ```
impl std::str::FromStr for MemoryType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "user" => Ok(MemoryType::User),
            "feedback" => Ok(MemoryType::Feedback),
            "project" => Ok(MemoryType::Project),
            "reference" => Ok(MemoryType::Reference),
            _ => Err(format!("unknown memory type: {}", s)),
        }
    }
}

/// 显示记忆类型
///
/// ## 为什么实现这个trait？
///
/// 当需要把 `MemoryType` 打印出来或转换成字符串时，
/// 可以直接用 `{}` 格式化。
///
/// ## 使用示例
///
/// ```rust
/// use claude_memory::MemoryType;
///
/// let ty = MemoryType::User;
/// println!("类型: {}", ty);  // 输出: 类型: user
/// ```
impl std::fmt::Display for MemoryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// 记忆文件的 frontmatter 元数据
///
/// ## 什么是 frontmatter？
///
/// Frontmatter 是放在文件开头的元数据区块，用 `---` 包围。
/// 它描述了文件的属性，而不是文件的内容。
///
/// ## 示例
///
/// ```markdown
/// ---
/// name: 用户角色
/// description: 用户是资深工程师
/// type: user
/// ---
///
/// 这是记忆的具体内容...
/// ```
///
/// ## 字段说明
///
/// - `name`: 记忆的名称（简短标题）
/// - `description`: 一句话描述（用于索引展示）
/// - `memory_type`: 记忆类型（user/feedback/project/reference）
/// - `extra`: 额外的自定义字段（通过 `#[serde(flatten)]` 支持扩展）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryFrontmatter {
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub memory_type: MemoryType,
    /// 额外的字段，允许用户添加自定义元数据
    #[serde(flatten)]
    pub extra: HashMap<String, serde_yaml::Value>,
}

/// 一条完整的记忆
///
/// ## 为什么需要这个结构？
///
/// 这是记忆系统的核心数据结构，包含了记忆的所有信息：
/// - 文件路径（相对和绝对）
/// - 元数据（frontmatter）
/// - 内容（去掉 frontmatter 的正文）
/// - 修改时间（用于排序和同步）
///
/// ## 使用场景
///
/// 当需要读取完整的记忆内容时使用（比如显示给用户或提供给AI）。
/// 如果只是列表面记忆，使用更轻量的 `MemoryHeader`。
#[derive(Debug, Clone)]
pub struct Memory {
    /// 相对于记忆目录的文件路径
    /// 例如: "user_role.md"
    pub path: String,
    /// 绝对文件路径
    /// 例如: "/home/user/.local/share/claude-code/memory/projects/my-project/user_role.md"
    pub absolute_path: std::path::PathBuf,
    /// 元数据（name, description, type 等）
    pub frontmatter: MemoryFrontmatter,
    /// 记忆的正文内容（不包含 frontmatter）
    pub content: String,
    /// 最后修改时间（UTC时间）
    /// 用于按时间排序，最新的记忆排在前面
    pub modified_at: chrono::DateTime<chrono::Utc>,
}

/// 记忆头部信息（轻量级索引）
///
/// ## 为什么需要这个结构？
///
/// 当扫描所有记忆文件生成索引时，不需要读取每个文件的完整内容
/// （那样太慢了）。只需要读取 frontmatter 和文件元数据即可。
///
/// ## 与 Memory 的区别
///
/// - `MemoryHeader`: 轻量，只有元数据，用于列表展示
/// - `Memory`: 完整，包含内容，用于详细展示
///
/// ## 使用场景
///
/// - 生成 MEMORY.md 索引
/// - 搜索相关记忆时快速筛选
/// - 按时间排序列出记忆
#[derive(Debug, Clone)]
pub struct MemoryHeader {
    /// 相对于记忆目录的文件名
    pub filename: String,
    /// 绝对文件路径
    pub file_path: std::path::PathBuf,
    /// 修改时间（毫秒时间戳）
    /// 使用毫秒而不是 DateTime 是为了避免时区问题
    pub mtime_ms: u64,
    /// 从 frontmatter 提取的描述
    /// 用于在索引中显示一句话简介
    pub description: Option<String>,
    /// 从 frontmatter 提取的记忆类型
    pub memory_type: Option<MemoryType>,
}

/// 记忆系统的配置
///
/// ## 为什么需要配置？
///
/// 不同的使用场景可能需要不同的限制：
/// - 本地开发 vs CI 环境可能有不同的存储位置
/// - 大型项目可能需要更大的文件限制
///
/// ## 可配置项
///
/// - `base_dir`: 记忆文件存储的根目录
/// - `max_entrypoint_lines`: MEMORY.md 最大行数（默认200）
/// - `max_entrypoint_bytes`: MEMORY.md 最大字节数（默认25KB）
/// - `max_memory_files`: 最多扫描多少个记忆文件（默认200）
#[derive(Debug, Clone)]
pub struct MemoryConfig {
    /// 记忆存储的基础目录
    /// 每个项目会在该目录下有独立的子目录
    pub base_dir: std::path::PathBuf,
    /// MEMORY.md 的最大行数限制
    /// 超过后会截断，防止文件过大影响性能
    pub max_entrypoint_lines: usize,
    /// MEMORY.md 的最大字节数限制
    pub max_entrypoint_bytes: usize,
    /// 最多扫描多少个记忆文件
    /// 防止项目过大时扫描太慢
    pub max_memory_files: usize,
}

/// 默认配置
///
/// ## 为什么这些默认值？
///
/// - 200行/25KB: 保证 MEMORY.md 能被快速读取，不会占用太多上下文窗口
/// - 200个文件: 大多数项目的记忆数量不会超过这个数
impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            base_dir: default_memory_dir(),
            max_entrypoint_lines: 200,
            max_entrypoint_bytes: 25_000,
            max_memory_files: 200,
        }
    }
}

/// 获取默认的记忆存储目录
///
/// ## 为什么用这个位置？
///
/// 遵循各操作系统的标准应用数据目录规范：
/// - Linux: `~/.local/share/claude-code/memory/`
/// - macOS: `~/Library/Application Support/com.anthropic.claude-code/memory/`
/// - Windows: `%APPDATA%/anthropic/claude-code/memory/`
///
/// ## 备用方案
///
/// 如果无法获取标准目录（比如在特殊环境下），
/// 回退到当前目录下的 `.claude-memory/`
fn default_memory_dir() -> std::path::PathBuf {
    directories::ProjectDirs::from("com", "anthropic", "claude-code")
        .map(|dirs| dirs.data_dir().join("memory"))
        .unwrap_or_else(|| std::path::PathBuf::from(".claude-memory"))
}

/// 测试模块
///
/// ## 测试策略
///
/// 1. **roundtrip 测试**: 确保字符串和类型之间可以正确互转
/// 2. **错误处理测试**: 确保无效输入被正确处理
#[cfg(test)]
mod tests {
    use super::*;

    /// 测试 MemoryType 的双向转换
    ///
    /// ## 测试逻辑
    ///
    /// 1. 遍历所有 MemoryType
    /// 2. 每个类型先转为字符串（as_str）
    /// 3. 再从字符串转回类型（from_str）
    /// 4. 验证转换后的类型与原始类型相同
    ///
    /// ## 为什么重要？
    ///
    /// 这是数据持久化的基础。如果转换有问题，
    /// 保存到文件的记忆类型就无法正确读取。
    #[test]
    fn test_memory_type_roundtrip() {
        use std::str::FromStr;

        for &ty in MemoryType::all() {
            let s = ty.as_str();
            let parsed = MemoryType::from_str(s).ok();
            assert_eq!(parsed, Some(ty));
        }
    }

    /// 测试无效的记忆类型字符串
    ///
    /// ## 测试逻辑
    ///
    /// 验证 "invalid" 和空字符串会被正确拒绝，返回错误。
    ///
    /// ## 为什么重要？
    ///
    /// 防止因为文件损坏或手动编辑错误导致的类型解析问题。
    ///  gracefully 处理错误比 panic 更好。
    #[test]
    fn test_invalid_memory_type() {
        use std::str::FromStr;

        assert!(MemoryType::from_str("invalid").is_err());
        assert!(MemoryType::from_str("").is_err());
    }
}
