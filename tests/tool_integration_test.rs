//! 工具集成测试
//!
//! 测试工具注册表和多个工具的协同工作

use claude_core::{SessionId, ToolContext};
use claude_tools::{BashTool, FileReadTool, FileWriteTool, GlobTool, GrepTool, ToolRegistry};
use std::collections::HashMap;
use tempfile::TempDir;

/// 创建测试上下文
fn create_test_context(temp_dir: &TempDir) -> ToolContext {
    ToolContext {
        session_id: SessionId::new(),
        working_directory: temp_dir.path().to_path_buf(),
        env_vars: HashMap::new(),
    }
}

/// 测试工具注册表的基本功能
#[test]
fn test_tool_registry_basic() {
    let mut registry = ToolRegistry::new();

    // 注册工具
    registry.register(Box::new(BashTool::new()));
    registry.register(Box::new(FileReadTool));
    registry.register(Box::new(FileWriteTool));

    // 验证工具列表
    let tools = registry.list();
    assert!(tools.contains(&"bash"));
    assert!(tools.contains(&"file_read"));
    assert!(tools.contains(&"file_write"));

    // 验证获取工具
    assert!(registry.get("bash").is_some());
    assert!(registry.get("file_read").is_some());
    assert!(registry.get("nonexistent").is_none());
}

/// 测试工具注册表获取定义
#[test]
fn test_tool_registry_definitions() {
    let mut registry = ToolRegistry::new();
    registry.register(Box::new(BashTool::new()));

    let definitions = registry.get_definitions();
    assert_eq!(definitions.len(), 1);

    let bash_def = &definitions[0];
    assert_eq!(bash_def.name, "bash");
    assert!(!bash_def.description.is_empty());
}

/// 测试默认注册表
#[test]
fn test_default_registry() {
    let registry = claude_tools::default_registry();

    // 验证包含所有默认工具
    let tools = registry.list();
    assert!(tools.contains(&"bash"));
    assert!(tools.contains(&"file_read"));
    assert!(tools.contains(&"file_write"));
    assert!(tools.contains(&"file_edit"));
    assert!(tools.contains(&"glob"));
    assert!(tools.contains(&"grep"));
    assert!(tools.contains(&"web_fetch"));
}

/// 测试多个工具协同工作：先写入文件，再读取，再搜索
#[tokio::test]
async fn test_tools_workflow_write_read_search() {
    let temp_dir = TempDir::new().unwrap();
    let ctx = create_test_context(&temp_dir);

    // 1. 使用 FileWriteTool 创建文件
    let write_tool = FileWriteTool;
    let write_input = claude_core::ToolInput::new(
        serde_json::json!({
            "file_path": "test.rs",
            "content": "fn main() { println!(\"Hello World\"); }"
        })
    );

    let write_result = write_tool.execute(write_input, &ctx).await.unwrap();
    assert!(!write_result.is_error);

    // 2. 使用 FileReadTool 读取文件
    let read_tool = FileReadTool;
    let read_input = claude_core::ToolInput::new(
        serde_json::json!({"file_path": "test.rs"})
    );

    let read_result = read_tool.execute(read_input, &ctx).await.unwrap();
    assert!(!read_result.is_error);
    assert!(read_result.content.contains("Hello World"));

    // 3. 使用 GrepTool 搜索内容
    let grep_tool = GrepTool;
    let grep_input = claude_core::ToolInput::new(
        serde_json::json!({
            "pattern": "println!",
            "path": "."
        })
    );

    let grep_result = grep_tool.execute(grep_input, &ctx).await.unwrap();
    assert!(!grep_result.is_error);
    assert!(grep_result.content.contains("println!"));
}

