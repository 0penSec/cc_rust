//! CLI 集成测试
//!
//! 测试命令行界面和参数解析
//! 注意：这些测试主要验证 CLI 结构，不涉及实际的 API 调用

use assert_cmd::Command;
use predicates::prelude::*;
use std::env;

/// 测试 --version 输出
#[test]
fn test_cli_version() {
    let mut cmd = Command::cargo_bin("claude_rs").unwrap();
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("claude"));
}

/// 测试 --help 输出
#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("claude_rs").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("AI-powered coding assistant"))
        .stdout(predicate::str::contains("--api-key"))
        .stdout(predicate::str::contains("--model"))
        .stdout(predicate::str::contains("chat"))
        .stdout(predicate::str::contains("run"))
        .stdout(predicate::str::contains("tools"));
}

/// 测试 tools 子命令
#[test]
fn test_cli_tools_command() {
    let mut cmd = Command::cargo_bin("claude_rs").unwrap();
    cmd.arg("--api-key")
        .arg("test-key")
        .arg("tools");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("bash"))
        .stdout(predicate::str::contains("file_read"))
        .stdout(predicate::str::contains("file_write"));
}

/// 测试缺少 API key 时应该失败
#[test]
fn test_cli_missing_api_key() {
    // 临时移除环境变量
    let original = env::var("ANTHROPIC_API_KEY").ok();
    env::remove_var("ANTHROPIC_API_KEY");

    let mut cmd = Command::cargo_bin("claude_rs").unwrap();
    cmd.arg("tools");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required").or(predicate::str::contains("error")));

    // 恢复环境变量
    if let Some(key) = original {
        env::set_var("ANTHROPIC_API_KEY", key);
    }
}

/// 测试无效的子命令
#[test]
fn test_cli_invalid_command() {
    let mut cmd = Command::cargo_bin("claude_rs").unwrap();
    cmd.arg("--api-key")
        .arg("test-key")
        .arg("invalid-command");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized subcommand").or(predicate::str::contains("error")));
}

/// 测试 config 子命令
#[test]
fn test_cli_config_command() {
    let mut cmd = Command::cargo_bin("claude_rs").unwrap();
    cmd.arg("--api-key")
        .arg("test-api-key-12345")
        .arg("config");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("API Key"))
        .stdout(predicate::str::contains("test-api-ke"))  // 部分显示
        .stdout(predicate::str::contains("Model"))
        .stdout(predicate::str::contains("Working Directory"));
}

/// 测试 --model 参数
#[test]
fn test_cli_model_flag() {
    let mut cmd = Command::cargo_bin("claude_rs").unwrap();
    cmd.arg("--api-key")
        .arg("test-key")
        .arg("--model")
        .arg("claude-opus-4")
        .arg("config");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("claude-opus-4"));
}

/// 测试 --verbose 标志
#[test]
fn test_cli_verbose_flag() {
    let mut cmd = Command::cargo_bin("claude_rs").unwrap();
    cmd.arg("--api-key")
        .arg("test-key")
        .arg("--verbose")
        .arg("tools");
    // verbose 应该会增加日志输出
    cmd.assert()
        .success();
}

/// 测试 --working-dir 参数
#[test]
fn test_cli_working_dir_flag() {
    use std::path::PathBuf;

    let temp_dir = std::env::temp_dir();
    let mut cmd = Command::cargo_bin("claude_rs").unwrap();
    cmd.arg("--api-key")
        .arg("test-key")
        .arg("--working-dir")
        .arg(&temp_dir)
        .arg("config");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(temp_dir.to_string_lossy().as_ref()));
}

/// 测试 run 子命令的帮助
#[test]
fn test_cli_run_help() {
    let mut cmd = Command::cargo_bin("claude_rs").unwrap();
    cmd.arg("--api-key")
        .arg("test-key")
        .arg("run")
        .arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("prompt"));
}

/// 测试 chat 子命令的帮助
#[test]
fn test_cli_chat_help() {
    let mut cmd = Command::cargo_bin("claude_rs").unwrap();
    cmd.arg("--api-key")
        .arg("test-key")
        .arg("chat")
        .arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("message"));
}

/// 测试从环境变量读取 API key
#[test]
fn test_cli_env_api_key() {
    // 设置环境变量
    env::set_var("ANTHROPIC_API_KEY", "env-test-key");

    let mut cmd = Command::cargo_bin("claude_rs").unwrap();
    cmd.arg("config");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("env-test"));

    // 清理
    env::remove_var("ANTHROPIC_API_KEY");
}

/// 测试二进制是否存在且可执行
#[test]
fn test_binary_exists() {
    let result = Command::cargo_bin("claude");
    assert!(result.is_ok(), "Binary should exist");
}

/// 测试多个参数组合
#[test]
fn test_cli_multiple_flags() {
    let mut cmd = Command::cargo_bin("claude_rs").unwrap();
    cmd.arg("--api-key")
        .arg("test-key")
        .arg("--model")
        .arg("custom-model")
        .arg("--verbose")
        .arg("tools");
    cmd.assert().success();
}

/// 测试空参数处理
#[test]
fn test_cli_empty_prompt() {
    let mut cmd = Command::cargo_bin("claude_rs").unwrap();
    cmd.arg("--api-key")
        .arg("test-key")
        .arg("run")
        .arg("");  // 空提示
    // 应该处理空输入，可能成功也可能失败，但不应该 panic
    let output = cmd.output().unwrap();
    // 只要程序没有崩溃就是成功
    assert!(output.status.code().is_some());
}
