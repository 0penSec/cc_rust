# Claude Code RS 代码详细分析

## 项目架构概览

```
claude-code-rs/
├── crates/
│   ├── core/          # 核心类型和 Trait 定义
│   ├── engine/        # LLM 引擎和对话管理
│   ├── tools/         # 工具实现（bash、文件操作等）
│   ├── memory/        # 持久化记忆系统
│   ├── cli/           # 命令行界面
│   ├── tui/           # 终端用户界面
│   └── ...
```

---

## 一、Core  crate（核心层）

### 1.1 lib.rs - 核心库入口

**模块导出：**
- `context` - 上下文管理
- `error` - 错误类型
- `message` - 消息类型
- `permission` - 权限控制
- `tool` - 工具 Trait
- `types` - 通用类型

**核心类型：**

```rust
/// 会话唯一标识符
pub struct SessionId(pub Uuid);

/// 代理唯一标识符  
pub struct AgentId(pub Uuid);

/// 工具执行唯一标识符
pub struct ToolExecutionId(pub Uuid);
```

**开发说明：**
- 所有 ID 类型使用 UUID v4 生成
- 实现了 `Default` trait，方便创建新实例
- 实现了 `Serialize/Deserialize`，支持持久化

---

### 1.2 tool.rs - 工具 Trait 定义

**核心 Trait：**

```rust
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;                           // 工具名称
    fn description(&self) -> &str;                    // 工具描述
    fn input_schema(&self) -> Value;                  // JSON Schema 参数定义
    fn permission_mode(&self) -> PermissionMode;      // 权限模式
    async fn execute(&self, input: ToolInput, ctx: &ToolContext) -> ToolResult;
}
```

**关键结构体：**

| 结构体 | 说明 |
|--------|------|
| `ToolInput` | 工具输入，包含原始 JSON 值，提供 `parse<T>()` 方法反序列化 |
| `ToolOutput` | 工具输出，包含内容、错误标志和元数据 |
| `ToolContext` | 执行上下文，包含会话ID、工作目录、环境变量 |
| `ToolDefinition` | 工具定义，用于注册和描述 |

**工厂方法：**
```rust
// 创建成功输出
ToolOutput::success("操作成功")

// 创建错误输出
ToolOutput::error("操作失败")

// 添加元数据
ToolOutput::success("结果").with_metadata(json!({"key": "value"}))
```

---

### 1.3 context.rs - 上下文管理

**四层上下文结构：**

```rust
pub struct Context {
    pub user: UserContext,              // 用户上下文
    pub project: ProjectContext,        // 项目上下文
    pub conversation: ConversationContext,  // 对话上下文
}
```

**UserContext（用户上下文）：**
```rust
pub struct UserContext {
    pub preferences: HashMap<String, String>,  // 用户偏好设置
    pub working_directory: PathBuf,            // 工作目录
    pub shell: String,                         // 默认 shell
    pub environment: HashMap<String, String>,  // 环境变量
}
```

**ProjectContext（项目上下文）：**
```rust
pub struct ProjectContext {
    pub root_path: PathBuf,               // 项目根路径
    pub git_remote: Option<String>,       // Git 远程地址
    pub detected_language: Option<String>, // 检测到的编程语言
    pub claude_md_content: Option<String>, // CLAUDE.md 内容
}
```

**ConversationContext（对话上下文）：**
```rust
pub struct ConversationContext {
    pub message_count: usize,              // 消息数量
    pub total_tokens: usize,               // 总 token 数
    pub cost_usd: f64,                     // 预估成本
    pub pending_tool_calls: Vec<ToolCall>, // 待处理的工具调用
}
```

---

## 二、Engine crate（引擎层）

### 2.1 lib.rs - 引擎入口

**EngineConfig（引擎配置）：**
```rust
pub struct EngineConfig {
    pub client: ClientConfig,      // API 客户端配置
    pub max_retries: u32,          // 最大重试次数
    pub retry_delay_ms: u64,       // 重试延迟
    pub enable_streaming: bool,    // 是否启用流式响应
}
```

**QueryEngine（查询引擎）：**
```rust
pub struct QueryEngine {
    _config: EngineConfig,
    _client: AnthropicClient,
    conversations: HashMap<SessionId, Conversation>,  // 会话管理
}
```

**核心方法：**

| 方法 | 说明 |
|------|------|
| `new(config)` | 创建引擎实例，初始化 HTTP 客户端 |
| `create_conversation(id)` | 创建新对话 |
| `get_conversation(id)` | 获取对话（只读） |
| `get_conversation_mut(id)` | 获取对话（可变） |

