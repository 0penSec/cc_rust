# Claude Code RS 项目结构详解

> 每个目录和文件的作用，以及测试编写方式

---

## 📁 顶层目录

```
claude-code-rs/
├── Cargo.toml          # 工作空间根配置
├── Cargo.lock          # 依赖版本锁定
├── README.md           # 项目主页
├── LICENSE             # MIT许可证
├── Dockerfile          # Docker构建配置
├── install.sh          # 一键安装脚本
├── Makefile            # 常用命令
├── .github/            # GitHub配置
│   └── workflows/      # CI/CD工作流
├── crates/             # 代码 crate（核心）
├── docs/               # 文档
└── target/             # 编译输出（自动生成）
```

---

## 🏗️ 工作空间配置 (Cargo.toml)

```toml
[workspace]
members = [
    "crates/core",       # 基础类型
    "crates/engine",     # 对话引擎
    "crates/tools",      # 工具实现
    "crates/commands",   # 斜杠命令
    "crates/tui",        # 终端UI
    "crates/services",   # 外部服务
    "crates/bridge",     # IDE桥接
    "crates/coordinator",# 多代理协调
    "crates/cli",        # 命令行入口
]
```

**为什么用 Workspace？**

就像一家大公司分成多个部门：
- 每个 crate 是一个部门
- 可以独立开发和测试
- 但共享同一个"大楼"（工作空间）

---

## 📦 Crates 详解

### 1. `crates/core` - 核心类型

**作用：** 定义所有人都要用的基础类型（就像公司的规章制度）

```
crates/core/
├── Cargo.toml
└── src/
    ├── lib.rs          # 模块导出
    ├── error.rs        # 错误类型定义
    ├── tool.rs         # Tool trait（工具接口）
    ├── message.rs      # 消息类型（User/Assistant/System）
    ├── permission.rs   # 权限控制
    ├── context.rs      # 上下文类型
    └── types.rs        # 通用类型
```

**关键代码：**

```rust
// tool.rs - 工具接口（就像"工具标准"）
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    async fn execute(&self, input: ToolInput, ctx: &ToolContext) -> ToolResult;
}

// message.rs - 消息枚举
pub enum Message {
    User { content: MessageContent },
    Assistant { content: AssistantContent },
    System { content: String },
}

// error.rs - 错误类型
pub enum ClaudeError {
    Io(String),
    Api { status: u16, message: String },
    ToolExecution(String),
    ...
}
```

**测试：**
- 当前无测试（主要是类型定义）
- 可以添加序列化/反序列化测试

---

### 2. `crates/engine` - 对话引擎

**作用：** AI 对话的核心逻辑（就像"大脑"）

```
crates/engine/
├── Cargo.toml
└── src/
    ├── lib.rs              # 模块导出
    ├── client.rs           # Anthropic API 客户端
    ├── stream.rs           # SSE 流处理
    ├── loop.rs             # ToolLoop（核心循环）
    ├── conversation.rs     # 对话管理
    ├── retry.rs            # 重试逻辑
    └── token.rs            # Token 计数
```

#### 文件详解

**`client.rs`** - API 客户端
```rust
pub struct AnthropicClient {
    config: ClientConfig,   // API密钥、地址
    http: Client,           // HTTP客户端
}

pub struct MessagesRequest {
    model: String,
    messages: Vec<Value>,   // 对话历史
    tools: Option<Vec<Value>>, // 可用工具
    system: Option<String>, // 系统提示
}
```

**`stream.rs`** - SSE 流处理 ⭐关键
```rust
pub struct EventStream {
    es: EventSource,                    // SSE连接
    current_tool_use: Option<ToolUseBuilder>, // 正在组装的工具调用
}

pub enum StreamEvent {
    TextDelta { text: String },          // AI说的每个字
    ToolUseStart { id, name, input },    // AI要调用工具
    MessageComplete { stop_reason, usage }, // 说完了
}
```

**`loop.rs`** - ToolLoop ⭐⭐核心
```rust
pub struct ToolLoop {
    client: AnthropicClient,
    tools: Vec<Box<dyn Tool>>,  // 所有可用工具
    max_iterations: usize,      // 防止无限循环
}

impl ToolLoop {
    pub async fn run(&self, conversation: &mut Conversation, ctx: &ToolContext) -> Result {
        loop {
            // 1. 调用API
            // 2. 流式接收响应
            // 3. 检测工具调用
            // 4. 执行工具
            // 5. 返回结果
            // 直到 stop_reason != "tool_use"
        }
    }
}
```

