# Claude Code RS 测试指南

> 详细介绍项目中的测试文件、编写方法和执行效果

---

## 📁 测试文件分布

### 单元测试（内联测试）

Rust 的单元测试通常和源代码放在一起，用 `#[cfg(test)]` 标记：

```
crates/
├── tools/src/
│   ├── bash.rs      # 包含 BashTool 测试
│   ├── file.rs      # 包含文件操作测试
│   ├── search.rs    # 包含搜索工具测试
│   └── web.rs       # 包含网络工具测试
├── engine/src/
│   ├── loop.rs      # 包含 ToolLoop 测试
│   └── token.rs     # 包含 Token 计数测试
```

### 集成测试

目前项目以单元测试为主，尚未添加 `tests/` 目录的集成测试。

---

## 🧪 测试详解

### 1. BashTool 测试 (`crates/tools/src/bash.rs`)

**测试代码：**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bash_echo() {
        // 1. 准备工具实例
        let tool = BashTool::new();
        
        // 2. 创建上下文（模拟的工作目录和环境变量）
        let ctx = ToolContext {
            session_id: Default::default(),
            working_directory: std::env::current_dir().unwrap(),
            env_vars: std::collections::HashMap::new(),
        };
        
        // 3. 构建输入参数
        let input = ToolInput::new(json!({
            "command": "echo 'Hello World'"
        }));

        // 4. 执行测试
        let result = tool.execute(input, &ctx).await.unwrap();
        
        // 5. 断言验证
        assert!(!result.is_error);  // 不应报错
        assert!(result.content.contains("Hello World"));  // 输出应包含预期内容
    }

    #[tokio::test]
    async fn test_bash_error() {
        let tool = BashTool::new();
        let ctx = ToolContext {
            session_id: Default::default(),
            working_directory: std::env::current_dir().unwrap(),
            env_vars: std::collections::HashMap::new(),
        };
        let input = ToolInput::new(json!({
            "command": "exit 1"  // 返回非零退出码
        }));

        let result = tool.execute(input, &ctx).await.unwrap();
        assert!(result.is_error);  // 应标记为错误
    }
}
```

**测试了什么？**
- ✅ 正常命令执行（echo）
- ✅ 错误命令处理（exit 1）
- ✅ 超时机制（通过工具默认配置）

---

### 2. 文件操作测试 (`crates/tools/src/file.rs`)

**测试代码：**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;  // 临时目录，测试后自动清理
    use tokio::fs;

    #[tokio::test]
    async fn test_file_read() {
        // 1. 创建临时目录
        let temp_dir = TempDir::new().unwrap();
        
        // 2. 创建测试文件
        fs::write(temp_dir.path().join("test.txt"), "Hello World")
            .await
            .unwrap();

        // 3. 测试读取
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

        // 验证文件确实被写入
        let content = fs::read_to_string(temp_dir.path().join("output.txt"))
            .await
            .unwrap();
        assert_eq!(content, "Test content");
    }

    #[tokio::test]
    async fn test_file_edit() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("edit.txt");
        
        // 先创建文件
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

        // 验证替换成功
        let content = fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(content, "Hello New World");
    }
}
```

**测试了什么？**
- ✅ 文件读取（FileReadTool）
- ✅ 文件写入（FileWriteTool）
- ✅ 文件编辑/替换（FileEditTool）
- ✅ 临时文件清理（使用 tempfile crate）

---

### 3. 搜索工具测试 (`crates/tools/src/search.rs`)

**测试代码：**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;

    #[tokio::test]
    async fn test_glob() {
        // 创建临时目录和文件
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
            "pattern": "**/*.rs"  // 只匹配 .rs 文件
        }));

        let result = tool.execute(input, &ctx).await.unwrap();
        assert!(!result.is_error);
        assert!(result.content.contains("test.rs"));    // 应包含
        assert!(!result.content.contains(".txt"));      // 不应包含
    }

    #[tokio::test]
    async fn test_grep() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("hello.rs"), 
            "fn main() { println!(\"Hello\"); }"
        ).await.unwrap();

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
```

**测试了什么？**
- ✅ 文件模式匹配（GlobTool）
- ✅ 内容搜索（GrepTool）
- ✅ 正则表达式匹配

---

### 4. 网络工具测试 (`crates/tools/src/web.rs`)

**测试代码：**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{MockServer, Mock, ResponseTemplate};  // HTTP 模拟服务器
    use wiremock::matchers::{method, path};

    #[tokio::test]
    async fn test_web_fetch() {
        // 1. 启动模拟服务器
        let mock_server = MockServer::start().await;

        // 2. 配置模拟响应
        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_string("Hello from web"))
            .mount(&mock_server)
            .await;

        // 3. 测试请求
        let tool = WebFetchTool;
        let ctx = ToolContext {
            session_id: Default::default(),
            working_directory: std::env::current_dir().unwrap(),
            env_vars: std::collections::HashMap::new(),
        };
        let input = ToolInput::new(json!({
            "url": format!("{}/test", mock_server.uri())
        }));

        let result = tool.execute(input, &ctx).await.unwrap();
        assert!(!result.is_error);
        assert!(result.content.contains("Hello from web"));
    }
}
```

**测试了什么？**
- ✅ HTTP 请求发送
- ✅ 响应内容获取
- ✅ 使用 wiremock 模拟外部服务（不依赖真实网络）

---

### 5. ToolLoop 测试 (`crates/engine/src/loop.rs`)