---

### 2.2 client.rs - API 客户端

**ClientConfig（客户端配置）：**
```rust
pub struct ClientConfig {
    pub api_key: String,                              // API 密钥
    pub api_base: String,                             // API 基础 URL
    pub version: String,                              // API 版本
    pub timeout: Duration,                            // 超时时间
}
```

**AnthropicClient（API 客户端）：**
```rust
pub struct AnthropicClient {
    config: ClientConfig,
    http: Client,  // reqwest HTTP 客户端
}
```

**构建过程：**
```rust
pub fn new(config: ClientConfig) -> ClaudeResult<Self> {
    // 1. 创建请求头
    let mut headers = header::HeaderMap::new();
    headers.insert("x-api-key", ...);           // API 密钥
    headers.insert("anthropic-version", ...);   // API 版本
    
    // 2. 构建 HTTP 客户端
    let http = Client::builder()
        .default_headers(headers)
        .timeout(config.timeout)
        .build()?;
    
    Ok(Self { config, http })
}
```

**MessagesRequest（API 请求体）：**
```rust
pub struct MessagesRequest {
    pub model: String,                    // 模型名称
    pub max_tokens: usize,                // 最大 token 数
    pub messages: Vec<Value>,             // 消息列表
    pub tools: Option<Vec<Value>>,        // 可用工具定义
    pub system: Option<String>,           // 系统提示词
    pub temperature: Option<f32>,         // 温度参数
    pub thinking: Option<ThinkingConfig>, // 思考模式配置
}
```

**ContentBlock（内容块）：**
```rust
pub enum ContentBlock {
    Text { text: String },                          // 文本内容
    Thinking { thinking: String, signature: String }, // 思考过程
    ToolUse { id: String, name: String, input: Value }, // 工具调用
}
```

---

### 2.3 conversation.rs - 对话管理

**Conversation（对话）：**
```rust
pub struct Conversation {
    pub session_id: SessionId,
    pub messages: Vec<Message>,           // 消息历史
    pub system_prompt: Option<String>,    // 系统提示词
    pub model: String,                    // 使用的模型
    pub max_tokens: usize,                // 最大 token 数
    pub total_input_tokens: usize,        // 输入 token 统计
    pub total_output_tokens: usize,       // 输出 token 统计
}
```

**消息操作方法：**

| 方法 | 说明 |
|------|------|
| `builder()` | 获取构建器 |
| `add_message(msg)` | 添加消息 |
| `add_user_message(content)` | 添加用户消息 |
| `add_assistant_message(content)` | 添加助手消息 |
| `add_tool_results(results)` | 添加工具执行结果 |
| `pending_tool_calls()` | 获取待处理的工具调用 |
| `update_token_usage(in, out)` | 更新 token 使用量 |

**ConversationBuilder（构建器模式）：**
```rust
pub struct ConversationBuilder {
    session_id: Option<SessionId>,
    system_prompt: Option<String>,
    model: Option<String>,
    max_tokens: Option<usize>,
}

// 链式调用 API
let conversation = Conversation::builder()
    .session_id(SessionId::new())
    .system_prompt("You are a helpful assistant")
    .model("claude-sonnet-4-6")
    .max_tokens(4096)
    .build();
```

---

## 三、Tools crate（工具层）

### 3.1 lib.rs - 工具注册

**默认工具注册表：**
```rust
pub fn default_registry() -> ToolRegistry {
    let mut registry = ToolRegistry::new();
    
    // 文件操作
    registry.register(Box::new(FileReadTool));
    registry.register(Box::new(FileWriteTool));
    registry.register(Box::new(FileEditTool));
    
    // 搜索
    registry.register(Box::new(GlobTool));
    registry.register(Box::new(GrepTool));
    
    // Shell
    registry.register(Box::new(BashTool::new()));
    
    // Web
    registry.register(Box::new(WebFetchTool));
    
    registry
}
```

---

### 3.2 bash.rs - Bash 工具

**BashTool（Bash 工具）：**
```rust
pub struct BashTool {
    timeout_seconds: u64,  // 超时时间（秒）
}
```

**构建方法：**
```rust
// 默认构造
BashTool::new()  // 默认 300 秒超时

// 自定义超时
BashTool::new().with_timeout(600)
```