**测试：**
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_turn_result_complete() {
        let result = TurnResult::Complete { usage: ... };
        match result {
            TurnResult::Complete { usage } => {
                assert_eq!(usage.input_tokens, 10);
            }
            _ => panic!("Expected Complete"),
        }
    }
}
```

---

### 3. `crates/tools` - 工具实现

**作用：** 实现各种具体工具（就像"工具箱"）

```
crates/tools/
├── Cargo.toml
└── src/
    ├── lib.rs          # 模块导出和注册表
    ├── registry.rs     # 工具注册表
    ├── bash.rs         # Bash命令执行
    ├── file.rs         # 文件操作（读/写/编辑）
    ├── search.rs       # 搜索工具（glob/grep）
    └── web.rs          # 网络工具（fetch/search）
```

#### 文件详解

**`registry.rs`** - 工具注册表
```rust
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>, // 名称 -> 工具
}

impl ToolRegistry {
    pub fn register(&mut self, tool: Box<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }
    
    pub fn get(&self, name: &str) -> Option<&dyn Tool> {
        self.tools.get(name).map(|t| t.as_ref())
    }
}
```

**`bash.rs`** - Bash命令 ⭐有测试
```rust
pub struct BashTool {
    timeout_seconds: u64,
}

#[async_trait]
impl Tool for BashTool {
    fn name(&self) -> &str { "bash" }
    
    async fn execute(&self, input: ToolInput, ctx: &ToolContext) -> ToolResult {
        let input: BashInput = input.parse()?;
        
        // 使用 tokio::process::Command 执行
        let output = Command::new("bash")
            .arg("-c").arg(&input.command)
            .current_dir(&ctx.working_directory)
            .output().await?;
            
        // 返回结果
        Ok(ToolOutput::success(...))
    }
}

// ========== 测试 ==========
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bash_echo() {
        // 1. 准备
        let tool = BashTool::new();
        let ctx = ToolContext {
            working_directory: std::env::current_dir().unwrap(),
            env_vars: HashMap::new(),
            session_id: Default::default(),
        };
        let input = ToolInput::new(json!({"command": "echo 'Hello'"}));
        
        // 2. 执行
        let result = tool.execute(input, &ctx).await.unwrap();
        
        // 3. 验证
        assert!(!result.is_error);
        assert!(result.content.contains("Hello"));
    }
}
```

**`file.rs`** - 文件操作 ⭐有测试
```rust
pub struct FileReadTool;
pub struct FileWriteTool;
pub struct FileEditTool;

#[async_trait]
impl Tool for FileReadTool {
    async fn execute(&self, input, ctx) -> ToolResult {
        let path = ctx.working_directory.join(input.path);
        let content = tokio::fs::read_to_string(&path).await?;
        Ok(ToolOutput::success(content))
    }
}

// ========== 测试 ==========
#[cfg(test)]
mod tests {
    use tempfile::TempDir;  // 临时目录
    
    #[tokio::test]
    async fn test_file_read() {
        // 1. 创建临时目录
        let temp_dir = TempDir::new().unwrap();
        
        // 2. 创建测试文件
        fs::write(temp_dir.path().join("test.txt"), "Hello")
            .await.unwrap();
        
        // 3. 执行测试
        let result = tool.execute(input, &ctx).await.unwrap();
        
        // 4. 验证
        assert!(result.content.contains("Hello"));
        
        // 5. 自动清理（temp_dir 离开作用域时删除）
    }
}
```

**`search.rs`** - 搜索工具 ⭐有测试
```rust
pub struct GlobTool;   // 文件模式匹配（如 "**/*.rs"）
pub struct GrepTool;   // 内容搜索（正则表达式）

// ========== 测试 ==========
#[tokio::test]
async fn test_glob() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.path().join("test.rs"), "").await.unwrap();
    fs::write(temp_dir.path().join("test.txt"), "").await.unwrap();
    
    let input = ToolInput::new(json!({"pattern": "**/*.rs"}));
    let result = tool.execute(input, &ctx).await.unwrap();
    
    assert!(result.content.contains("test.rs"));    // 应包含
    assert!(!result.content.contains(".txt"));      // 不应包含
}
```

**`web.rs`** - 网络工具 ⭐有测试
```rust
pub struct WebFetchTool;   // 抓取网页
pub struct WebSearchTool;  // 网络搜索（TODO）

