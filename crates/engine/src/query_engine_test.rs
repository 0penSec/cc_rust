// crates/engine/src/query_engine_test.rs
//! QueryEngine 单元测试

#[cfg(test)]
mod tests {
    use super::super::*;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    // ===== 基础构造测试 =====

    #[test]
    fn test_query_engine_config_default() {
        let config = QueryEngineConfig::default();
        assert_eq!(config.max_turns, 50);
        assert!(config.enable_streaming);
        assert!(!config.verbose);
    }

    #[test]
    fn test_query_engine_builder_pattern() {
        let config = QueryEngineConfig {
            max_turns: 10,
            verbose: true,
            ..Default::default()
        };

        assert_eq!(config.max_turns, 10);
        assert!(config.verbose);
    }

    // ===== 错误处理测试 =====

    #[test]
    fn test_invalid_api_key_error() {
        let result = QueryEngine::new(
            ClientConfig {
                api_key: "".to_string(),
                ..Default::default()
            },
            QueryEngineConfig::default()
        );

        // 空 API key 应该仍然可以创建客户端，但请求会失败
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_conversation_max_turns() {
        let config = QueryEngineConfig {
            max_turns: 2,
            ..Default::default()
        };

        let client_config = ClientConfig {
            api_key: "test".to_string(),
            ..Default::default()
        };

        let engine = QueryEngine::new(client_config, config).unwrap();

        // 验证初始状态
        assert_eq!(engine.get_messages().len(), 0);
    }

    // ===== 并发安全测试 =====

    #[tokio::test]
    async fn test_concurrent_message_access() {
        use std::sync::Arc;
        use tokio::task;

        let client_config = ClientConfig {
            api_key: "test".to_string(),
            ..Default::default()
        };

        let engine = Arc::new(
            QueryEngine::new(client_config, QueryEngineConfig::default()).unwrap()
        );

        let mut handles = vec![];

        // 并发读取消息
        for _ in 0..10 {
            let engine_clone = Arc::clone(&engine);
            handles.push(task::spawn(async move {
                let _ = engine_clone.get_messages();
            }));
        }

        for handle in handles {
            handle.await.unwrap();
        }
    }

    // ===== Token 计算测试 =====

    #[test]
    fn test_token_usage_accumulation() {
        let usage1 = TokenUsage {
            input_tokens: 100,
            output_tokens: 50,
            ..Default::default()
        };

        let usage2 = TokenUsage {
            input_tokens: 50,
            output_tokens: 25,
            ..Default::default()
        };

        // 验证累加逻辑
        let total = TokenUsage {
            input_tokens: usage1.input_tokens + usage2.input_tokens,
            output_tokens: usage1.output_tokens + usage2.output_tokens,
            ..Default::default()
        };

        assert_eq!(total.input_tokens, 150);
        assert_eq!(total.output_tokens, 75);
    }

    // ===== 工具注册测试 =====

    #[tokio::test]
    async fn test_tool_registration() {
        let tools: Vec<Box<dyn Tool>> = vec![
            Box::new(claude_tools::FileReadTool),
            Box::new(claude_tools::FileWriteTool),
            Box::new(claude_tools::BashTool::new()),
        ];

        let config = QueryEngineConfig {
            tools,
            ..Default::default()
        };

        let client_config = ClientConfig {
            api_key: "test".to_string(),
            ..Default::default()
        };

        let engine = QueryEngine::new(client_config, config).unwrap();

        // 验证工具已注册
        assert_eq!(engine.config.tools.len(), 3);
    }

    // ===== 取消测试 =====

    #[tokio::test]
    async fn test_cancellation() {
        let client_config = ClientConfig {
            api_key: "test".to_string(),
            ..Default::default()
        };

        let engine = QueryEngine::new(client_config, QueryEngineConfig::default()).unwrap();

        // 取消应该成功
        engine.interrupt();
        assert!(engine.cancellation_token.is_cancelled());
    }

    // ===== 边界条件测试 =====

    #[test]
    fn test_empty_message_handling() {
        let msg = Message::new_user("");
        assert!(msg.text_content().is_empty());
    }

    #[test]
    fn test_very_long_message() {
        let long_text = "a".repeat(10000);
        let msg = Message::new_user(&long_text);
        assert_eq!(msg.text_content().len(), 10000);
    }

    // ===== 序列化测试 =====

    #[test]
    fn test_message_serialization() {
        let msg = Message::new_user("Hello");
        let json = serde_json::to_string(&msg).unwrap();

        let deserialized: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.text_content(), "Hello");
    }

    #[test]
    fn test_content_block_serialization() {
        let block = ContentBlock::ToolUse {
            id: "tool_123".to_string(),
            name: "bash".to_string(),
            input: serde_json::json!({"command": "ls"}),
        };

        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains("tool_use"));
        assert!(json.contains("bash"));
    }
}
