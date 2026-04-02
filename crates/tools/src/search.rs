use async_trait::async_trait;
use ignore::WalkBuilder;
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::debug;

use claude_core::{
    Tool, ToolContext, ToolInput, ToolOutput, ToolResult, PermissionMode,
};

/// Glob tool for file pattern matching
pub struct GlobTool;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct GlobInput {
    /// Glob pattern to match (e.g., "**/*.rs", "src/**/*.ts")
    pattern: String,
    /// Working directory (defaults to current)
    #[serde(default)]
    path: Option<String>,
}

#[async_trait]
impl Tool for GlobTool {
    fn name(&self) -> &str {
        "glob"
    }

    fn description(&self) -> &str {
        "Find files matching a glob pattern. Returns a list of file paths. \
         Supports ** for recursive matching."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::to_value(schema_for!(GlobInput)).unwrap()
    }

    fn permission_mode(&self) -> PermissionMode {
        PermissionMode::Allow
    }

    async fn execute(&self, input: ToolInput, ctx: &ToolContext) -> ToolResult {
        let input: GlobInput = input.parse()?;
        let base_path = input
            .path
            .map(PathBuf::from)
            .unwrap_or_else(|| ctx.working_directory.clone());

        debug!("Glob pattern: {} in {:?}", input.pattern, base_path);

        let glob = match globset::Glob::new(&input.pattern) {
            Ok(g) => g.compile_matcher(),
            Err(e) => return Ok(ToolOutput::error(format!("Invalid glob pattern: {}", e))),
        };

        let mut matches = Vec::new();

        let walker = WalkBuilder::new(&base_path)
            .hidden(false)
            .git_ignore(true)
            .build();

        for entry in walker {
            match entry {
                Ok(entry) => {
                    let path = entry.path();
                    if path.is_file() {
                        let relative = path.strip_prefix(&base_path).unwrap_or(path);
                        if glob.is_match(relative) {
                            matches.push(path.to_string_lossy().to_string());
                        }
                    }
                }
                Err(e) => {
                    debug!("Error walking directory: {}", e);
                }
            }
        }

        matches.sort();
        Ok(ToolOutput::success(matches.join("\n")))
    }
}

/// Grep tool for content search
pub struct GrepTool;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct GrepInput {
    /// Regex pattern to search for
    pattern: String,
    /// Path to search in (defaults to current directory)
    #[serde(default)]
    path: Option<String>,
    /// File glob pattern to filter by (e.g., "*.rs")
    #[serde(default)]
    include: Option<String>,
    /// Case insensitive search
    #[serde(default)]
    ignore_case: bool,
}

#[derive(Debug, Serialize)]
struct GrepMatch {
    file_path: String,
    line_number: usize,
    column: usize,
    line_content: String,
}

#[async_trait]
impl Tool for GrepTool {
    fn name(&self) -> &str {
        "grep"
    }

    fn description(&self) -> &str {
        "Search file contents using regex. Returns matching lines with file paths, \
         line numbers, and content. Respects .gitignore."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::to_value(schema_for!(GrepInput)).unwrap()
    }

    fn permission_mode(&self) -> PermissionMode {
        PermissionMode::Allow
    }

    async fn execute(&self, input: ToolInput, ctx: &ToolContext) -> ToolResult {
        let input: GrepInput = input.parse()?;
        let base_path = input
            .path
            .map(PathBuf::from)
            .unwrap_or_else(|| ctx.working_directory.clone());

        debug!("Grep pattern: {} in {:?}", input.pattern, base_path);

        let regex = match regex::RegexBuilder::new(&input.pattern)
            .case_insensitive(input.ignore_case)
            .build()
        {
            Ok(r) => r,
            Err(e) => return Ok(ToolOutput::error(format!("Invalid regex: {}", e))),
        };

        let glob_matcher = input.include.and_then(|g| {
            globset::Glob::new(&g).ok().map(|g| g.compile_matcher())
        });

        let mut matches = Vec::new();

        let walker = WalkBuilder::new(&base_path)
            .hidden(false)
            .git_ignore(true)
            .build();

        for entry in walker {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            // Check glob filter
            if let Some(ref glob) = glob_matcher {
                let relative = path.strip_prefix(&base_path).unwrap_or(path);
                if !glob.is_match(relative) {
                    continue;
                }
            }

            // Read and search file
            match tokio::fs::read_to_string(path).await {
                Ok(content) => {
                    for (line_num, line) in content.lines().enumerate() {
                        if let Some(mat) = regex.find(line) {
                            matches.push(GrepMatch {
                                file_path: path.to_string_lossy().to_string(),
                                line_number: line_num + 1,
                                column: mat.start() + 1,
                                line_content: line.to_string(),
                            });
                        }
                    }
                }
                Err(_) => {
                    // Skip binary or unreadable files
                    continue;
                }
            }
        }

        // Format results
        let output = matches
            .into_iter()
            .map(|m| format!("{}:{}:{}", m.file_path, m.line_number, m.line_content))
            .collect::<Vec<_>>()
            .join("\n");

        Ok(ToolOutput::success(output))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;
    use serde_json::json;

    #[tokio::test]
    async fn test_glob() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("test.rs"), "").await.unwrap();
        fs::write(temp_dir.path().join("test.txt"), "").await.unwrap();

        let tool = GlobTool;
        let ctx = ToolContext {
            session_id: Default::default(),
            working_directory: temp_dir.path().to_path_buf(),
            env_vars: std::collections::HashMap::new(),
        };
        let input = ToolInput::new(json!({
            "pattern": "**/*.rs"
        }));

        let result = tool.execute(input, &ctx).await.unwrap();
        assert!(!result.is_error);
        assert!(result.content.contains("test.rs"));
        assert!(!result.content.contains(".txt"));
    }

    #[tokio::test]
    async fn test_grep() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("hello.rs"), "fn main() { println!(\"Hello\"); }")
            .await
            .unwrap();

        let tool = GrepTool;
        let ctx = ToolContext {
            session_id: Default::default(),
            working_directory: temp_dir.path().to_path_buf(),
            env_vars: std::collections::HashMap::new(),
        };
        let input = ToolInput::new(json!({
            "pattern": "println!",
            "path": temp_dir.path().to_str().unwrap()
        }));

        let result = tool.execute(input, &ctx).await.unwrap();
        assert!(!result.is_error);
        assert!(result.content.contains("println!"));
    }
}
