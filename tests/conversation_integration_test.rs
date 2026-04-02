//! 对话流程集成测试
//!
//! 测试完整的对话流程，包括消息传递和状态管理

use claude_core::{
    message::{AssistantContent, MessageContent},
    AgentId, Message, SessionId,
};
use claude_engine::{Conversation, ConversationBuilder};

/// 测试创建对话
#[test]
fn test_conversation_creation() {
    let conversation = Conversation::builder()
        .system_prompt("You are a helpful assistant".to_string())
        .model("claude-sonnet-4-6".to_string())
        .max_tokens(4096)
        .build();

    assert_eq!(conversation.messages.len(), 0);
    assert_eq!(conversation.model, "claude-sonnet-4-6");
    assert_eq!(conversation.max_tokens, 4096);
    assert!(conversation.system_prompt.is_some());
}

/// 测试添加用户消息
#[test]
fn test_conversation_add_user_message() {
    let mut conversation = Conversation::builder().build();

    conversation.add_user_message("Hello, Claude!");

    assert_eq!(conversation.messages.len(), 1);
    match &conversation.messages[0] {
        Message::User { content } => match content {
            MessageContent::Text(text) => assert_eq!(text, "Hello, Claude!"),
            _ => panic!("Expected text content"),
        },
        _ => panic!("Expected user message"),
    }
}

/// 测试添加助手消息
#[test]
fn test_conversation_add_assistant_message() {
    let mut conversation = Conversation::builder().build();

    conversation.add_assistant_message("Hello! How can I help you?");

    assert_eq!(conversation.messages.len(), 1);
    match &conversation.messages[0] {
        Message::Assistant { content } => match content {
            AssistantContent::Text(text) => {
                assert_eq!(text, "Hello! How can I help you?")
            }
            _ => panic!("Expected text content"),
        },
        _ => panic!("Expected assistant message"),
    }
}

/// 测试多轮对话
#[test]
fn test_conversation_multi_turn() {
    let mut conversation = Conversation::builder().build();

    // 第1轮：用户
    conversation.add_user_message("What is Rust?");

    // 第1轮：助手
    conversation.add_assistant_message("Rust is a systems programming language.");

    // 第2轮：用户
    conversation.add_user_message("Why is it good?");

    // 第2轮：助手
    conversation.add_assistant_message("It provides memory safety without garbage collection.");

    assert_eq!(conversation.messages.len(), 4);

    // 验证消息顺序
    match &conversation.messages[0] {
        Message::User { .. } => (),
        _ => panic!("Expected user message at index 0"),
    }
    match &conversation.messages[1] {
        Message::Assistant { .. } => (),
        _ => panic!("Expected assistant message at index 1"),
    }
    match &conversation.messages[2] {
        Message::User { .. } => (),
        _ => panic!("Expected user message at index 2"),
    }
    match &conversation.messages[3] {
        Message::Assistant { .. } => (),
        _ => panic!("Expected assistant message at index 3"),
    }
}

/// 测试 Token 使用统计
#[test]
fn test_conversation_token_usage() {
    let mut conversation = Conversation::builder().build();

    conversation.update_token_usage(100, 50);
    assert_eq!(conversation.total_input_tokens, 100);
    assert_eq!(conversation.total_output_tokens, 50);

    conversation.update_token_usage(50, 30);
    assert_eq!(conversation.total_input_tokens, 150);
    assert_eq!(conversation.total_output_tokens, 80);
}

/// 测试带工具调用的对话
#[test]
fn test_conversation_with_tool_calls() {
    use claude_core::message::ToolCall;

    let mut conversation = Conversation::builder().build();

    // 添加用户消息
    conversation.add_user_message("List files");

    // 添加助手消息（包含工具调用）
    let tool_calls = vec![ToolCall {
        id: "call_1".to_string(),
        name: "bash".to_string(),
        input: serde_json::json!({"command": "ls"}),
    }];

    conversation.add_message(Message::Assistant {
        content: AssistantContent::ToolCalls(tool_calls),
    });

    // 添加工具结果
    conversation.add_user_message("Tool result: file1.txt file2.rs");

    assert_eq!(conversation.messages.len(), 3);
}

/// 测试对话构建器的链式调用
#[test]
fn test_conversation_builder_chain() {
    let session_id = SessionId::new();

    let conversation = Conversation::builder()
        .session_id(session_id)
        .system_prompt("Custom system prompt".to_string())
        .model("claude-opus-4".to_string())
        .max_tokens(8192)
        .build();

    assert_eq!(conversation.session_id, session_id);
    assert_eq!(conversation.system_prompt, Some("Custom system prompt".to_string()));
    assert_eq!(conversation.model, "claude-opus-4");
    assert_eq!(conversation.max_tokens, 8192);
}

/// 测试空对话
#[test]
fn test_empty_conversation() {
    let conversation = Conversation::builder().build();

    assert!(conversation.messages.is_empty());
    assert_eq!(conversation.total_input_tokens, 0);
    assert_eq!(conversation.total_output_tokens, 0);
}

/// 测试长对话消息
#[test]
fn test_long_conversation() {
    let mut conversation = Conversation::builder().build();

    // 添加20轮对话
    for i in 0..20 {
        conversation.add_user_message(format!("Question {}", i));
        conversation.add_assistant_message(format!("Answer {}", i));
    }

    assert_eq!(conversation.messages.len(), 40);

    // 验证最后一条消息
    match conversation.messages.last() {
        Some(Message::Assistant { content }) => match content {
            AssistantContent::Text(text) => assert_eq!(text, "Answer 19"),
            _ => panic!("Expected text"),
        },
        _ => panic!("Expected assistant message"),
    }
}

/// 测试消息类型转换（用于 API 调用）
#[test]
fn test_message_serialization() {
    let message = Message::User {
        content: MessageContent::Text("Hello".to_string()),
    };

    let json = serde_json::to_string(&message).unwrap();
    assert!(json.contains("Hello"));

    let deserialized: Message = serde_json::from_str(&json).unwrap();
    match deserialized {
        Message::User { content } => match content {
            MessageContent::Text(text) => assert_eq!(text, "Hello"),
            _ => panic!("Expected text content"),
        },
        _ => panic!("Expected user message"),
    }
}

/// 测试带多部分内容的用户消息
#[test]
fn test_multi_content_message() {
    use claude_core::message::ContentPart;

    let parts = vec![
        ContentPart::Text {
            text: "Check this code:".to_string(),
        },
        ContentPart::Text {
            text: "fn main() {}".to_string(),
        },
    ];

    let message = Message::User {
        content: MessageContent::MultiContent(parts),
    };

    match message {
        Message::User { content } => match content {
            MessageContent::MultiContent(parts) => assert_eq!(parts.len(), 2),
            _ => panic!("Expected multi content"),
        },
        _ => panic!("Expected user message"),
    }
}
