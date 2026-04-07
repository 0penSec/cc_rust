//! 记忆系统提示词构建模块
//!
//! 这个模块负责生成指导 AI 如何使用记忆系统的系统提示词。
//!
//! ## 为什么需要这个模块？
//!
//! AI 需要知道：
//! 1. 记忆系统是什么
//! 2. 有哪些类型的记忆
//! 3. 什么时候该保存记忆
//! 4. 什么时候该使用记忆
//! 5. 如何格式化记忆文件
//!
//! 这个模块把这些知识编码成系统提示词，
//! 插入到 AI 的上下文中。
//!
//! ## 提示词结构
//!
//! ```markdown
//! # Auto Memory
//!
//! You have a persistent memory system at `/path/to/memory`...
//!
//! ## Types of memory
//!
//! ### user
//! When you learn any details about the user's role...
//!
//! ### feedback
//! ...
//!
//! ## What NOT to save
//!
//! ...
//!
//! ## How to save memories
//!
//! ...
//!
//! ## MEMORY.md
//!
//! {索引文件内容}
//! ```

use crate::storage::MemoryStorage;
use crate::types::MemoryType;

/// 构建记忆系统的提示词
///
/// ## 这个提示词的作用
///
/// 1. **介绍记忆系统**: 告诉 AI 有持久化存储可用
/// 2. **指导保存时机**: 什么情况下应该保存记忆
/// 3. **指导使用时机**: 什么情况下应该参考记忆
/// 4. **提供当前索引**: 把 MEMORY.md 的内容附在后面
///
/// ## 参数
///
/// - `storage`: 存储管理器（用于读取 MEMORY.md）
///
/// ## 返回值
///
/// 完整的提示词字符串，可以直接插入到系统提示词中。
///
/// ## 使用示例
///
/// ```rust,no_run
/// use claude_memory::{MemoryStorage, MemoryConfig, build_memory_prompt};
///
/// # async fn example() -> std::io::Result<()> {
/// let storage = MemoryStorage::new(MemoryConfig::default(), "my-project");
/// let prompt = build_memory_prompt(&storage).await?;
///
/// // 把 prompt 添加到系统提示词中
/// println!("{}", prompt);
/// # Ok(())
/// # }
/// ```
pub async fn build_memory_prompt(
    storage: &MemoryStorage,
) -> std::io::Result<String> {
    let memory_dir = storage.memory_dir();
    let entrypoint = storage.read_entrypoint().await?;

    // 构建提示词的各个部分
    let mut lines = vec![
        "# Auto Memory".to_string(),
        "".to_string(),
        format!(
            "You have a persistent, file-based memory system at `{}`. This directory already exists — write to it directly.",
            memory_dir.display()
        ),
        "".to_string(),
        "You should build up this memory system over time so that future conversations can have a complete picture of who the user is, how they'd like to collaborate with you, what behaviors to avoid or repeat, and the context behind the work the user gives you.".to_string(),
        "".to_string(),
    ];

    // 添加记忆类型说明
    lines.extend(build_types_section());
    lines.push("".to_string());

    // 添加"不要保存什么"的说明
    lines.extend(build_what_not_to_save_section());
    lines.push("".to_string());

    // 添加"如何保存"的说明
    lines.extend(build_how_to_save_section());
    lines.push("".to_string());

    // 添加"什么时候访问"的说明
    lines.extend(build_when_to_access_section());
    lines.push("".to_string());

    // 添加 MEMORY.md 内容
    if let Some(content) = entrypoint {
        lines.push("## MEMORY.md".to_string());
        lines.push("".to_string());
        lines.push(content);
    } else {
        lines.push("## MEMORY.md".to_string());
        lines.push("".to_string());
        lines.push("Your MEMORY.md is currently empty. When you save new memories, they will appear here.".to_string());
    }

    Ok(lines.join("\n"))
}

/// 构建记忆类型说明部分
///
/// ## 为什么需要这个部分？
///
/// AI 需要知道四种记忆类型的区别，
/// 才能正确地分类保存记忆。
///
/// ## 内容
///
/// 对每种类型（user, feedback, project, reference）：
/// 1. 类型名称
/// 2. 什么时候保存（调用 `when_to_save()`）
fn build_types_section() -> Vec<String> {
    vec![
        "## Types of memory".to_string(),
        "".to_string(),
        "There are several discrete types of memory that you can store in your memory system:".to_string(),
        "".to_string(),
        format!("### user\n\n{}", MemoryType::User.when_to_save()),
        "".to_string(),
        format!("### feedback\n\n{}", MemoryType::Feedback.when_to_save()),
        "".to_string(),
        format!("### project\n\n{}", MemoryType::Project.when_to_save()),
        "".to_string(),
        format!("### reference\n\n{}", MemoryType::Reference.when_to_save()),
    ]
}

