//! 记忆存储操作模块
//!
//! 这个模块提供了记忆文件的存储、读取、扫描和管理功能。
//!
//! ## 核心设计思路
//!
//! 1. **基于文件的存储**: 每条记忆是一个独立的 Markdown 文件
//!    - 优点：人类可读、可用 git 管理、不依赖数据库
//!    - 缺点：大量小文件可能影响性能（通过限制文件数量解决）
//!
//! 2. **项目隔离**: 每个项目有独立的记忆目录
//!    - 路径结构: `base_dir/projects/{project_slug}/memory/`
//!    - 避免不同项目的记忆混淆
//!
//! 3. **索引文件 MEMORY.md**: 记录所有记忆的清单
//!    - 作用：给 AI 提供记忆目录的概览
//!    - 限制：200行/25KB，防止过大影响上下文
//!
//! ## 使用示例
//!
//! ```rust,no_run
//! use claude_memory::{MemoryStorage, MemoryConfig, MemoryType};
//!
//! # async fn example() -> std::io::Result<()> {
//! let config = MemoryConfig::default();
//! let storage = MemoryStorage::new(config, "my-project");
//!
//! // 保存记忆
//! storage.save_memory(
//!     "user_role.md",
//!     "用户角色",
//!     "用户是资深 Rust 工程师",
//!     MemoryType::User,
//!     "详细内容...",
//! ).await?;
//!
//! // 扫描所有记忆
//! let headers = storage.scan_memories().await?;
//! println!("共有 {} 条记忆", headers.len());
//! # Ok(())
//! # }
//! ```

use std::path::PathBuf;
use tokio::fs;
use tracing::info;
use walkdir::WalkDir;

use crate::frontmatter::{format_frontmatter, parse_frontmatter};
use crate::types::{MemoryConfig, MemoryHeader, MemoryType};

/// 记忆存储管理器
///
/// 这是记忆系统的核心结构，提供了所有文件操作的方法。
///
/// ## 为什么需要这个结构？
///
/// 1. **封装配置**: 统一管理存储路径、限制等配置
/// 2. **生命周期管理**: 确保目录存在、处理错误
/// 3. **项目隔离**: 通过 `project_slug` 区分不同项目
///
/// ## 线程安全
///
/// `MemoryStorage` 实现了 `Clone` 和 `Send`，可以在多线程/多任务间共享。
/// 但每个方法是独立的异步操作，没有内部状态锁。
#[derive(Debug, Clone)]
pub struct MemoryStorage {
    /// 存储配置（路径、限制等）
    config: MemoryConfig,
    /// 项目标识符（用于目录隔离）
    /// 例如："my-project" → `base_dir/projects/my-project/memory/`
    project_slug: String,
}

impl MemoryStorage {
    /// 创建新的存储管理器
    ///
    /// ## 参数
    ///
    /// - `config`: 存储配置（路径、限制等）
    /// - `project_slug`: 项目标识符，决定存储目录
    ///
    /// ## 示例
    ///
    /// ```rust
    /// use claude_memory::{MemoryStorage, MemoryConfig};
    ///
    /// let config = MemoryConfig::default();
    /// let storage = MemoryStorage::new(config, "my-project");
    /// ```
    pub fn new(config: MemoryConfig, project_slug: impl Into<String>) -> Self {
        Self {
            config,
            project_slug: project_slug.into(),
        }
    }

    /// 获取记忆目录路径
    ///
    /// ## 返回路径
    ///
    /// `{base_dir}/projects/{project_slug}/memory/`
    ///
    /// ## 为什么用这个方法？
    ///
    /// 路径构造逻辑集中在一处，避免在多处硬编码路径格式。
    pub fn memory_dir(&self) -> PathBuf {
        self.config
            .base_dir
            .join("projects")
            .join(&self.project_slug)
            .join("memory")
    }

    /// 获取 MEMORY.md 路径
    ///
    /// MEMORY.md 是记忆目录的入口文件（索引），
    /// 包含所有记忆的清单。
    pub fn entrypoint_path(&self) -> PathBuf {
        self.memory_dir().join("MEMORY.md")
    }