**输入结构体：**
```rust
struct BashInput {
    command: String,                        // 要执行的命令
    cwd: Option<String>,                    // 工作目录
    env: Option<HashMap<String, String>>,   // 额外环境变量
    timeout: Option<u64>,                   // 覆盖默认超时
}
```

**输出结构体：**
```rust
struct BashOutput {
    stdout: String,     // 标准输出
    stderr: String,     // 标准错误
    exit_code: i32,     // 退出码
}
```

**执行流程：**
```rust
async fn execute(&self, input: ToolInput, ctx: &ToolContext) -> ToolResult {
    // 1. 解析输入
    let input: BashInput = input.parse()?;
    
    // 2. 确定工作目录
    let cwd = input.cwd.map(...).unwrap_or(ctx.working_directory);
    
    // 3. 构建命令
    let mut cmd = Command::new("bash");
    cmd.arg("-c")
       .arg(&input.command)
       .current_dir(&cwd)
       .stdout(Stdio::piped())
       .stderr(Stdio::piped())
       .stdin(Stdio::null());
    
    // 4. 设置环境变量
    cmd.env_clear();
    for (k, v) in &ctx.env_vars { cmd.env(k, v); }
    if let Some(env) = input.env { ... }
    
    // 5. 执行并设置超时
    let result = timeout(Duration::from_secs(timeout), cmd.output()).await;
    
    // 6. 处理结果
    match result {
        Ok(Ok(output)) => { /* 成功 */ }
        Ok(Err(e)) => { /* 执行失败 */ }
        Err(_) => { /* 超时 */ }
    }
}
```

---

## 四、CLI crate（命令行层）

### 4.1 config.rs - 配置管理

**CliConfig（CLI 配置）：**
```rust
pub struct CliConfig {
    pub api: ApiConfig,           // API 配置
    pub model: ModelConfig,       // 模型配置
    pub tools: ToolsConfig,       // 工具配置
    pub ui: UiConfig,             // UI 配置
    pub working_dir: Option<PathBuf>,  // 工作目录
    pub env_vars: HashMap<String, String>,  // 环境变量
}
```

**核心方法：**

| 方法 | 说明 |
|------|------|
| `load()` | 从默认位置加载配置 |
| `load_from_path(path)` | 从指定路径加载配置 |
| `config_dir()` | 获取配置目录（XDG 标准） |
| `data_dir()` | 获取数据目录（XDG 标准） |
| `create_default()` | 创建默认配置文件 |
| `get_api_key()` | 获取 API 密钥（检查环境变量和配置） |

**配置加载优先级：**
1. CLI 参数（最高优先级）
2. 环境变量（`ANTHROPIC_API_KEY`, `CLAUDE_MODEL` 等）
3. 配置文件
4. 默认值（最低优先级）

**merge 机制：**
```rust
fn merge(&mut self, other: CliConfig) {
    // 对于 Option 字段：other 有值则覆盖
    if other.api_key.is_some() { self.api_key = other.api_key; }
    
    // 对于基本类型：直接覆盖
    self.timeout_seconds = other.timeout_seconds;
    
    // 对于集合：合并而不是替换
    for (k, v) in other.env_vars { self.env_vars.insert(k, v); }
}
```

---

### 4.2 main.rs - CLI 入口

**Cli（命令行参数）：**
```rust
#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    config: Option<PathBuf>,          // 配置文件路径
    
    #[arg(short, long)]
    working_dir: Option<PathBuf>,     // 工作目录
    
    #[arg(long, env = "ANTHROPIC_API_KEY")]
    api_key: Option<String>,          // API 密钥（也可从环境变量读取）
    
    #[arg(short, long)]
    model: Option<String>,            // 模型名称
    
    #[arg(short, long)]
    verbose: bool,                    // 详细日志
    
    #[command(subcommand)]
    command: Option<Commands>,        // 子命令
}
```

**Commands（子命令）：**
```rust
enum Commands {
    Chat { message: Option<String> },  // 交互式对话
    Run { prompt: String },            // 单轮执行
    Tools,                             // 列出工具
    Config,                            // 显示配置
    Init { force: bool },              // 初始化配置
    Version,                           // 版本信息
}
```

