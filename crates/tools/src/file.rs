use async_trait::async_trait;
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use tracing::debug;

use claude_core::{
    ClaudeResult, PermissionMode, Tool, ToolContext, ToolInput, ToolOutput, ToolResult,
};

/// File read tool
pub struct FileReadTool;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct FileReadInput {
    /// Absolute or relative path to the file
    file_path: String,
    /// Offset to start reading from (0-indexed line number)
    #[serde(default)]
    offset: Option<usize>,
    /// Maximum number of lines to read
    #[serde(default)]
    limit: Option<usize>,
}

#[async_trait]
impl Tool for FileReadTool {
    fn name(&self) -> &str {
        "file_read"
    }

    fn description(&self) -> &str {
        "Read the contents of a file. Returns the file content as text. \
         Supports optional offset and limit for reading partial content."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::to_value(schema_for!(FileReadInput)).unwrap()
    }

    fn permission_mode(&self) -> PermissionMode {
        PermissionMode::Allow
    }

    async fn execute(&self, input: ToolInput, ctx: &ToolContext) -> ToolResult {
        let input: FileReadInput = input.parse()?;
        let path = resolve_path(&input.file_path, &ctx.working_directory)?;

        debug!("Reading file: {:?}", path);

        // Check if file exists
        if !path.exists() {
            return Ok(ToolOutput::error(format!(
                "File not found: {}",
                path.display()
            )));
        }

        // Check if it's a directory
        if path.is_dir() {
            return Ok(ToolOutput::error(format!(
                "Path is a directory, not a file: {}",
                path.display()
            )));
        }

        // Read file content
        let content = fs::read_to_string(&path)
            .await
            .map_err(|e| claude_core::ClaudeError::Io(format!("Failed to read file: {}", e)))?;

        // Apply offset and limit if specified
        let lines: Vec<&str> = content.lines().collect();
        let offset = input.offset.unwrap_or(0);
        let limit = input.limit.unwrap_or(lines.len());

        let selected_lines: Vec<&str> = lines.into_iter().skip(offset).take(limit).collect();

        Ok(ToolOutput::success(selected_lines.join("\n")))
    }
}

/// File write tool
pub struct FileWriteTool;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct FileWriteInput {
    /// Absolute or relative path to the file
    file_path: String,
    /// Content to write to the file
    content: String,
}

#[async_trait]
impl Tool for FileWriteTool {
    fn name(&self) -> &str {
        "file_write"
    }

    fn description(&self) -> &str {
        "Create a new file or completely overwrite an existing file with the given content. \
         Use with caution as this will replace any existing content."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::to_value(schema_for!(FileWriteInput)).unwrap()
    }

    fn permission_mode(&self) -> PermissionMode {
        PermissionMode::Ask
    }

    async fn execute(&self, input: ToolInput, ctx: &ToolContext) -> ToolResult {
        let input: FileWriteInput = input.parse()?;
        let path = resolve_path(&input.file_path, &ctx.working_directory)?;

        debug!("Writing file: {:?}", path);

        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                claude_core::ClaudeError::Io(format!("Failed to create directories: {}", e))
            })?;
        }

        // Write file
        fs::write(&path, input.content)
            .await
            .map_err(|e| claude_core::ClaudeError::Io(format!("Failed to write file: {}", e)))?;

        Ok(ToolOutput::success(format!(
            "Successfully wrote to {}",
            path.display()
        )))
    }
}

/// File edit tool - performs string replacement
pub struct FileEditTool;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct FileEditInput {
    /// Absolute or relative path to the file
    file_path: String,
    /// String to find (will be replaced)
    old_string: String,
    /// String to replace with
    new_string: String,
}

#[async_trait]
impl Tool for FileEditTool {
    fn name(&self) -> &str {
        "file_edit"
    }

    fn description(&self) -> &str {
        "Edit a file by replacing a specific string with another string. \
         The old_string must match exactly once in the file."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::to_value(schema_for!(FileEditInput)).unwrap()
    }

    fn permission_mode(&self) -> PermissionMode {
        PermissionMode::Ask
    }

    async fn execute(&self, input: ToolInput, ctx: &ToolContext) -> ToolResult {
        let input: FileEditInput = input.parse()?;
        let path = resolve_path(&input.file_path, &ctx.working_directory)?;

        debug!("Editing file: {:?}", path);

        // Read current content
        let content = fs::read_to_string(&path)
            .await
            .map_err(|e| claude_core::ClaudeError::Io(format!("Failed to read file: {}", e)))?;

        // Count occurrences
        let occurrences = content.matches(&input.old_string).count();
        if occurrences == 0 {
            return Ok(ToolOutput::error(format!(
                "String not found in file: {}",
                input.old_string
            )));
        }
        if occurrences > 1 {
            return Ok(ToolOutput::error(format!(
                "String found {} times in file, must match exactly once",
                occurrences
            )));
        }

        // Perform replacement
        let new_content = content.replacen(&input.old_string, &input.new_string, 1);

        // Write back
        fs::write(&path, new_content)
            .await
            .map_err(|e| claude_core::ClaudeError::Io(format!("Failed to write file: {}", e)))?;

        Ok(ToolOutput::success(format!(
            "Successfully edited {}",
            path.display()
        )))
    }
}

/// Resolve a path relative to working directory
fn resolve_path(path: &str, working_dir: &std::path::Path) -> ClaudeResult<PathBuf> {
    let path = PathBuf::from(path);
    if path.is_absolute() {
        Ok(path)
    } else {
        Ok(working_dir.join(path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_file_read() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "Hello World").await.unwrap();

        let tool = FileReadTool;
        let ctx = ToolContext {
            session_id: Default::default(),
            working_directory: temp_dir.path().to_path_buf(),
            env_vars: std::collections::HashMap::new(),
        };
        let input = ToolInput::new(json!({
            "file_path": "test.txt"
        }));

        let result = tool.execute(input, &ctx).await.unwrap();
        assert!(!result.is_error);
        assert!(result.content.contains("Hello World"));
    }

    #[tokio::test]
    async fn test_file_write() {
        let temp_dir = TempDir::new().unwrap();

        let tool = FileWriteTool;
        let ctx = ToolContext {
            session_id: Default::default(),
            working_directory: temp_dir.path().to_path_buf(),
            env_vars: std::collections::HashMap::new(),
        };
        let input = ToolInput::new(json!({
            "file_path": "output.txt",
            "content": "Test content"
        }));

        let result = tool.execute(input, &ctx).await.unwrap();
        assert!(!result.is_error);

        let content = fs::read_to_string(temp_dir.path().join("output.txt"))
            .await
            .unwrap();
        assert_eq!(content, "Test content");
    }

    #[tokio::test]
    async fn test_file_edit() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("edit.txt");
        fs::write(&file_path, "Hello Old World").await.unwrap();

        let tool = FileEditTool;
        let ctx = ToolContext {
            session_id: Default::default(),
            working_directory: temp_dir.path().to_path_buf(),
            env_vars: std::collections::HashMap::new(),
        };
        let input = ToolInput::new(json!({
            "file_path": "edit.txt",
            "old_string": "Old",
            "new_string": "New"
        }));

        let result = tool.execute(input, &ctx).await.unwrap();
        assert!(!result.is_error);

        let content = fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(content, "Hello New World");
    }
}
