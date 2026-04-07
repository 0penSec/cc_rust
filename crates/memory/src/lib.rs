//! # Claude Memory - 记忆系统
//!
//! 一个基于文件的持久化记忆系统，灵感来自 claude-code-main。
//!
//! ## 什么是记忆系统？
//!
//! 想象一下，如果你和一个朋友每次聊天后都失忆，那会很糟糕对吧？
//! 记忆系统就是让 AI 能够"记住"之前和用户互动的重要信息。
//!
//! ## 核心设计理念
//!
//! 1. **基于文件的存储**
//!    - 每条记忆是一个独立的 Markdown 文件
//!    - 优点：人类可读、可用 git 管理、不依赖数据库
//!    - 缺点：大量小文件，但通过限制数量解决
//!
//! 2. **四种记忆类型**
//!    - **User**: 用户信息（角色、技能、偏好）
//!    - **Feedback**: 反馈（喜欢什么、不喜欢什么）
//!    - **Project**: 项目信息（截止日期、技术栈）
//!    - **Reference**: 参考资料（文档链接、Bug追踪）
//!
//! 3. **两步保存法**
//!    - 步骤1：写入独立记忆文件（带 frontmatter）
//!    - 步骤2：在 MEMORY.md 中添加索引
//!
//! 4. **索引截断**
//!    - MEMORY.md 限制 200行/25KB
//!    - 防止索引过大占用 AI 上下文窗口
//!
//! ## 目录结构
//!
//! ```text
//! ~/.local/share/claude-code/memory/          # 基础目录
//! └── projects/
//!     └── my-project/                         # 每个项目独立
//!         └── memory/
//!             ├── MEMORY.md                   # 索引文件
//!             ├── user_role.md                # 用户角色记忆
//!             ├── feedback_testing.md         # 测试反馈
//!             ├── project_deadline.md         # 项目截止日期
//!             └── reference_api_docs.md       # API文档链接
//! ```
//!
//! ## 快速开始
//!
//! ```rust,no_run
//! use claude_memory::{MemoryStorage, MemoryConfig, MemoryType};
//!
//! # async fn example() -> std::io::Result<()> {
//! // 1. 创建存储管理器
//! let config = MemoryConfig::default();
//! let storage = MemoryStorage::new(config, "my-project");
//!
//! // 2. 保存一条记忆
//! storage.save_memory(
//!     "user_role.md",
//!     "User Role",
//!     "User is a senior Rust engineer",
//!     MemoryType::User,
//!     "Detailed content here...",
//! ).await?;
//!
//! // 3. 更新索引
//! storage.update_entrypoint_index(
//!     "user_role.md",
//!     "User Role",
//!     "User is a senior Rust engineer",
//! ).await?;
//!
//! // 4. 生成系统提示词
//! let prompt = claude_memory::build_memory_prompt(&storage).await?;
//! println!("{}", prompt);
//! # Ok(())
//! # }
//! ```
//!
//! ## 模块说明
//!
//! - [`types`]: 核心数据类型（MemoryType, MemoryHeader 等）
//! - [`frontmatter`]: Frontmatter 解析和生成
//! - [`storage`]: 文件存储操作
//! - [`retrieval`]: 记忆检索和搜索
//! - [`prompt`]: 系统提示词构建

pub mod types;
pub mod frontmatter;
pub mod storage;
pub mod retrieval;
pub mod prompt;

// 重新导出主要类型，方便用户使用
pub use types::{
    Memory,
    MemoryConfig,
    MemoryFrontmatter,
    MemoryHeader,
    MemoryType,
};
pub use storage::MemoryStorage;
pub use retrieval::{MemoryRetriever, RetrievedMemory};
pub use prompt::build_memory_prompt;
pub use frontmatter::{format_frontmatter, parse_frontmatter};

/// 错误类型
///
/// 定义了记忆系统可能返回的错误。
/// 使用 `thiserror` 派生宏自动生成错误消息。
#[derive(Debug, thiserror::Error)]
pub enum MemoryError {
    /// IO 错误（文件读写失败等）
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// 无效的记忆类型
    #[error("Invalid memory type: {0}")]
    InvalidType(String),

    /// 记忆不存在
    #[error("Memory not found: {0}")]
    NotFound(String),

    /// 记忆内容过大
    #[error("Memory too large")]
    TooLarge,
}

/// 结果类型别名
///
/// 记忆系统函数的统一返回类型。
/// 使用别名可以简化函数签名。
pub type Result<T> = std::result::Result<T, MemoryError>;

/// 记忆系统版本号
///
/// 从 Cargo.toml 中的 package.version 自动获取。
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