**主流程：**
```rust
#[tokio::main]
async fn main() -> Result<()> {
    // 1. 解析命令行参数
    let cli = Cli::parse();
    
    // 2. 加载配置（文件 -> 环境变量）
    let mut config = if let Some(config_path) = &cli.config {
        CliConfig::load_from_path(config_path)?
    } else {
        CliConfig::load().unwrap_or_default()
    };
    
    // 3. CLI 参数覆盖配置
    if let Some(api_key) = cli.api_key { config.api.api_key = Some(api_key); }
    if let Some(model) = cli.model { config.model.name = model; }
    
    // 4. 初始化日志
    tracing_subscriber::fmt()...init();
    
    // 5. 处理子命令
    match cli.command {
        Some(Commands::Tools) => list_tools().await,
        Some(Commands::Config) => show_config(&config, &working_dir)?,
        Some(Commands::Init { force }) => init_config(force)?,
        Some(Commands::Run { prompt }) => run_single_prompt(...).await?,
        Some(Commands::Chat { message }) => run_interactive(...).await?,
        None => run_interactive(...).await?,  // 默认交互模式
    }
}
```

**交互模式流程：**
```rust
async fn run_interactive(api_key, model, working_dir, initial_message, config) {
    // 1. 打印欢迎信息
    println!("╔══════════════════════════════════════════╗");
    println!("║       Claude Code - Interactive Mode     ║");
    
    // 2. 初始化引擎
    let client = AnthropicClient::new(config)?;
    let mut conversation = Conversation::builder()...build();
    let mut tool_loop = ToolLoop::new(client);
    
    // 3. 注册工具（使用配置中的超时）
    tool_loop.register_tool(Box::new(
        BashTool::new().with_timeout(config.tools.bash.timeout_seconds)
    ));
    
    // 4. 处理初始消息（如果有）
    if let Some(msg) = initial_message { ... }
    
    // 5. 主循环
    loop {
        print!("> ");              // 显示提示符
        std::io::stdin().read_line(&mut input)?;  // 读取输入
        
        // 处理内置命令（支持完整命令和快捷方式）
        match input {
            "exit" | "quit" | ":q" => break,       // 退出
            "help" | ":h" => print_help(),          // 帮助
            "tools" | ":t" => list_tools().await,   // 列出工具
            "clear" | ":c" => print!("\x1B[2J\x1B[1;1H"), // 清屏
            "context" | ":ctx" => show_context(),   // 显示对话上下文
            _ => {
                // 发送到 Claude 处理
                conversation.add_user_message(input);
                tool_loop.run(&mut conversation, &tool_ctx).await?;
            }
        }
    }
}
```

---

## 五、开发流程详解

### 5.1 添加新工具的流程

**步骤 1：在 tools crate 创建新文件**
```rust
// crates/tools/src/my_tool.rs
use async_trait::async_trait;
use claude_core::{Tool, ToolInput, ToolOutput, ToolResult, ToolContext};

pub struct MyTool;

#[async_trait]
impl Tool for MyTool {
    fn name(&self) -> &str { "my_tool" }
    fn description(&self) -> &str { "描述工具功能" }
    fn input_schema(&self) -> Value { ... }
    fn permission_mode(&self) -> PermissionMode { PermissionMode::Ask }
    
    async fn execute(&self, input: ToolInput, ctx: &ToolContext) -> ToolResult {
        // 实现工具逻辑
        Ok(ToolOutput::success("成功"))
    }
}
```

**步骤 2：导出工具**
```rust
// crates/tools/src/lib.rs
pub mod my_tool;
pub use my_tool::MyTool;
```

**步骤 3：注册到默认注册表**
```rust
// crates/tools/src/lib.rs
pub fn default_registry() -> ToolRegistry {
    let mut registry = ToolRegistry::new();
    registry.register(Box::new(MyTool));
    registry
}
```

**步骤 4：在 CLI 中使用（可选）**
```rust
// crates/cli/src/main.rs
tool_loop.register_tool(Box::new(claude_tools::MyTool));
```

---

### 5.2 添加配置项的流程

**步骤 1：在 config.rs 添加字段**
```rust
// crates/cli/src/config.rs
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ToolsConfig {
    pub bash: BashConfig,
    pub my_tool: MyToolConfig,  // 新增
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct MyToolConfig {
    #[serde(default = "default_my_setting")]
    pub my_setting: u64,
}

fn default_my_setting() -> u64 { 100 }
```

**步骤 2：更新默认配置文件模板**
```toml
# crates/cli/config.default.toml
[tools.my_tool]
my_setting = 100
```

**步骤 3：在代码中使用配置**
```rust
// 读取配置值
let setting = config.tools.my_tool.my_setting;
```

---

### 5.3 CI/CD 流程

