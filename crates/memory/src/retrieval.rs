//! 记忆检索和相关性评分模块
//!
//! 这个模块提供了从记忆中搜索相关信息的功能。
//!
//! ## 核心问题：如何找到相关的记忆？
//!
//! 当用户说"按照我之前说的方式做"时，AI需要找到正确的记忆。
//! 这个模块就是解决这个问题的。
//!
//! ## 当前实现：关键词匹配
//!
//! 这是一个简单但有效的方案：
//! 1. 把查询拆分成关键词
//! 2. 在文件名、描述、类型中查找这些词
//! 3. 根据匹配位置给分（描述匹配分数最高）
//! 4. 按分数排序，返回最相关的
//!
//! ## 为什么不用向量搜索？
//!
//! 向量搜索（embeddings）确实更智能，但需要：
//! 1. 额外的依赖（如 onnxruntime）
//! 2. 预计算和存储向量
//! 3. 更多的内存和CPU
//!
//! 对于个人项目的记忆数量（通常<100条），
//! 关键词匹配已经足够好，且更简单可靠。
//!
//! ## 未来改进方向
//!
//! 如果记忆数量增长到上千条，可以考虑：
//! - 使用向量数据库（如 qdrant、chromadb）
//! - 本地 embedding 模型（如 all-MiniLM）
//! - 混合搜索（关键词 + 向量）

use std::collections::HashSet;

use crate::storage::MemoryStorage;
use crate::types::MemoryHeader;

/// 检索到的记忆结果
///
/// 包含记忆的所有信息，以及相关性分数。
#[derive(Debug, Clone)]
pub struct RetrievedMemory {
    /// 记忆头部信息（文件名、描述等）
    pub header: MemoryHeader,
    /// 相关性分数（0.0 - 1.0）
    /// 分数越高表示越相关
    pub score: f64,
    /// 记忆的完整内容（包含 frontmatter）
    pub content: String,
}

/// 基于关键词的记忆检索器
///
/// 这是一个无状态的结构体，所有方法都是纯函数。
/// 这样设计是为了简单和线程安全。
pub struct MemoryRetriever;