**测试代码：**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_turn_result_complete() {
        // 测试 TurnResult::Complete 变体
        let result = TurnResult::Complete {
            usage: TokenUsage {
                input_tokens: 10,
                output_tokens: 20,
            },
        };

        match result {
            TurnResult::Complete { usage } => {
                assert_eq!(usage.input_tokens, 10);
                assert_eq!(usage.output_tokens, 20);
            }
            _ => panic!("Expected Complete"),
        }
    }
}
```

**测试了什么？**
- ✅ 数据结构验证
- ⚠️ 注意：这只是简单的结构体测试，没有测试完整的对话流程

---

## 🎯 测试编写方法

### 基本结构

```rust
#[cfg(test)]      // ← 只在测试时编译
mod tests {       // ← 测试模块
    use super::*; // ← 导入父模块的所有内容

    #[tokio::test]  // ← 异步测试用例
    async fn test_name() {
        // 1. 准备（Arrange）
        let tool = MyTool::new();
        let ctx = create_test_context();
        let input = create_test_input();
        
        // 2. 执行（Act）
        let result = tool.execute(input, &ctx).await.unwrap();
        
        // 3. 验证（Assert）
        assert!(!result.is_error);
        assert!(result.content.contains("expected"));
    }
}
```

### 常用断言宏

```rust
assert!(condition);                    // 断言为真
assert_eq!(left, right);              // 断言相等
assert_ne!(left, right);              // 断言不等
assert!(result.is_err());             // 断言出错
assert!(result.is_ok());              // 断言成功
panic!("message");                    // 手动触发失败
```

### 异步测试

```rust
// 使用 #[tokio::test] 替代 #[test]
#[tokio::test]
async fn async_test() {
    let result = async_function().await;
    assert!(result.is_ok());
}
```

### 使用临时文件

```rust
use tempfile::TempDir;

#[tokio::test]
async fn test_with_temp_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    
    // 写入文件
    fs::write(&file_path, "content").await.unwrap();
    
    // 测试...
    
    // 自动清理：temp_dir 离开作用域时删除整个目录
}
```

### 模拟外部服务

```rust
use wiremock::{MockServer, Mock, ResponseTemplate};

#[tokio::test]
async fn test_with_mock_server() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("GET"))
        .and(path("/api"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(json!({"status": "ok"})))
        .mount(&mock_server)
        .await;
    
    // 使用 mock_server.uri() 作为 API 地址
    let api_url = format!("{}/api", mock_server.uri());
}
```

---

## ⚠️ 测试覆盖的不足

### 当前测试缺失

| 组件 | 测试状态 | 缺失内容 |
|------|----------|----------|
| `core` | ❌ 无测试 | Error、Message、Permission 等基础类型 |
| `engine/client` | ❌ 无测试 | API 客户端、HTTP 请求 |
| `engine/stream` | ❌ 无测试 | SSE 流解析 |
| `engine/loop` | ⚠️ 简单测试 | 只有数据结构测试，没有完整流程测试 |
| `tools` | ✅ 有测试 | 但缺少错误场景测试 |
| `cli` | ❌ 无测试 | 命令行参数解析、主流程 |

### 建议添加的测试

1. **集成测试** (`tests/` 目录)
   ```rust
   // tests/integration_test.rs
   #[tokio::test]
   async fn test_full_conversation() {
       // 测试完整对话流程
   }
   ```

2. **API Mock 测试**
   ```rust
   // 模拟 Anthropic API 响应
   #[tokio::test]
   async fn test_api_response_handling() {
       let mock_api = MockServer::start().await;
       // ...
   }
   ```

3. **错误场景测试**
   ```rust
   #[tokio::test]
   async fn test_file_not_found() {
       let result = tool.execute(invalid_input, &ctx).await;
       assert!(result.is_error);
       assert!(result.content.contains("not found"));
   }
   ```

---

## 🚀 运行测试

### 运行所有测试

```bash
cargo test --all
```

### 运行特定包的测试

```bash
cargo test --package claude-tools
cargo test --package claude-engine
```

### 运行特定测试

```bash
# 按名称过滤
cargo test test_bash_echo

# 运行工具的所有测试
cargo test --package claude-tools --lib
```

### 显示输出

```bash
cargo test -- --nocapture
```

### 详细模式

```bash
cargo test -- --show-output
cargo test -v
```

---

## 📊 测试执行效果

### 预期输出示例

```
$ cargo test --package claude-tools

running 8 tests
test bash::tests::test_bash_echo ... ok
test bash::tests::test_bash_error ... ok
test file::tests::test_file_edit ... ok
test file::tests::test_file_read ... ok
test file::tests::test_file_write ... ok
test search::tests::test_glob ... ok
test search::tests::test_grep ... ok
test web::tests::test_web_fetch ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### 测试耗时

- **简单测试**（如 echo）：~10-50ms
- **文件操作测试**：~50-100ms（含临时文件创建）
- **网络测试**：~100-300ms（含 mock 服务器启动）

---

## 📝 最佳实践

1. **每个工具都要有测试**
   - 正常路径测试
   - 错误路径测试
   - 边界条件测试

2. **使用临时资源**
   - `TempDir` 代替真实文件系统
   - `MockServer` 代替真实网络请求

3. **异步测试要正确**
   - 使用 `#[tokio::test]`
   - 正确处理 `.await`

4. **断言要明确**
   - 不仅检查 `is_error`，还要检查错误内容
   - 验证输出的具体内容

5. **测试名称要描述性**
   ```rust
   // 好
   async fn test_bash_command_with_timeout()
   
   // 不好
   async fn test_1()
   ```

---

## 🎓 学习资源

- [Rust Book - Testing](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Tokio Testing](https://tokio.rs/tokio/topics/testing)
- [Wiremock Docs](https://docs.rs/wiremock/)
- [Tempfile Docs](https://docs.rs/tempfile/)

---

> 测试是代码质量的重要保障，建议随着开发持续补充测试用例！