**持续集成（CI）：**
1. `cargo fmt --check` - 检查代码格式
2. `cargo clippy -- -D warnings` - 静态分析
3. `cargo check --all` - 编译检查
4. `cargo test --all` - 运行测试
5. `cargo build --release --target <target>` - 交叉编译

**发布流程（Release）：**
1. 推送标签 `git push origin v0.1.0`
2. GitHub Actions 自动触发
3. 创建 Release 页面
4. 并行构建 5 个平台的二进制文件
5. 生成 SHA256 校验和
6. 上传到 Release 页面

---

## 六、关键设计模式

### 6.1 Builder 模式
用于构建复杂对象，如 `ConversationBuilder`：
```rust
let conv = Conversation::builder()
    .session_id(id)
    .system_prompt(prompt)
    .model("claude-sonnet-4-6")
    .build();
```

### 6.2 Trait 对象
用于工具的多态：
```rust
pub trait Tool: Send + Sync { ... }
Box<dyn Tool>  // trait 对象，运行时多态
```

### 6.3 异步/等待
所有 IO 操作都是异步的：
```rust
async fn execute(&self, ...) -> ToolResult {
    let output = timeout(duration, cmd.output()).await?;
}
```

### 6.4 错误处理
使用 `anyhow` 和自定义错误类型：
```rust
type ClaudeResult<T> = Result<T, ClaudeError>;

#[derive(Error, Debug)]
pub enum ClaudeError {
    #[error("Network error: {0}")]
    Network(String),
    #[error("API error: {0}")]
    Api(String),
}
```

---

## 七、最佳实践

1. **配置优先**：所有可配置项都应支持配置文件、环境变量和 CLI 参数
2. **默认安全**：危险操作默认需要用户确认（PermissionMode::Ask）
3. **超时控制**：所有外部调用都应有超时机制
4. **错误传递**：使用 `?` 操作符简化错误处理
5. **日志记录**：使用 `tracing` crate 记录关键操作
6. **测试覆盖**：为每个工具编写单元测试

---

## 八、Memory crate（记忆系统）

### 8.1 概述

**Claude Memory** 是一个基于文件的持久化记忆系统，灵感来自 claude-code-main。

**核心特性：**
- **基于文件的存储**：每条记忆是一个独立的 Markdown 文件
- **四种记忆类型**：User、Feedback、Project、Reference
- **两步保存法**：写入独立文件 + 更新 MEMORY.md 索引
- **索引截断**：MEMORY.md 限制 200行/25KB，防止过大

**存储结构：**
```
~/.local/share/claude-code/memory/
└── projects/
    └── {project_slug}/
        └── memory/
            ├── MEMORY.md              # 索引文件
            ├── user_role.md           # 用户角色
            ├── feedback_testing.md    # 测试反馈
            └── reference_api_docs.md  # API文档链接
```

---

### 8.2 types.rs - 核心类型定义

**MemoryType（记忆类型枚举）：**
```rust
pub enum MemoryType {
    User,      // 用户信息：角色、偏好、知识水平
    Feedback,  // 反馈信息：工作方式指导
    Project,   // 项目信息：上下文、截止日期
    Reference, // 参考资料：外部系统链接
}
```

**核心结构体：**

| 结构体 | 说明 |
|--------|------|
| `Memory` | 完整的记忆（路径、frontmatter、内容、修改时间） |
| `MemoryHeader` | 轻量级头部（用于索引） |
| `MemoryFrontmatter` | 元数据（name、description、type） |
| `MemoryConfig` | 配置（路径、限制等） |

**类型方法：**
```rust
// 获取所有类型
MemoryType::all() -> &[MemoryType]

// 类型 ↔ 字符串转换
memory_type.as_str() -> "user" | "feedback" | ...
"user".parse::<MemoryType>()

// 获取系统提示词说明
memory_type.when_to_save() -> "何时保存这种记忆"
memory_type.how_to_use() -> "如何使用这种记忆"
```

---

### 8.3 storage.rs - 存储操作

**MemoryStorage（存储管理器）：**
```rust
pub struct MemoryStorage {
    config: MemoryConfig,
    project_slug: String,
}
```

**核心方法：**

| 方法 | 说明 |
|------|------|
| `new(config, project_slug)` | 创建存储实例 |
| `memory_dir()` | 获取记忆目录路径 |
| `entrypoint_path()` | 获取 MEMORY.md 路径 |
| `save_memory(filename, name, desc, type, content)` | 保存记忆文件 |
| `read_memory(filename)` | 读取记忆文件 |
| `scan_memories()` | 扫描所有记忆（返回 MemoryHeader 列表） |
| `delete_memory(filename)` | 删除记忆文件 |
| `read_entrypoint()` | 读取 MEMORY.md |
| `write_entrypoint(content)` | 写入 MEMORY.md（自动截断） |
| `update_entrypoint_index(filename, name, desc)` | 向索引添加条目 |