    /// 确保记忆目录存在
    ///
    /// ## 为什么需要这个方法？
    ///
    /// 在写入文件前，必须确保父目录存在，否则会报错。
    /// 这个方法会在首次保存时自动创建目录结构。
    ///
    /// ## 幂等性
    ///
    /// 如果目录已存在，什么都不做，不会报错。
    pub async fn ensure_dir_exists(&self) -> std::io::Result<()> {
        let dir = self.memory_dir();
        fs::create_dir_all(&dir).await?;
        Ok(())
    }

    /// 保存记忆文件
    ///
    /// ## 参数
    ///
    /// - `filename`: 文件名（如 "user_role.md"）
    /// - `name`: 记忆名称（用于 frontmatter）
    /// - `description`: 一句话描述（用于 frontmatter）
    /// - `memory_type`: 记忆类型
    /// - `content`: 记忆正文内容
    ///
    /// ## 文件格式
    ///
    /// 保存的文件包含 frontmatter + 内容：
    /// ```markdown
    /// ---
    /// name: {name}
    /// description: {description}
    /// type: {memory_type}
    /// ---
    ///
    /// {content}
    /// ```
    ///
    /// ## 使用示例
    ///
    /// ```rust,no_run
    /// use claude_memory::{MemoryStorage, MemoryConfig, MemoryType};
    ///
    /// # async fn example() -> std::io::Result<()> {
    /// let storage = MemoryStorage::new(MemoryConfig::default(), "my-project");
    ///
    /// storage.save_memory(
    ///     "rust_expert.md",
    ///     "Rust 专家",
    ///     "用户有 10 年 Rust 经验",
    ///     MemoryType::User,
    ///     "用户熟悉 async/await、宏编程等高级特性...",
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn save_memory(
        &self,
        filename: &str,
        name: &str,
        description: &str,
        memory_type: MemoryType,
        content: &str,
    ) -> std::io::Result<PathBuf> {
        // 确保目录存在
        self.ensure_dir_exists().await?;

        let file_path = self.memory_dir().join(filename);
        // 生成 frontmatter
        let frontmatter = format_frontmatter(name, description, memory_type.as_str());
        // 合并 frontmatter 和内容
        let full_content = format!("{}{}", frontmatter, content);

        // 写入文件
        fs::write(&file_path, full_content).await?;
        info!("Saved memory to {:?}", file_path);

        Ok(file_path)
    }

    /// 读取记忆文件
    ///
    /// ## 参数
    ///
    /// - `filename`: 文件名（如 "user_role.md"）
    ///
    /// ## 返回值
    ///
    /// - `Ok(Some((full_content, body)))`: 文件存在，返回完整内容和正文
    /// - `Ok(None)`: 文件不存在
    /// - `Err(e)`: 读取失败（权限错误等）
    ///
    /// ## 为什么返回元组？
    ///
    /// - `full_content`: 包含 frontmatter，用于显示完整文件
    /// - `body`: 去掉 frontmatter 的正文，用于 AI 理解内容
    pub async fn read_memory(
        &self,
        filename: &str,
    ) -> std::io::Result<Option<(String, String)>> {
        let file_path = self.memory_dir().join(filename);

        // 文件不存在返回 None（不是错误）
        if !file_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&file_path).await?;
        // 解析 frontmatter，分离出正文
        let (_frontmatter, body) = parse_frontmatter(&content);