impl MemoryRetriever {
    /// 查找与查询相关的记忆
    ///
    /// ## 参数
    ///
    /// - `storage`: 存储管理器
    /// - `query`: 查询字符串（如 "rust 经验"）
    /// - `limit`: 最多返回多少条结果
    ///
    /// ## 返回值
    ///
    /// 返回按相关性排序的记忆列表，最相关的在前。
    ///
    /// ## 评分机制
    ///
    /// 每个关键词匹配的位置有不同权重：
    /// - 描述匹配: +0.5（最准确）
    /// - 文件名匹配: +0.3（中等）
    /// - 类型匹配: +0.2（较宽泛）
    ///
    /// ## 为什么这样设计？
    ///
    /// 1. **描述匹配权重最高**：描述是用户写的，最准确
    /// 2. **文件名次之**：文件名通常包含关键词
    /// 3. **类型最低**："user"类型包含很多记忆，太宽泛
    ///
    /// ## 使用示例
    ///
    /// ```rust,no_run
    /// use claude_memory::{MemoryStorage, MemoryConfig, MemoryRetriever};
    ///
    /// # async fn example() -> std::io::Result<()> {
    /// let storage = MemoryStorage::new(MemoryConfig::default(), "my-project");
    ///
    /// // 搜索与 "rust" 相关的记忆
    /// let results = MemoryRetriever::find_relevant(&storage, "rust", 5).await?;
    ///
    /// for mem in results {
    ///     println!("{} ({}%)", mem.header.filename, mem.score * 100.0);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn find_relevant(
        storage: &MemoryStorage,
        query: &str,
        limit: usize,
    ) -> std::io::Result<Vec<RetrievedMemory>> {
        // 1. 扫描所有记忆头部信息
        let headers = storage.scan_memories().await?;

        // 2. 把查询拆分成关键词（小写，去重）
        let query_words: HashSet<String> = query
            .to_lowercase()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        let mut scored: Vec<(MemoryHeader, f64)> = Vec::new();

        // 3. 为每个记忆计算相关性分数
        for header in headers {
            let mut score = 0.0;

            // 检查文件名匹配
            let filename_lower = header.filename.to_lowercase();
            for word in &query_words {
                if filename_lower.contains(word) {
                    score += 0.3;
                }
            }

            // 检查描述匹配（权重最高）
            if let Some(desc) = &header.description {
                let desc_lower = desc.to_lowercase();
                for word in &query_words {
                    if desc_lower.contains(word) {
                        score += 0.5;
                    }
                }
            }

            // 检查类型匹配
            if let Some(mem_type) = &header.memory_type {
                let type_str = mem_type.as_str().to_lowercase();
                for word in &query_words {
                    if type_str.contains(word) {
                        score += 0.2;
                    }
                }
            }

            // 只保留有匹配的记忆
            if score > 0.0 {
                scored.push((header, score));
            }
        }

        // 4. 按分数降序排列
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // 5. 获取前 N 个的完整内容
        let mut results = Vec::new();
        for (header, score) in scored.into_iter().take(limit) {
            let content = storage
                .read_memory(&header.filename)
                .await?
                .map(|(full, _)| full)
                .unwrap_or_default();

            results.push(RetrievedMemory {
                header,
                score,
                content,
            });
        }

        Ok(results)
    }

    /// 将记忆格式化为提示词上下文
    ///
    /// ## 为什么需要这个方法？
    ///
    /// 检索到的记忆需要格式化成文本，才能插入到系统提示词中。
    /// 统一的格式让 AI 更容易理解。
    ///
    /// ## 输出格式
    ///
    /// ```markdown
    /// ## Relevant Memories
    ///
    /// ### filename.md (85%)
    ///
    /// {记忆的完整内容}
    ///
    /// ---
    ///
    /// ### another.md (60%)
    ///
    /// {内容}
    ///
    /// ---
    /// ```
    ///
    /// ## 参数
    ///
    /// - `memories`: 检索到的记忆列表
    ///
    /// ## 返回值
    ///
    /// 格式化后的 Markdown 字符串。如果输入为空，返回空字符串。
    pub fn format_memories_for_prompt(memories: &[RetrievedMemory]) -> String {
        if memories.is_empty() {
            return String::new();
        }

        let mut output = String::from("## Relevant Memories\n\n");

        for mem in memories {
            output.push_str(&format!(
                "### {} ({}%)\n\n{}",
                mem.header.filename,
                (mem.score * 100.0) as u32,
                mem.content
            ));
            output.push_str("\n---\n\n");
        }

        output
    }

    /// 获取所有记忆的摘要（用于系统提示词）
    ///
    /// ## 与 find_relevant 的区别
    ///
    /// - `find_relevant`: 智能搜索，返回最相关的
    /// - `get_all_memories_summary`: 列出所有，用于给 AI 概览
    ///
    /// ## 输出格式
    ///
    /// ```markdown
    /// ## Available Memories
    ///
    /// - [user] user_role.md — 用户是资深工程师
    /// - [reference] api_docs.md — API 文档链接
    /// ```
    ///
    /// ## 为什么需要这个方法？
    ///
    /// 在系统提示词中列出所有可用的记忆，
    /// 让 AI 知道有哪些记忆可以参考。
    pub async fn get_all_memories_summary(storage: &MemoryStorage) -> std::io::Result<String> {
        let headers = storage.scan_memories().await?;

        // 没有记忆时返回提示
        if headers.is_empty() {
            return Ok("No memories stored yet.".to_string());
        }

        let mut output = String::from("## Available Memories\n\n");

        for header in headers {
            // 类型标签，如 "[user] "
            let type_tag = header
                .memory_type
                .as_ref()
                .map(|t| format!("[{}] ", t.as_str()))
                .unwrap_or_default();

            // 描述，如 " — 用户是资深工程师"
            let desc = header
                .description
                .as_ref()
                .map(|d| format!(" — {}", d))
                .unwrap_or_default();

            output.push_str(&format!("- {}{}{}\n", type_tag, header.filename, desc));
        }

        Ok(output)
    }
}