/// 测试 Glob + Grep 组合：先找到文件，再搜索内容
#[tokio::test]
async fn test_glob_and_grep_combination() {
    let temp_dir = TempDir::new().unwrap();
    let ctx = create_test_context(&temp_dir);

    // 创建多个文件
    tokio::fs::write(temp_dir.path().join("src/main.rs"), "fn main() {}")
        .await
        .unwrap();
    tokio::fs::write(temp_dir.path().join("src/lib.rs"), "pub fn add() {}")
        .await
        .unwrap();
    tokio::fs::write(temp_dir.path().join("README.md"), "# Project")
        .await
        .unwrap();

    // 1. 使用 Glob 找到所有 .rs 文件
    let glob_tool = GlobTool;
    let glob_input = claude_core::ToolInput::new(
        serde_json::json!({"pattern": "**/*.rs"})
    );

    let glob_result = glob_tool.execute(glob_input, &ctx).await.unwrap();
    assert!(!glob_result.is_error);
    assert!(glob_result.content.contains("main.rs"));
    assert!(glob_result.content.contains("lib.rs"));
    assert!(!glob_result.content.contains("README.md"));

    // 2. 使用 Grep 搜索所有文件中的 "fn"
    let grep_tool = GrepTool;
    let grep_input = claude_core::ToolInput::new(
        serde_json::json!({
            "pattern": "^fn |^pub fn",
            "path": "."
        })
    );

    let grep_result = grep_tool.execute(grep_input, &ctx).await.unwrap();
    assert!(!grep_result.is_error);
    // 应该找到 main.rs 和 lib.rs 中的函数定义
    let matches: Vec<&str> = grep_result.content.lines().collect();
    assert!(matches.len() >= 2);
}

/// 测试 Bash 和 File 工具组合：执行命令创建文件，然后读取
#[tokio::test]
async fn test_bash_and_file_combination() {
    let temp_dir = TempDir::new().unwrap();
    let ctx = create_test_context(&temp_dir);

    // 使用 BashTool 创建文件
    let bash_tool = BashTool::new();
    let bash_input = claude_core::ToolInput::new(
        serde_json::json!({
            "command": "echo 'Created by bash' > bash_output.txt"
        })
    );

    let bash_result = bash_tool.execute(bash_input, &ctx).await.unwrap();
    assert!(!bash_result.is_error);

    // 使用 FileReadTool 读取
    let read_tool = FileReadTool;
    let read_input = claude_core::ToolInput::new(
        serde_json::json!({"file_path": "bash_output.txt"})
    );

    let read_result = read_tool.execute(read_input, &ctx).await.unwrap();
    assert!(!read_result.is_error);
    assert!(read_result.content.contains("Created by bash"));
}

/// 测试错误处理：读取不存在的文件
#[tokio::test]
async fn test_file_read_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let ctx = create_test_context(&temp_dir);

    let read_tool = FileReadTool;
    let read_input = claude_core::ToolInput::new(
        serde_json::json!({"file_path": "nonexistent.txt"})
    );

    let result = read_tool.execute(read_input, &ctx).await;
    // 应该返回错误
    assert!(result.is_err() || result.unwrap().is_error);
}

/// 测试错误处理：执行无效命令
#[tokio::test]
async fn test_bash_invalid_command() {
    let temp_dir = TempDir::new().unwrap();
    let ctx = create_test_context(&temp_dir);

    let bash_tool = BashTool::new();
    let bash_input = claude_core::ToolInput::new(
        serde_json::json!({"command": "this_command_does_not_exist_12345"})
    );

    let result = bash_tool.execute(bash_input, &ctx).await;
    assert!(result.is_ok()); // 工具执行成功
    let output = result.unwrap();
    assert!(output.is_error); // 但结果标记为错误
}

/// 测试并发执行多个工具（读操作是并发的）
#[tokio::test]
async fn test_concurrent_tool_execution() {
    use futures::future::join_all;

    let temp_dir = TempDir::new().unwrap();
    let ctx = create_test_context(&temp_dir);

    // 创建多个文件
    for i in 0..5 {
        tokio::fs::write(
            temp_dir.path().join(format!("file{}.txt", i)),
            format!("Content {}", i)
        )
        .await
        .unwrap();
    }

    // 并发读取所有文件
    let read_tool = FileReadTool;
    let futures: Vec<_> = (0..5)
        .map(|i| {
            let input = claude_core::ToolInput::new(
                serde_json::json!({"file_path": format!("file{}.txt", i)})
            );
            read_tool.execute(input, &ctx)
        })
        .collect();

    let results = join_all(futures).await;

    // 验证所有读取都成功
    for (i, result) in results.iter().enumerate() {
        assert!(result.is_ok());
        let output = result.as_ref().unwrap();
        assert!(!output.is_error);
        assert!(output.content.contains(&format!("Content {}", i)));
    }
}
