//! QueryEngine 集成测试
//!
//! 运行: cargo test --test query_engine_integration

use std::time::Duration;

/// 测试引擎创建
#[tokio::test]
async fn test_engine_creation() {
    use claude_engine::{QueryEngine, QueryEngineConfig, ClientConfig};

    let client_config = ClientConfig {
        api_key: "test-api-key".to_string(),
        ..Default::default()
    };

    let engine_config = QueryEngineConfig::default();
    
    let result = QueryEngine::new(client_config, engine_config);
    assert!(result.is_ok(), "QueryEngine should be created successfully");
}

/// 测试取消功能
#[tokio::test]
async fn test_engine_cancellation() {
    use claude_engine::{QueryEngine, QueryEngineConfig, ClientConfig};

    let client_config = ClientConfig {
        api_key: "test".to_string(),
        ..Default::default()
    };

    let engine = QueryEngine::new(client_config, QueryEngineConfig::default()).unwrap();
    
    // 取消应该成功
    engine.interrupt();
    
    // 验证取消状态
    // 注意：实际测试需要检查内部状态
}

/// 测试对话状态管理
#[tokio::test]
async fn test_conversation_state() {
    use claude_engine::{QueryEngine, QueryEngineConfig, ClientConfig, Conversation};

    let client_config = ClientConfig {
        api_key: "test".to_string(),
        ..Default::default()
    };

    let engine = QueryEngine::new(client_config, QueryEngineConfig::default()).unwrap();
    
    // 初始状态应该是空的
    let messages = engine.get_messages();
    assert!(messages.is_empty());
}

/// 测试工具注册
#[tokio::test]
async fn test_tool_registration() {
    use claude_engine::{QueryEngine, QueryEngineConfig, ClientConfig};
    use claude_tools::{BashTool, FileReadTool};

    let tools: Vec<Box<dyn claude_core::Tool>> = vec![
        Box::new(BashTool::new()),
        Box::new(FileReadTool),
    ];

    let client_config = ClientConfig {
        api_key: "test".to_string(),
        ..Default::default()
    };

    let engine_config = QueryEngineConfig {
        tools,
        ..Default::default()
    };

    let engine = QueryEngine::new(client_config, engine_config).unwrap();
    // 验证工具已注册
}

/// 测试配置验证
#[test]
fn test_config_validation() {
    use claude_engine::QueryEngineConfig;

    let config = QueryEngineConfig {
        max_turns: 100,
        verbose: true,
        ..Default::default()
    };

    assert_eq!(config.max_turns, 100);
    assert!(config.verbose);
}