/// 测试模块
///
/// ## 测试策略
///
/// 1. **基本搜索**: 验证能找到相关记忆
/// 2. **按字段搜索**: 验证文件名、描述、类型都能被搜索
/// 3. **空结果**: 验证无匹配时返回空
/// 4. **数量限制**: 验证 limit 参数有效
/// 5. **格式化**: 验证输出格式正确
#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{MemoryConfig, MemoryType};
    use tempfile::TempDir;

    /// 测试基本的相关性搜索
    ///
    /// ## 验证点
    ///
    /// 1. 能找到与查询相关的记忆
    /// 2. 最相关的排在前面
    /// 3. 返回的记忆包含文件名、分数、内容
    #[tokio::test]
    async fn test_find_relevant() {
        let temp_dir = TempDir::new().unwrap();
        let config = MemoryConfig {
            base_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };
        let storage = MemoryStorage::new(config, "test-project");

        // 保存测试记忆
        storage
            .save_memory(
                "rust_expertise.md",
                "Rust Expertise",
                "User has 10 years Rust experience",
                MemoryType::User,
                "Detailed content about Rust",
            )
            .await
            .unwrap();

        storage
            .save_memory(
                "frontend_feedback.md",
                "Frontend Feedback",
                "Prefers React over Vue",
                MemoryType::Feedback,
                "Detailed feedback content",
            )
            .await
            .unwrap();

        // 搜索与 Rust 相关的记忆
        let results = MemoryRetriever::find_relevant(&storage, "rust experience", 5)
            .await
            .unwrap();

        assert!(!results.is_empty());
        assert!(results[0].header.filename.contains("rust"));
    }

    /// 测试格式化输出
    ///
    /// ## 验证点
    ///
    /// 1. 包含文件名
    /// 2. 包含分数百分比
    /// 3. 包含分隔线
    #[test]
    fn test_format_memories() {
        let memories = vec![RetrievedMemory {
            header: MemoryHeader {
                filename: "test.md".to_string(),
                file_path: std::path::PathBuf::from("/test.md"),
                mtime_ms: 0,
                description: Some("Test".to_string()),
                memory_type: Some(MemoryType::User),
            },
            score: 0.9,
            content: "Test content".to_string(),
        }];

        let formatted = MemoryRetriever::format_memories_for_prompt(&memories);
        assert!(formatted.contains("test.md"));
        assert!(formatted.contains("90%"));
    }

    /// 测试按描述搜索
    ///
    /// ## 验证点
    ///
    /// 描述中的关键词能匹配，且权重最高。
    #[tokio::test]
    async fn test_find_relevant_by_description() {
        let temp_dir = TempDir::new().unwrap();
        let config = MemoryConfig {
            base_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };
        let storage = MemoryStorage::new(config, "test-project");

        // 保存记忆
        storage
            .save_memory(
                "python_guide.md",
                "Python Guide",
                "User prefers Python for scripting",
                MemoryType::User,
                "Content",
            )
            .await
            .unwrap();

        storage
            .save_memory(
                "rust_guide.md",
                "Rust Guide",
                "User prefers Rust for systems",
                MemoryType::User,
                "Content",
            )
            .await
            .unwrap();

        // 搜索 "python" - 应该找到 python_guide
        let results = MemoryRetriever::find_relevant(&storage, "python", 5)
            .await
            .unwrap();
        assert!(!results.is_empty());
        assert!(results[0].header.filename.contains("python"));
    }

    /// 测试按类型搜索
    ///
    /// ## 验证点
    ///
    /// 类型名（如 "feedback"）能作为搜索词匹配。
    #[tokio::test]
    async fn test_find_relevant_by_type() {
        let temp_dir = TempDir::new().unwrap();
        let config = MemoryConfig {
            base_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };
        let storage = MemoryStorage::new(config, "test-project");

        storage
            .save_memory(
                "feedback_1.md",
                "Feedback",
                "Some feedback",
                MemoryType::Feedback,
                "Content",
            )
            .await
            .unwrap();

        // 按类型名搜索
        let results = MemoryRetriever::find_relevant(&storage, "feedback", 5)
            .await
            .unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].header.memory_type, Some(MemoryType::Feedback));
    }

    /// 测试无匹配的情况
    ///
    /// ## 验证点
    ///
    /// 没有匹配时返回空列表，不是错误。
    #[tokio::test]
    async fn test_find_relevant_no_matches() {
        let temp_dir = TempDir::new().unwrap();
        let config = MemoryConfig {
            base_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };
        let storage = MemoryStorage::new(config, "test-project");

        storage
            .save_memory(
                "rust.md",
                "Rust",
                "Rust content",
                MemoryType::User,
                "Content",
            )
            .await
            .unwrap();

        // 搜索不相关的词
        let results = MemoryRetriever::find_relevant(&storage, "python javascript java", 5)
            .await
            .unwrap();
        assert!(results.is_empty());
    }

    /// 测试返回数量限制
    ///
    /// ## 验证点
    ///
    /// limit 参数能正确限制返回数量。
    #[tokio::test]
    async fn test_find_relevant_limit() {
        let temp_dir = TempDir::new().unwrap();
        let config = MemoryConfig {
            base_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };
        let storage = MemoryStorage::new(config, "test-project");

        // 保存 5 个记忆
        for i in 0..5 {
            storage
                .save_memory(
                    &format!("memory_{}.md", i),
                    &format!("Memory {}", i),
                    "Test description with rust keyword",
                    MemoryType::User,
                    "Content",
                )
                .await
                .unwrap();
        }

        let results = MemoryRetriever::find_relevant(&storage, "rust", 3)
            .await
            .unwrap();
        assert_eq!(results.len(), 3);
    }

    /// 测试获取所有记忆摘要
    ///
    /// ## 验证点
    ///
    /// 1. 包含所有记忆的文件名
    /// 2. 包含类型标签
    /// 3. 包含描述
    #[tokio::test]
    async fn test_get_all_memories_summary() {
        let temp_dir = TempDir::new().unwrap();
        let config = MemoryConfig {
            base_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };
        let storage = MemoryStorage::new(config, "test-project");

        storage
            .save_memory(
                "user_pref.md",
                "User Preferences",
                "User likes dark mode",
                MemoryType::User,
                "Content",
            )
            .await
            .unwrap();

        storage
            .save_memory(
                "api_ref.md",
                "API Reference",
                "Internal API docs",
                MemoryType::Reference,
                "Content",
            )
            .await
            .unwrap();

        let summary = MemoryRetriever::get_all_memories_summary(&storage)
            .await
            .unwrap();
        assert!(summary.contains("user_pref.md"));
        assert!(summary.contains("api_ref.md"));
        assert!(summary.contains("[user]"));
        assert!(summary.contains("[reference]"));
        assert!(summary.contains("User likes dark mode"));
        assert!(summary.contains("Internal API docs"));
    }

    /// 测试空记忆摘要
    ///
    /// ## 验证点
    ///
    /// 没有记忆时返回特定提示。
    #[tokio::test]
    async fn test_get_all_memories_summary_empty() {
        let temp_dir = TempDir::new().unwrap();
        let config = MemoryConfig {
            base_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };
        let storage = MemoryStorage::new(config, "test-project");

        let summary = MemoryRetriever::get_all_memories_summary(&storage)
            .await
            .unwrap();
        assert_eq!(summary, "No memories stored yet.");
    }

    /// 测试格式化空记忆列表
    ///
    /// ## 验证点
    ///
    /// 输入为空时返回空字符串。
    #[test]
    fn test_format_empty_memories() {
        let formatted = MemoryRetriever::format_memories_for_prompt(&[]);
        assert!(formatted.is_empty());
    }
}
