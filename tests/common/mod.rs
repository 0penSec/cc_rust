//! 测试共享模块
//!
//! 提供测试用的工具函数和辅助方法

use std::collections::HashMap;
use tempfile::TempDir;

/// 创建临时工作目录
pub fn setup_temp_dir() -> TempDir {
    TempDir::new().expect("Failed to create temp directory")
}

/// 创建测试环境变量（仅包含安全的变量）
pub fn setup_test_env_vars() -> HashMap<String, String> {
    let mut env = HashMap::new();
    env.insert("PATH".to_string(), std::env::var("PATH").unwrap_or_default());
    env.insert("HOME".to_string(), std::env::var("HOME").unwrap_or_default());
    env
}

/// 测试辅助 trait
pub trait StringAssertions {
    fn contains_any(&self, substrs: &[&str]) -> bool;
    fn contains_all(&self, substrs: &[&str]) -> bool;
}

impl StringAssertions for str {
    fn contains_any(&self, substrs: &[&str]) -> bool {
        substrs.iter().any(|&s| self.contains(s))
    }

    fn contains_all(&self, substrs: &[&str]) -> bool {
        substrs.iter().all(|&s| self.contains(s))
    }
}

/// 异步测试超时包装器
#[allow(dead_code)]
pub async fn with_timeout<T>(
    fut: impl std::future::Future<Output = T>,
    seconds: u64,
) -> Result<T, String> {
    tokio::time::timeout(
        tokio::time::Duration::from_secs(seconds),
        fut,
    )
    .await
    .map_err(|_| format!("Operation timed out after {} seconds", seconds))
}