**文件格式：**
```markdown
---
name: 用户角色
description: 用户是资深 Rust 工程师
type: user
---

详细内容...
```

---

### 8.4 retrieval.rs - 记忆检索

**基于关键词的相关性搜索：**
```rust
pub struct MemoryRetriever;

impl MemoryRetriever {
    // 查找相关记忆
    pub async fn find_relevant(
        storage: &MemoryStorage,
        query: &str,
        limit: usize,
    ) -> Result<Vec<RetrievedMemory>>

    // 格式化记忆为提示词
    pub fn format_memories_for_prompt(memories: &[RetrievedMemory]) -> String

    // 获取所有记忆摘要
    pub async fn get_all_memories_summary(storage: &MemoryStorage) -> Result<String>
}
```

**评分机制：**
- 描述匹配: +0.5（权重最高）
- 文件名匹配: +0.3
- 类型匹配: +0.2

**输出格式：**
```markdown
## Relevant Memories

### user_role.md (85%)

---
name: 用户角色
description: 用户是资深工程师
type: user
---

用户有10年Rust经验...

---
```

---

### 8.5 frontmatter.rs - 元数据解析

**支持的格式：**

1. **YAML 格式**（默认）：
```yaml
---
name: value
description: value
type: user
---
```

2. **TOML 格式**（兼容）：
```toml
+++
name = "value"
description = "value"
type = "user"
+++
```

**核心函数：**
```rust
// 解析 frontmatter
pub fn parse_frontmatter(content: &str) -> (HashMap<String, String>, String)

// 生成 frontmatter
pub fn format_frontmatter(name: &str, description: &str, memory_type: &str) -> String
```

---

### 8.6 prompt.rs - 系统提示词构建

**build_memory_prompt 函数：**
```rust
pub async fn build_memory_prompt(storage: &MemoryStorage) -> Result<String>
```

**生成的提示词结构：**
```markdown
You have access to a memory system that persists across conversations...

## Memory System Overview

The memory system has four types:
- user: ...
- feedback: ...
- project: ...
- reference: ...

## Available Memories

- [user] user_role.md — 用户是资深工程师
- [reference] api_docs.md — API文档链接
```

---

### 8.7 使用示例

**完整使用流程：**
```rust
use claude_memory::{MemoryStorage, MemoryConfig, MemoryType, MemoryRetriever};

async fn example() -> Result<()> {
    // 1. 创建存储
    let config = MemoryConfig::default();
    let storage = MemoryStorage::new(config, "my-project");

    // 2. 保存记忆
    storage.save_memory(
        "user_role.md",
        "用户角色",
        "用户是资深 Rust 工程师",
        MemoryType::User,
        "详细内容...",
    ).await?;

    // 3. 更新索引
    storage.update_entrypoint_index(
        "user_role.md",
        "用户角色",
        "用户是资深 Rust 工程师",
    ).await?;

    // 4. 搜索相关记忆
    let results = MemoryRetriever::find_relevant(&storage, "rust", 5).await?;

    // 5. 生成系统提示词
    let prompt = claude_memory::build_memory_prompt(&storage).await?;

    Ok(())
}
```

---

### 8.8 配置说明

**MemoryConfig 默认值：**
```rust
MemoryConfig {
    base_dir: "~/.local/share/claude-code/memory",  // Linux
    max_entrypoint_lines: 200,    // MEMORY.md 最大行数
    max_entrypoint_bytes: 25_000, // MEMORY.md 最大字节
    max_memory_files: 200,        // 最多扫描文件数
}
```

**目录位置（按操作系统）：**
- Linux: `~/.local/share/claude-code/memory/`
- macOS: `~/Library/Application Support/com.anthropic.claude-code/memory/`
- Windows: `%APPDATA%/anthropic/claude-code/memory/`

---

## 九、扩展建议

1. **添加新工具**：按照 5.1 节的流程
2. **支持新模型**：更新 `ModelConfig` 和 API 调用
3. **添加 UI 组件**：在 `tui` crate 中实现
4. **会话持久化**：使用 `data_dir()` 保存对话历史
5. **记忆系统增强**：添加向量搜索、自动记忆提取
6. **插件系统**：动态加载外部工具