/// 构建"不要保存什么"的说明
///
/// ## 为什么需要这个部分？
///
/// 防止 AI 保存无用的信息，比如：
/// - 代码内容（可以从文件读取）
/// - git 历史（可以用 git 命令）
/// - 临时信息（只与当前对话相关）
///
/// ## 目的
///
/// 保持记忆系统精简，只保存真正有价值的信息。
fn build_what_not_to_save_section() -> Vec<String> {
    vec![
        "## What NOT to save in memory".to_string(),
        "".to_string(),
        "- Code patterns, conventions, architecture, file paths, or project structure — these can be derived by reading the current project state.".to_string(),
        "- Git history, recent changes, or who-changed-what — `git log` / `git blame` are authoritative.".to_string(),
        "- Debugging solutions or fix recipes — the fix is in the code; the commit message has the context.".to_string(),
        "- Anything already documented in CLAUDE.md files.".to_string(),
        "- Ephemeral task details: in-progress work, temporary state, current conversation context.".to_string(),
        "".to_string(),
        "These exclusions apply even when the user explicitly asks you to save. If they ask you to save a PR list or activity summary, ask what was *surprising* or *non-obvious* about it — that is the part worth keeping.".to_string(),
    ]
}

/// 构建"如何保存记忆"的说明
///
/// ## 为什么需要这个部分？
///
/// AI 需要知道正确的文件格式，才能创建有效的记忆文件。
///
/// ## 关键要点
///
/// 1. **两步保存法**:
///    - 步骤1：写入独立文件（带 frontmatter）
///    - 步骤2：在 MEMORY.md 中添加索引
///
/// 2. **格式要求**:
///    - 必须包含 name, description, type
///    - 使用 YAML frontmatter
///
/// 3. **索引规则**:
///    - MEMORY.md 只放一行索引
///    - 不要写完整内容到索引
fn build_how_to_save_section() -> Vec<String> {
    vec![
        "## How to save memories".to_string(),
        "".to_string(),
        "Saving a memory is a two-step process:".to_string(),
        "".to_string(),
        "**Step 1** — write the memory to its own file (e.g., `user_role.md`, `feedback_testing.md`) using this frontmatter format:".to_string(),
        "".to_string(),
        "```markdown".to_string(),
        "---".to_string(),
        "name: {{memory name}}".to_string(),
        "description: {{one-line description}}".to_string(),
        "type: {{user, feedback, project, reference}}".to_string(),
        "---".to_string(),
        "".to_string(),
        "{{memory content — for feedback/project types, structure as: rule/fact, then **Why:** and **How to apply:** lines}}".to_string(),
        "```".to_string(),
        "".to_string(),
        "**Step 2** — add a pointer to that file in `MEMORY.md`. `MEMORY.md` is an index, not a memory — each entry should be one line, under ~150 characters: `- [Title](file.md) — one-line hook`. It has no frontmatter. Never write memory content directly into `MEMORY.md`.".to_string(),
        "".to_string(),
        "- `MEMORY.md` is always loaded into your conversation context — lines after 200 will be truncated, so keep the index concise".to_string(),
        "- Keep the name, description, and type fields in memory files up-to-date with the content".to_string(),
        "- Organize memory semantically by topic, not chronologically".to_string(),
        "- Update or remove memories that turn out to be wrong or outdated".to_string(),
        "- Do not write duplicate memories. First check if there is an existing memory you can update before writing a new one.".to_string(),
    ]
}

/// 构建"什么时候访问记忆"的说明
///
/// ## 为什么需要这个部分？
///
/// 指导 AI 正确使用记忆：
/// 1. 什么时候主动查看记忆
/// 2. 如何处理用户的记忆相关指令
/// 3. 如何验证记忆的时效性
fn build_when_to_access_section() -> Vec<String> {
    vec![
        "## When to access memories".to_string(),
        "".to_string(),
        "- When memories seem relevant, or the user references prior-conversation work.".to_string(),
        "- You MUST access memory when the user explicitly asks you to check, recall, or remember.".to_string(),
        "- If the user says to *ignore* or *not use* memory: proceed as if MEMORY.md were empty. Do not apply remembered facts, cite, compare against, or mention memory content.".to_string(),
        "- Memory records can become stale over time. Use memory as context for what was true at a given point in time. Before answering the user or building assumptions based solely on information in memory records, verify that the memory is still correct and up-to-date by reading the current state of the files or resources. If a recalled memory conflicts with current information, trust what you observe now — and update or remove the stale memory rather than acting on it.".to_string(),
    ]
}

/// 测试模块
///
/// ## 测试内容
///
/// 主要验证 build_memory_prompt 能正确生成包含各个部分的提示词。
#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::MemoryConfig;
    use tempfile::TempDir;

    /// 测试提示词构建
    ///
    /// ## 验证点
    ///
    /// 1. 包含 "Auto Memory" 标题
    /// 2. 包含四种记忆类型
    /// 3. 包含 MEMORY.md 章节
    #[tokio::test]
    async fn test_build_memory_prompt() {
        let temp_dir = TempDir::new().unwrap();
        let config = MemoryConfig {
            base_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };
        let storage = MemoryStorage::new(config, "test-project");

        let prompt = build_memory_prompt(&storage).await.unwrap();

        assert!(prompt.contains("Auto Memory"));
        assert!(prompt.contains("Types of memory"));
        assert!(prompt.contains("user"));
        assert!(prompt.contains("feedback"));
        assert!(prompt.contains("MEMORY.md"));
    }
}