// ========== 测试 ==========
#[tokio::test]
async fn test_web_fetch() {
    // 使用 wiremock 模拟 HTTP 服务器
    let mock_server = MockServer::start().await;
    
    Mock::given(method("GET"))
        .and(path("/test"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string("Hello from web"))
        .mount(&mock_server)
        .await;
    
    let input = ToolInput::new(json!({
        "url": format!("{}/test", mock_server.uri())
    }));
    
    let result = tool.execute(input, &ctx).await.unwrap();
    assert!(result.content.contains("Hello from web"));
}
```

---

### 4. `crates/cli` - 命令行入口

**作用：** 用户交互界面（就像"前台接待"）

```
crates/cli/
├── Cargo.toml
└── src/
    └── main.rs         # 程序入口
```

**`main.rs`**
```rust
#[derive(Parser)]
struct Cli {
    #[arg(long, env = "ANTHROPIC_API_KEY")]
    api_key: String,
    #[arg(short, long, default_value = "claude-sonnet-4-6")]
    model: String,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Chat { message: Option<String> },
    Run { prompt: String },
    Tools,
    Config,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Some(Commands::Tools) => list_tools().await,
        Some(Commands::Run { prompt }) => run_single_prompt(...).await,
        Some(Commands::Chat { .. }) => run_interactive(...).await,
        None => run_interactive(...).await,
    }
}
```

**测试：**
- 当前无测试
- 可以添加集成测试（使用 `assert_cmd`）

---

### 5. 其他 Crates

| Crate | 作用 | 状态 |
|-------|------|------|
| `commands` | 斜杠命令实现（/commit, /cost等） | 🚧 框架 |
| `tui` | 终端界面（ratatui） | 🚧 框架 |
| `services` | 外部服务（MCP, LSP, 认证） | 🚧 框架 |
| `bridge` | IDE桥接（VS Code等） | 🚧 框架 |
| `coordinator` | 多代理协调 | 🚧 框架 |

---

## 🧪 测试编写详解

### 测试类型

| 类型 | 位置 | 用途 |
|------|------|------|
| 单元测试 | 源码文件中 `#[cfg(test)]` | 测试单个函数/模块 |
| 集成测试 | `tests/` 目录 | 测试完整流程 |
| 文档测试 | 代码注释中 `///` | 示例代码验证 |

### 测试标记

```rust
#[test]           // 同步测试
#[tokio::test]    // 异步测试（需要 tokio 运行时）
#[should_panic]   // 期望测试 panic
#[ignore]         // 忽略此测试
```

### 测试结构（AAA模式）

```rust
#[tokio::test]
async fn test_feature() {
    // Arrange - 准备
    let tool = MyTool::new();
    let temp_dir = TempDir::new().unwrap();
    let ctx = create_context(&temp_dir);
    let input = create_input();
    
    // Act - 执行
    let result = tool.execute(input, &ctx).await;
    
    // Assert - 验证
    assert!(result.is_ok());
    assert_eq!(result.unwrap().content, "expected");
}
```

### 常用测试工具

| Crate | 用途 | 示例 |
|-------|------|------|
| `tempfile` | 临时文件/目录 | `TempDir::new().unwrap()` |
| `wiremock` | HTTP模拟 | `MockServer::start().await` |
| `mockall` | Mock对象 | `mock!(Trait)` |
| `assert_cmd` | CLI测试 | `Command::cargo_bin("claude")` |

### 运行测试

```bash
# 运行所有测试
cargo test --all

# 运行特定包
cargo test --package claude-tools

# 运行特定测试
cargo test test_bash_echo

# 显示输出
cargo test -- --nocapture

# 并行运行
cargo test -- --test-threads=4

# 只运行忽略测试
cargo test -- --ignored
```

---

## 📊 项目依赖关系

```
cli
├── core (基础类型)
├── engine (对话引擎)
│   ├── core
│   └── services (API客户端)
├── tools (工具实现)
│   └── core
└── commands (斜杠命令)
    └── core
```

**依赖规则：**
- 上层可以依赖下层
- 下层不能依赖上层
- 同层尽量不互相依赖

---

## 🎯 开发建议

### 添加新工具

1. 在 `crates/tools/src/` 创建新文件
2. 实现 `Tool` trait
3. 在 `registry.rs` 注册
4. 编写测试
5. 更新文档

### 添加测试

1. 在源文件底部添加 `#[cfg(test)] mod tests`
2. 使用 `TempDir` 避免污染真实文件系统
3. 使用 `MockServer` 避免真实网络请求
4. 覆盖正常和异常场景

### 调试技巧

```bash
# 详细输出
RUST_LOG=debug cargo run

# 特定模块日志
RUST_LOG=claude_tools=trace cargo test

# 单测试调试
cargo test test_name -- --nocapture
```

---

**项目地址：** https://github.com/0penSec/cc_rust