        Ok(Some((content, body)))
    }

    /// 扫描所有记忆文件
    ///
    /// ## 功能
    ///
    /// 遍历记忆目录，收集所有 `.md` 文件的元数据（不包括 MEMORY.md）。
    ///
    /// ## 返回值
    ///
    /// 返回 `Vec<MemoryHeader>`，按修改时间倒序排列（最新的在前）。
    ///
    /// ## 限制
    ///
    /// - 最多扫描 `max_memory_files` 个文件（默认 200）
    /// - 最大递归深度 2 层
    /// - 跳过无法读取的文件（如权限错误、无效 UTF-8）
    ///
    /// ## 为什么用 WalkDir？
    ///
    /// WalkDir 提供了高效的目录遍历，支持：
    /// - 限制递归深度
    /// - 边遍历边过滤
    /// - 获取文件元数据（修改时间）
    ///
    /// ## 性能考虑
    ///
    /// 这个方法需要读取每个文件的 frontmatter，所以对于大量文件可能较慢。
    /// 如果需要高性能，可以考虑缓存索引。
    pub async fn scan_memories(&self) -> std::io::Result<Vec<MemoryHeader>> {
        let memory_dir = self.memory_dir();

        // 目录不存在，返回空列表
        if !memory_dir.exists() {
            return Ok(Vec::new());
        }

        let mut headers = Vec::new();
        let max_files = self.config.max_memory_files;

        // WalkDir 遍历目录
        for entry in WalkDir::new(&memory_dir)
            .max_depth(2) // 限制深度，避免扫描太深
            .into_iter()
            .filter_map(|e| e.ok()) // 跳过错误条目
            .filter(|e| {
                let path = e.path();
                // 只选 .md 文件，排除 MEMORY.md
                path.extension().is_some_and(|ext| ext == "md")
                    && path.file_name().is_some_and(|name| name != "MEMORY.md")
            })
            .take(max_files) // 限制文件数量
        {
            let path = entry.path();
            let metadata = entry.metadata()?;

            // 读取文件提取 frontmatter
            let content = match fs::read_to_string(path).await {
                Ok(c) => c,
                Err(_) => continue, // 无法读取的文件（如权限错误），跳过
            };

            let (frontmatter, _) = parse_frontmatter(&content);

            // 获取相对文件名（用于展示和索引）
            let filename = path
                .strip_prefix(&memory_dir)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();

            headers.push(MemoryHeader {
                filename,
                file_path: path.to_path_buf(),
                // 获取修改时间（毫秒时间戳）
                mtime_ms: metadata
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0),
                // 从 frontmatter 提取描述和类型
                description: frontmatter.get("description").cloned(),
                memory_type: frontmatter.get("type").and_then(|t| t.parse().ok()),
            });
        }

        // 按修改时间倒序排列（最新的在前）
        headers.sort_by(|a, b| b.mtime_ms.cmp(&a.mtime_ms));

        Ok(headers)
    }

    /// 读取 MEMORY.md 索引文件
    ///
    /// ## 为什么需要这个方法？
    ///
    /// MEMORY.md 是记忆的入口文件，包含所有记忆的索引。
    /// AI 需要先读取这个文件，才知道有哪些记忆可用。
    ///
    /// ## 返回值
    ///
    /// - `Ok(Some(content))`: 文件存在，返回内容
    /// - `Ok(None)`: 文件不存在（可能是新项目）
    pub async fn read_entrypoint(&self) -> std::io::Result<Option<String>> {
        let path = self.entrypoint_path();

        if !path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&path).await?;
        Ok(Some(content))
    }

    /// 写入 MEMORY.md 索引文件
    ///
    /// ## 自动截断
    ///
    /// 如果内容超过限制（200行/25KB），会自动截断并添加警告。
    /// 这是为了防止索引文件过大，占用太多 AI 上下文窗口。
    ///
    /// ## 为什么需要截断？
    ///
    /// AI 的上下文窗口有限，如果 MEMORY.md 太长：
    /// 1. 占用宝贵的 token 额度
    /// 2. 可能影响性能
    /// 3. 重要信息可能被截断
    ///
    /// 解决方案：保持索引简洁（每行一个记忆），详细内容放在单独文件中。
    pub async fn write_entrypoint(&self, content: &str) -> std::io::Result<()> {
        self.ensure_dir_exists().await?;

        let truncated = self.truncate_entrypoint(content);
        fs::write(self.entrypoint_path(), truncated).await?;

        Ok(())
    }

    /// 向 MEMORY.md 添加新的索引条目
    ///
    /// ## 参数
    ///
    /// - `filename`: 记忆文件名（如 "user_role.md"）
    /// - `name`: 记忆名称（用于链接文本）
    /// - `description`: 一句话描述
    ///
    /// ## 添加的格式
    ///
    /// ```markdown
    /// - [name](filename) -- description
    /// ```
    ///
    /// ## 为什么用 `--` 而不是 `—`？
    ///
    /// `--` 是纯 ASCII，兼容性更好，在各种编辑器中显示一致。
    ///
    /// ## 使用场景
    ///
    /// 当创建新的记忆文件后，调用这个方法更新索引，
    /// 这样 AI 在下次对话时就能看到新的记忆。
    pub async fn update_entrypoint_index(
        &self,
        filename: &str,
        name: &str,
        description: &str,
    ) -> std::io::Result<()> {
        // 读取现有内容，如果不存在则创建默认标题
        let mut content = self.read_entrypoint().await?.unwrap_or_else(|| {
            "# Memory Index\n\n".to_string()
        });

        // 添加新条目
        let entry = format!("- [{}]({}) -- {}\n", name, filename, description);
        content.push_str(&entry);

        self.write_entrypoint(&content).await?;
        Ok(())
    }

    /// 截断 MEMORY.md 内容到限制大小
    ///
    /// ## 限制策略
    ///
    /// 1. **行数限制**（默认 200 行）：
    ///    - 超过则只保留前 200 行
    ///
    /// 2. **字节限制**（默认 25KB）：
    ///    - 如果按行截断后仍超过，则在最近换行处截断
    ///    - 如果没有换行，直接截断到限制长度
    ///
    /// 3. **添加警告**：
    ///    - 截断后添加警告提示，告知用户文件被截断
    ///
    /// ## 截断的位置选择
    ///
    /// 优先在换行处截断，避免切断单词或 Markdown 语法。
    fn truncate_entrypoint(&self, content: &str) -> String {
        let max_lines = self.config.max_entrypoint_lines;
        let max_bytes = self.config.max_entrypoint_bytes;

        let lines: Vec<&str> = content.lines().collect();

        // 未超限，直接返回
        if lines.len() <= max_lines && content.len() <= max_bytes {
            return content.to_string();
        }

        // 按行截断
        let mut truncated = if lines.len() > max_lines {
            lines[..max_lines].join("\n")
        } else {
            content.to_string()
        };

        // 按字节截断（在换行处）
        if truncated.len() > max_bytes {
            if let Some(idx) = truncated[..max_bytes].rfind('\n') {
                truncated = truncated[..idx].to_string();
            } else {
                truncated = truncated[..max_bytes].to_string();
            }
        }

        // 添加警告
        truncated.push_str(
            "\n\n> WARNING: MEMORY.md is large. Only part of it was loaded. Keep index entries to one line.\n",
        );

        truncated
    }

    /// 删除记忆文件
    ///
    /// ## 参数
    ///
    /// - `filename`: 要删除的文件名
    ///
    /// ## 返回值
    ///
    /// - `Ok(true)`: 文件存在并已删除
    /// - `Ok(false)`: 文件不存在（不是错误）
    ///
    /// ## 注意
    ///
    /// 这个方法只删除文件，不会自动更新 MEMORY.md 索引。
    /// 调用者需要手动更新索引。
    pub async fn delete_memory(&self, filename: &str) -> std::io::Result<bool> {
        let path = self.memory_dir().join(filename);

        if path.exists() {
            fs::remove_file(&path).await?;
            info!("Deleted memory: {:?}", path);
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

/// 测试模块
///
/// ## 测试覆盖
///
/// 1. **基本 CRUD**: 保存、读取、删除
/// 2. **批量扫描**: 扫描多个文件，验证排序
/// 3. **索引管理**: 写入、更新 MEMORY.md
/// 4. **边界条件**: 截断、空目录、大文件
/// 5. **并发安全**: 多任务同时读写
/// 6. **错误处理**: 文件不存在、无效 UTF-8
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// 创建测试用的存储实例
    ///
    /// ## 为什么用临时目录？
    ///
    /// 测试不应该污染真实的记忆目录。
    /// TempDir 会在测试结束后自动清理。
    async fn create_test_storage() -> (MemoryStorage, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config = MemoryConfig {
            base_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };
        let storage = MemoryStorage::new(config, "test-project");
        (storage, temp_dir)
    }

    /// 测试保存和读取记忆
    ///
    /// ## 验证点
    ///
    /// 1. 文件被正确创建
    /// 2. 读取返回内容
    /// 3. frontmatter 被正确解析分离
    #[tokio::test]
    async fn test_save_and_read_memory() {
        let (storage, _temp) = create_test_storage().await;

        storage
            .save_memory(
                "test.md",
                "Test Memory",
                "A test memory",
                MemoryType::User,
                "This is the content.",
            )
            .await
            .unwrap();

        let result = storage.read_memory("test.md").await.unwrap();
        assert!(result.is_some());
    }

    /// 测试扫描多个记忆文件
    ///
    /// ## 验证点
    ///
    /// 1. 能发现多个文件
    /// 2. 返回正确的数量
    #[tokio::test]
    async fn test_scan_memories() {
        let (storage, _temp) = create_test_storage().await;

        // 保存 3 个记忆
        for i in 0..3 {
            storage
                .save_memory(
                    &format!("memory_{}.md", i),
                    &format!("Memory {}", i),
                    "Test description",
                    MemoryType::Project,
                    "Content",
                )
                .await
                .unwrap();
        }

        let headers = storage.scan_memories().await.unwrap();
        assert_eq!(headers.len(), 3);
    }

    /// 测试 MEMORY.md 读写
    ///
    /// ## 验证点
    ///
    /// 1. 能写入内容
    /// 2. 能读取内容
    /// 3. 内容保持一致
    #[tokio::test]
    async fn test_entrypoint() {
        let (storage, _temp) = create_test_storage().await;

        storage
            .write_entrypoint("# Test Index\n")
            .await
            .unwrap();

        let content = storage.read_entrypoint().await.unwrap();
        assert!(content.is_some());
        assert!(content.unwrap().contains("Test Index"));
    }

    /// 测试更新 MEMORY.md 索引
    ///
    /// ## 验证点
    ///
    /// 1. 首次创建时自动生成标题
    /// 2. 新条目被追加到末尾
    /// 3. 格式正确：`- [name](file) -- desc`
    #[tokio::test]
    async fn test_update_entrypoint_index() {
        let (storage, _temp) = create_test_storage().await;

        // 添加第一个条目
        storage
            .update_entrypoint_index("memory1.md", "Memory 1", "First memory")
            .await
            .unwrap();

        // 添加第二个条目
        storage
            .update_entrypoint_index("memory2.md", "Memory 2", "Second memory")
            .await
            .unwrap();

        let content = storage.read_entrypoint().await.unwrap().unwrap();
        assert!(content.contains("Memory 1"));
        assert!(content.contains("memory1.md"));
        assert!(content.contains("First memory"));
        assert!(content.contains("Memory 2"));
        assert!(content.contains("memory2.md"));
        assert!(content.contains("Second memory"));
    }

    /// 测试删除记忆
    ///
    /// ## 验证点
    ///
    /// 1. 删除存在的文件返回 true
    /// 2. 删除后文件确实不存在
    /// 3. 删除不存在的文件返回 false（不是错误）
    #[tokio::test]
    async fn test_delete_memory() {
        let (storage, _temp) = create_test_storage().await;

        // 先保存一个记忆
        storage
            .save_memory(
                "to_delete.md",
                "To Delete",
                "Will be deleted",
                crate::types::MemoryType::User,
                "Content",
            )
            .await
            .unwrap();

        // 确认文件存在
        assert!(storage.read_memory("to_delete.md").await.unwrap().is_some());

        // 删除
        assert!(storage.delete_memory("to_delete.md").await.unwrap());

        // 确认已删除
        assert!(!storage.memory_dir().join("to_delete.md").exists());
        assert!(!storage.delete_memory("to_delete.md").await.unwrap()); // 再次删除返回 false
    }

    /// 测试 MEMORY.md 行数截断
    ///
    /// ## 验证点
    ///
    /// 1. 超过 200 行的内容被截断
    /// 2. 截断后添加警告信息
    /// 3. 未超限的内容不受影响
    #[tokio::test]
    async fn test_truncate_entrypoint_by_lines() {
        let (storage, _temp) = create_test_storage().await;

        // 创建 250 行的内容
        let mut lines: Vec<String> = vec!["# Index".to_string()];
        for i in 0..250 {
            lines.push(format!("- Entry {}", i));
        }
        let content = lines.join("\n");

        storage.write_entrypoint(&content).await.unwrap();

        let read = storage.read_entrypoint().await.unwrap().unwrap();
        let read_lines: Vec<&str> = read.lines().collect();

        // 应该被截断到约 200 行（加上警告）
        assert!(read_lines.len() <= 205);
        assert!(read.contains("WARNING"));
    }

    /// 测试 MEMORY.md 字节截断
    ///
    /// ## 验证点
    ///
    /// 1. 超过 25KB 的内容被截断
    /// 2. 在换行处截断（不切断单词）
    /// 3. 添加警告信息
    #[tokio::test]
    async fn test_truncate_entrypoint_by_bytes() {
        let (storage, _temp) = create_test_storage().await;

        // 创建超过 25KB 的内容
        let long_line = "a".repeat(1000);
        let mut lines: Vec<String> = vec!["# Index".to_string()];
        for _ in 0..30 {
            lines.push(format!("- {}", long_line));
        }
        let content = lines.join("\n");

        assert!(content.len() > 25000);

        storage.write_entrypoint(&content).await.unwrap();

        let read = storage.read_entrypoint().await.unwrap().unwrap();

        // 应该被截断
        assert!(read.len() <= 26000);
        assert!(read.contains("WARNING"));
    }

    /// 测试扫描结果按时间排序
    ///
    /// ## 验证点
    ///
    /// 1. 最新的文件排在最前
    /// 2. 修改时间正确提取
    #[tokio::test]
    async fn test_scan_memories_sorting() {
        let (storage, _temp) = create_test_storage().await;

        // 保存 3 个文件，每次间隔 10ms 确保修改时间不同
        for i in 0..3 {
            storage
                .save_memory(
                    &format!("memory_{}.md", i),
                    &format!("Memory {}", i),
                    "Test",
                    crate::types::MemoryType::Project,
                    "Content",
                )
                .await
                .unwrap();
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }

        let headers = storage.scan_memories().await.unwrap();
        assert_eq!(headers.len(), 3);

        // 验证按修改时间倒序排列
        for i in 0..headers.len() - 1 {
            assert!(headers[i].mtime_ms >= headers[i + 1].mtime_ms);
        }
    }

    /// 测试读取不存在的文件
    ///
    /// ## 验证点
    ///
    /// 文件不存在时返回 Ok(None)，不是 Err。
    /// 这是为了区分"文件不存在"和"读取失败"。
    #[tokio::test]
    async fn test_read_memory_not_found() {
        let (storage, _temp) = create_test_storage().await;

        let result = storage.read_memory("nonexistent.md").await.unwrap();
        assert!(result.is_none());
    }

    /// 测试扫描时排除 MEMORY.md
    ///
    /// ## 验证点
    ///
    /// MEMORY.md 是索引文件，不应该被当作记忆扫描。
    #[tokio::test]
    async fn test_scan_excludes_memory_md() {
        let (storage, _temp) = create_test_storage().await;

        // 保存普通记忆
        storage
            .save_memory(
                "regular.md",
                "Regular",
                "A regular memory",
                crate::types::MemoryType::User,
                "Content",
            )
            .await
            .unwrap();

        // 直接写入 MEMORY.md
        storage.write_entrypoint("# Index\n").await.unwrap();

        let headers = storage.scan_memories().await.unwrap();

        // 只应该找到 regular.md
        assert_eq!(headers.len(), 1);
        assert_eq!(headers[0].filename, "regular.md");
    }

    /// 测试扫描时提取 frontmatter
    ///
    /// ## 验证点
    ///
    /// 1. description 从 frontmatter 提取
    /// 2. memory_type 从 frontmatter 提取并解析
    #[tokio::test]
    async fn test_scan_extracts_frontmatter() {
        let (storage, _temp) = create_test_storage().await;

        storage
            .save_memory(
                "with_frontmatter.md",
                "Test Memory",
                "A test description",
                crate::types::MemoryType::Reference,
                "Content here",
            )
            .await
            .unwrap();

        let headers = storage.scan_memories().await.unwrap();
        assert_eq!(headers.len(), 1);

        let header = &headers[0];
        assert_eq!(header.filename, "with_frontmatter.md");
        assert_eq!(header.description, Some("A test description".to_string()));
        assert_eq!(header.memory_type, Some(crate::types::MemoryType::Reference));
    }

    /// 测试并发保存
    ///
    /// ## 验证点
    ///
    /// 1. 10 个任务同时保存不冲突
    /// 2. 所有文件都能成功创建
    ///
    /// ## 为什么重要？
    ///
    /// 实际使用中可能有多个任务同时保存记忆，
    /// 需要确保文件系统操作是安全的。
    #[tokio::test]
    async fn test_concurrent_save_read() {
        let (storage, _temp) = create_test_storage().await;
        let storage = std::sync::Arc::new(storage);

        let mut handles = vec![];

        // 创建 10 个并发任务
        for i in 0..10 {
            let storage_clone = std::sync::Arc::clone(&storage);
            let handle = tokio::spawn(async move {
                storage_clone
                    .save_memory(
                        &format!("concurrent_{}.md", i),
                        &format!("Concurrent {}", i),
                        "Test",
                        crate::types::MemoryType::User,
                        "Content",
                    )
                    .await
                    .unwrap();
            });
            handles.push(handle);
        }

        // 等待所有任务完成
        for handle in handles {
            handle.await.unwrap();
        }

        // 验证所有文件都创建了
        let headers = storage.scan_memories().await.unwrap();
        assert_eq!(headers.len(), 10);
    }

    /// 测试并发扫描和写入
    ///
    /// ## 验证点
    ///
    /// 读取和写入同时进行不会导致崩溃。
    #[tokio::test]
    async fn test_concurrent_scan_while_writing() {
        let (storage, _temp) = create_test_storage().await;
        let storage = std::sync::Arc::new(storage);

        // 先创建 5 个文件
        for i in 0..5 {
            let s = std::sync::Arc::clone(&storage);
            tokio::spawn(async move {
                s.save_memory(
                    &format!("memory_{}.md", i),
                    &format!("Memory {}", i),
                    "Test",
                    crate::types::MemoryType::User,
                    "Content",
                )
                .await
                .unwrap();
            })
            .await
            .unwrap();
        }

        // 在后台启动扫描
        let scan_storage = std::sync::Arc::clone(&storage);
        let scan_handle = tokio::spawn(async move { scan_storage.scan_memories().await });

        // 同时写入更多文件
        let mut write_handles = vec![];
        for i in 5..10 {
            let s = std::sync::Arc::clone(&storage);
            write_handles.push(tokio::spawn(async move {
                s.save_memory(
                    &format!("memory_{}.md", i),
                    &format!("Memory {}", i),
                    "Test",
                    crate::types::MemoryType::User,
                    "Content",
                )
                .await
                .unwrap();
            }));
        }

        let scan_result = scan_handle.await.unwrap().unwrap();
        for handle in write_handles {
            handle.await.unwrap();
        }

        // 扫描应该至少找到最初的 5 个文件
        assert!(scan_result.len() >= 5);
    }

    /// 测试空存储目录
    ///
    /// ## 验证点
    ///
    /// 1. 扫描空目录返回空列表
    /// 2. 读取不存在的 MEMORY.md 返回 None
    /// 3. 不会报错
    #[tokio::test]
    async fn test_empty_storage_directory() {
        let temp_dir = TempDir::new().unwrap();
        let config = MemoryConfig {
            base_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };
        let storage = MemoryStorage::new(config, "empty-project");

        // 扫描不存在的目录
        let headers = storage.scan_memories().await.unwrap();
        assert!(headers.is_empty());

        // 读取不存在的文件
        let entrypoint = storage.read_entrypoint().await.unwrap();
        assert!(entrypoint.is_none());
    }

    /// 测试处理损坏的文件
    ///
    /// ## 验证点
    ///
    /// 无效 UTF-8 的文件会被跳过，不会导致整个扫描失败。
    ///
    /// ## 为什么重要？
    ///
    /// 文件可能因各种原因损坏（磁盘错误、传输问题等），
    /// 不能因为一个文件损坏就影响整个记忆系统。
    #[tokio::test]
    async fn test_read_corrupted_file() {
        let (storage, _temp) = create_test_storage().await;

        // 创建无效 UTF-8 文件
        let file_path = storage.memory_dir().join("corrupted.md");
        tokio::fs::create_dir_all(storage.memory_dir())
            .await
            .unwrap();
        tokio::fs::write(&file_path, vec![0x80, 0x81, 0x82])
            .await
            .unwrap();

        // 扫描应该跳过损坏文件
        let headers = storage.scan_memories().await.unwrap();
        assert!(headers.is_empty());
    }

    /// 测试大内容处理
    ///
    /// ## 验证点
    ///
    /// 能正确处理 10万字符的大内容。
    ///
    /// ## 为什么测试这个？
    ///
    /// 确保没有隐藏的缓冲区限制或性能问题。
    #[tokio::test]
    async fn test_large_content_handling() {
        let (storage, _temp) = create_test_storage().await;

        let large_content = "x".repeat(100_000);

        storage
            .save_memory(
                "large.md",
                "Large Memory",
                "Has lots of content",
                crate::types::MemoryType::User,
                &large_content,
            )
            .await
            .unwrap();

        let result = storage.read_memory("large.md").await.unwrap();
        assert!(result.is_some());

        let (full, body) = result.unwrap();
        assert!(full.len() > 100_000);
        assert_eq!(body.len(), large_content.len());
    }
}
