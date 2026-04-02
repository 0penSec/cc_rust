# Claude Code Rust 重构完整指南

基于 claude-code-main (TypeScript) 到 claude-code-rs (Rust) 的重构分析与学习路线图

---

## 一、源码结构分析

### 1.1 TypeScript 原版架构

```
claude-code-main/src/
├── main.tsx                    # 入口点 (803KB - 庞大!)
├── Tool.ts                     # 工具核心抽象
├── tools.ts                    # 工具注册
├── Tool.ts                     # 工具类型定义
├── query.ts                    # AI 查询引擎
├── QueryEngine.ts              # 查询引擎实现
├── commands.ts                 # 斜杠命令
├── commands/                   # 各命令实现
│   ├── commit/
│   ├── config/
│   ├── cost/
│   ├── doctor/
│   └── ... (40+ 命令)
├── tools/                      # 工具实现
│   ├── BashTool/
│   ├── FileReadTool/
│   ├── FileWriteTool/
│   ├── FileEditTool/
│   ├── GrepTool/
│   ├── GlobTool/
│   ├── AgentTool/
│   ├── WebFetchTool/
│   ├── WebSearchTool/
│   └── ... (50+ 工具)
├── components/                 # React 组件
├── services/                   # 外部服务
│   ├── api/                    # Anthropic API
│   ├── mcp/                    # MCP 服务
│   ├── lsp/                    # LSP 集成
│   └── ...
├── state/                      # 状态管理
├── context/                    # React Context
├── hooks/                      # 自定义 Hooks
├── utils/                      # 工具函数
└── types/                      # 类型定义
```

### 1.2 核心依赖分析

| 类别 | TypeScript 依赖 | 说明 |
|------|----------------|------|
| 运行时 | Bun | JavaScript 运行时 |
| AI SDK | `@anthropic-ai/sdk` | Anthropic API 客户端 |
| CLI | `ink` | React-based TUI 框架 |
| 状态 | React Context + hooks | 状态管理 |
| HTTP | 原生 fetch | HTTP 请求 |
| 序列化 | Zod | 运行时类型验证 |
| 文件系统 | Node.js fs | 文件操作 |
| 进程 | child_process | 命令执行 |

---

## 二、核心功能清单

### 2.1 工具系统 (Tools)

#### 文件操作工具
- **FileReadTool**: 读取文件内容，支持偏移量和限制
- **FileWriteTool**: 写入文件，支持图片
- **FileEditTool**: 编辑文件（diff-based）
- **GlobTool**: 文件模式匹配搜索
- **GrepTool**: 内容搜索（基于 ripgrep）

#### 系统工具
- **BashTool**: 执行 shell 命令
- **LSPTool**: LSP 符号查询
- **NotebookEditTool**: Jupyter notebook 编辑

#### 网络工具
- **WebFetchTool**: 抓取网页
- **WebSearchTool**: 网络搜索

#### 高级工具
- **AgentTool**: 子代理管理
- **MCPTool**: MCP 服务器工具调用
- **TaskCreateTool**: 后台任务创建
- **CronCreateTool/CronDeleteTool**: 定时任务

### 2.2 命令系统 (Slash Commands)

```
/commit      - 创建 git commit
/config      - 配置管理
/cost        - 成本统计
/compact     - 对话压缩
/clear       - 清除对话
/doctor      - 诊断检查
/diff        - 显示 diff
/help        - 帮助
/login       - 登录
/logout      - 登出
/memory      - 记忆管理
/review      - 代码审查
/resume      - 恢复会话
/status      - 状态显示
/tasks       - 任务管理
/theme       - 主题切换
/usage       - 使用量
... (50+ 命令)
```

### 2.3 对话引擎 (Query Engine)

```
┌─────────────────────────────────────────────────────────────┐
│                    Query Engine Flow                        │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────┐    ┌──────────────┐    ┌─────────────────┐   │
│  │  User    │───▶│  Messages    │───▶│  System Prompt  │   │
│  │  Input   │    │  + Context   │    │  + Attachments  │   │
│  └──────────┘    └──────────────┘    └─────────────────┘   │
│                                               │             │
│                                               ▼             │
│  ┌──────────┐    ┌──────────────┐    ┌─────────────────┐   │
│  │  Tool    │◀───│  API Stream  │◀───│  Anthropic API  │   │
│  │  Results │    │  (SSE)       │    │  (Claude)       │   │
│  └──────────┘    └──────────────┘    └─────────────────┘   │
│       │                                                     │
│       ▼                                                     │
│  ┌──────────┐    ┌──────────────┐    ┌─────────────────┐   │
│  │  Tool    │───▶│  Execute     │───▶│  Update UI      │   │
│  │  Calls   │    │  (Parallel)  │    │                 │   │
│  └──────────┘    └──────────────┘    └─────────────────┘   │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### 2.4 关键子系统

1. **权限系统 (Permissions)**
   - 自动模式 / 询问模式 / 拒绝模式
   - 基于模式的权限规则
   - 沙箱支持

2. **状态管理 (State)**
   - 对话历史
   - 工具调用状态
   - 应用配置
   - 文件缓存

3. **服务层 (Services)**
   - API 客户端（重试、限流）
   - MCP 服务发现
   - LSP 集成
   - 遥测/分析

4. **渲染层 (UI)**
   - 消息渲染
   - 工具调用显示
   - 进度指示
   - 输入处理

---

## 三、Rust 项目结构设计

### 3.1 Workspace 架构

```
claude-code-rs/
├── Cargo.toml                    # Workspace 根配置
├── crates/
│   ├── core/                     # 核心类型和 trait
│   │   ├── src/
│   │   │   ├── lib.rs           # 模块导出
│   │   │   ├── error.rs         # 错误类型
│   │   │   ├── tool.rs          # Tool trait
│   │   │   ├── message.rs       # 消息类型
│   │   │   ├── permission.rs    # 权限模型
│   │   │   ├── context.rs       # 上下文类型
│   │   │   └── types.rs         # 通用类型
│   │   └── Cargo.toml
│   │
│   ├── tools/                    # 工具实现
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── registry.rs      # 工具注册表
│   │   │   ├── bash.rs          # BashTool
│   │   │   ├── file.rs          # FileRead/Write/Edit
│   │   │   ├── search.rs        # Glob/Grep
│   │   │   ├── web.rs           # WebFetch/Search
│   │   │   ├── agent.rs         # AgentTool
│   │   │   └── mcp.rs           # MCPTool
│   │   └── Cargo.toml
│   │
│   ├── engine/                   # 查询引擎
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── client.rs        # API 客户端
│   │   │   ├── stream.rs        # SSE 流处理
│   │   │   ├── conversation.rs  # 对话管理
│   │   │   ├── loop.rs          # 工具调用循环
│   │   │   ├── retry.rs         # 重试逻辑
│   │   │   └── token.rs         # Token 计数
│   │   └── Cargo.toml
│   │
│   ├── commands/                 # 斜杠命令
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── registry.rs      # 命令注册表
│   │   │   ├── commit.rs
│   │   │   ├── config.rs
│   │   │   ├── cost.rs
│   │   │   └── ...
│   │   └── Cargo.toml
│   │
│   ├── services/                 # 外部服务
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── api/             # Anthropic API
│   │   │   ├── mcp/             # MCP 服务
│   │   │   ├── lsp/             # LSP 客户端
│   │   │   ├── auth/            # 认证服务
│   │   │   └── telemetry/       # 遥测服务
│   │   └── Cargo.toml
│   │
│   ├── tui/                      # 终端 UI
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── app.rs           # TUI 应用主循环
│   │   │   ├── ui.rs            # UI 布局
│   │   │   ├── event.rs         # 事件处理
│   │   │   └── components/      # UI 组件
│   │   │       ├── input.rs
│   │   │       ├── message_list.rs
│   │   │       └── spinner.rs
│   │   └── Cargo.toml
│   │
│   ├── coordinator/              # 多代理协调
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── agent.rs         # Agent 定义
│   │   │   ├── team.rs          # 团队管理
│   │   │   └── router.rs        # 消息路由
│   │   └── Cargo.toml
│   │
│   ├── bridge/                   # IDE 桥接
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── server.rs        # WebSocket 服务器
│   │   │   └── protocol.rs      # 通信协议
│   │   └── Cargo.toml
│   │
│   └── cli/                      # CLI 入口
│       ├── src/
│       │   ├── main.rs          # 程序入口
│       │   ├── args.rs          # 参数解析
│       │   └── commands/        # 子命令
│       └── Cargo.toml
│
├── tests/                        # 集成测试
├── docs/                         # 文档
└── scripts/                      # 构建脚本
```

### 3.2 核心依赖建议

```toml
[workspace.dependencies]
# 异步运行时
tokio = { version = "1.40", features = ["full"] }
futures = "0.3"
async-trait = "0.1"

# CLI
clap = { version = "4.5", features = ["derive"] }

# TUI
ratatui = "0.29"
crossterm = "0.28"

# HTTP/API
reqwest = { version = "0.12", features = ["json", "stream"] }
reqwest-eventsource = "0.6"

# 序列化
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
schemars = "0.8"

# 错误处理
thiserror = "2.0"
anyhow = "1.0"

# 日志/追踪
tracing = "0.1"
tracing-subscriber = "0.3"

# 文本处理
regex = "1.11"
globset = "0.4"
ignore = "0.4"

# 并发
dashmap = "6.1"
parking_lot = "0.12"

# 其他
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.11", features = ["v4", "serde"] }
tempfile = "3.14"
```

---

## 四、分阶段重构步骤

### Phase 1: 核心基础 (第 1-2 周)

#### 目标
建立可编译、可测试的基础架构，实现最基本的工具。

#### 任务清单

1. **项目脚手架搭建**
   ```bash
   cargo new claude-code-rs --lib
   cd claude-code-rs
   # 创建 workspace
   ```

2. **Core 类型系统实现**
   - [ ] `Tool` trait 设计
   - [ ] `Message` 类型（枚举变体）
   - [ ] `Error` 类型（使用 thiserror）
   - [ ] `Permission` 模型

3. **基础工具实现**
   - [ ] `FileReadTool`
   - [ ] `FileWriteTool`
   - [ ] `BashTool` (基础版)

4. **测试框架**
   - [ ] 单元测试结构
   - [ ] 临时文件 fixtures

#### 示例代码: Tool Trait

```rust
// crates/core/src/tool.rs
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::any::Any;

#[async_trait]
pub trait Tool: Send + Sync {
    /// 工具名称
    fn name(&self) -> &'static str;
    
    /// 输入 Schema (JSON Schema)
    fn input_schema(&self) -> serde_json::Value;
    
    /// 执行工具
    async fn call(
        &self,
        input: serde_json::Value,
        context: &ToolContext,
    ) -> Result<ToolResult, ToolError>;
    
    /// 是否只读
    fn is_read_only(&self) -> bool {
        false
    }
    
    /// 是否并发安全
    fn is_concurrent_safe(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub content: String,
    pub is_error: bool,
}

pub struct ToolContext {
    pub working_dir: std::path::PathBuf,
    pub abort_signal: tokio::sync::watch::Receiver<bool>,
}
```

---

### Phase 2: 查询引擎 (第 3-4 周)

#### 目标
实现与 Anthropic API 的通信和流式响应处理。

#### 任务清单

1. **API 客户端**
   - [ ] HTTP 客户端封装
   - [ ] 认证处理 (API Key)
   - [ ] 请求/响应序列化

2. **流式处理**
   - [ ] SSE (Server-Sent Events) 解析
   - [ ] 增量内容接收
   - [ ] Token 计数追踪

3. **对话管理**
   - [ ] Conversation 结构
   - [ ] 消息历史维护
   - [ ] 上下文窗口计算

4. **工具调用循环**
   - [ ] ToolLoop 实现
   - [ ] 并行工具执行

#### 示例代码: SSE 流处理

```rust
// crates/engine/src/stream.rs
use reqwest_eventsource::{Event, EventSource};
use futures::StreamExt;

pub struct StreamHandler {
    client: reqwest::Client,
    api_key: String,
}

impl StreamHandler {
    pub async fn stream_response(
        &self,
        request: AnthropicRequest,
    ) -> Result<impl futures::Stream<Item = Result<StreamEvent, Error>>, Error> {
        let mut es = EventSource::new(
            self.client
                .post("https://api.anthropic.com/v1/messages")
                .header("x-api-key", &self.api_key)
                .json(&request)
        )?;
        
        let stream = async_stream::try_stream! {
            while let Some(event) = es.next().await {
                match event {
                    Ok(Event::Message(message)) => {
                        let event: StreamEvent = serde_json::from_str(&message.data)?;
                        yield event;
                    }
                    Ok(Event::Open) => continue,
                    Err(e) => Err(e)?,
                }
            }
        };
        
        Ok(stream)
    }
}
```

---

### Phase 3: 终端 UI (第 5-6 周)

#### 目标
实现基本的交互式终端界面。

#### 任务清单

1. **TUI 框架集成**
   - [ ] ratatui 初始化
   - [ ] 事件循环

2. **核心组件**
   - [ ] 输入框 (带历史)
   - [ ] 消息显示区域
   - [ ] 加载动画

3. **会话界面**
   - [ ] 对话历史滚动
   - [ ] 代码块渲染
   - [ ] 工具调用展示

4. **快捷键**
   - [ ] Ctrl+C 中断
   - [ ] Ctrl+D 退出
   - [ ] 上下历史导航

#### 示例代码: TUI 主循环

```rust
// crates/tui/src/app.rs
use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    DefaultTerminal,
};

pub struct App {
    input: String,
    messages: Vec<Message>,
    should_quit: bool,
}

impl App {
    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<(), Error> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;
            
            if event::poll(std::time::Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char(c) => self.input.push(c),
                        KeyCode::Backspace => { self.input.pop(); }
                        KeyCode::Enter => self.submit().await?,
                        KeyCode::Up => self.history_prev(),
                        KeyCode::Down => self.history_next(),
                        KeyCode::Char('c') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                            self.interrupt().await;
                        }
                        KeyCode::Char('d') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                            self.should_quit = true;
                        }
                        _ => {}
                    }
                }
            }
            
            if self.should_quit {
                break Ok(());
            }
        }
    }
}
```

---

### Phase 4: 搜索与高级工具 (第 7-8 周)

#### 目标
完善工具系统，实现代码搜索能力。

#### 任务清单

1. **搜索工具**
   - [ ] `GlobTool` (使用 globset)
   - [ ] `GrepTool` (使用 ripgrep 库)

2. **Git 集成**
   - [ ] git2 库集成
   - [ ] 状态检测
   - [ ] diff 生成

3. **高级文件操作**
   - [ ] 图片/PDF 读取
   - [ ] Notebook 编辑

4. **Web 工具**
   - [ ] `WebFetchTool`
   - [ ] `WebSearchTool`

---

### Phase 5: 服务层 (第 9-10 周)

#### 目标
实现外部服务集成。

#### 任务清单

1. **认证服务**
   - [ ] API Key 管理
   - [ ] 密钥链集成 (keyring)

2. **MCP 服务**
   - [ ] MCP 客户端
   - [ ] 服务器发现

3. **LSP 集成**
   - [ ] tower-lsp 客户端
   - [ ] 符号查询

4. **遥测**
   - [ ] tracing 集成
   - [ ] OpenTelemetry

---

### Phase 6: 命令系统 (第 11-12 周)

#### 目标
实现所有斜杠命令。

#### 任务清单

1. **核心命令**
   - [ ] `/commit` - 使用 git2
   - [ ] `/cost` - Token 统计
   - [ ] `/config` - 配置管理

2. **会话命令**
   - [ ] `/compact`
   - [ ] `/resume`
   - [ ] `/clear`

3. **开发命令**
   - [ ] `/doctor`
   - [ ] `/review`
   - [ ] `/diff`

---

### Phase 7+: IDE 桥接、多代理、优化

按照 DEVELOPMENT_PLAN.md 继续后续阶段...

---

## 五、Rust 特性学习路线图

### 5.1 Phase 1 - 掌握的核心概念

#### 所有权与借用
```rust
// 实践: Tool 注册表
pub struct ToolRegistry {
    // HashMap 拥有 Tools
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    // 借用检查: &self 只读借用
    pub fn get(&self, name: &str) -> Option<&dyn Tool> {
        self.tools.get(name).map(|t| t.as_ref())
    }
    
    // 所有权转移: tool 被 move 进 registry
    pub fn register(&mut self, tool: Box<dyn Tool>) {
        let name = tool.name().to_string();
        self.tools.insert(name, tool);
    }
}
```

#### 错误处理 (thiserror)
```rust
// crates/core/src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ToolError {
    #[error("工具未找到: {0}")]
    NotFound(String),
    
    #[error("执行失败: {0}")]
    ExecutionFailed(String),
    
    #[error("权限被拒绝")]
    PermissionDenied,
    
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("JSON 错误: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, ToolError>;
```

#### Trait 与泛型
```rust
// 实践: 泛型工具输入
pub trait Input: for<'de> Deserialize<'de> + Send + Sync {
    fn validate(&self) -> Result<()>;
}

pub struct ToolCall<I: Input, O: Output> {
    input: I,
    _output: PhantomData<O>,
}
```

### 5.2 Phase 2 - 异步与并发

#### async/await
```rust
// 实践: 并行工具执行
use futures::future::join_all;

pub async fn execute_tools_parallel(
    calls: Vec<ToolCall>,
    context: &ToolContext,
) -> Vec<Result<ToolResult>> {
    let futures = calls.into_iter().map(|call| {
        let ctx = context.clone();
        async move {
            call.execute(&ctx).await
        }
    });
    
    join_all(futures).await
}
```

#### 取消传播
```rust
// 实践: 优雅取消
use tokio::select;

pub async fn call_with_timeout<T>(
    fut: impl Future<Output = T>,
    abort_rx: watch::Receiver<bool>,
    timeout: Duration,
) -> Result<T> {
    select! {
        result = fut => Ok(result),
        _ = sleep(timeout) => Err(Error::Timeout),
        _ = abort_rx.changed() => {
            if *abort_rx.borrow() {
                Err(Error::Cancelled)
            } else {
                unreachable!()
            }
        }
    }
}
```

#### 流处理
```rust
// 实践: SSE 流
use tokio_stream::StreamExt;

let mut stream = client.stream_events(request).await?;

while let Some(event) = stream.next().await {
    match event {
        Ok(StreamEvent::ContentBlock { text }) => {
            print!("{}", text);
        }
        Ok(StreamEvent::ToolUse { name, input }) => {
            let result = execute_tool(name, input).await?;
            client.send_result(result).await?;
        }
        Err(e) => eprintln!("错误: {}", e),
    }
}
```

### 5.3 Phase 3 - 生命周期与高级类型

#### 生命周期标注
```rust
// 实践: 消息引用
pub struct MessageView<'a> {
    messages: &'a [Message],
    range: Range<usize>,
}

impl<'a> MessageView<'a> {
    // 返回的切片生命周期与 &self 相同
    pub fn visible(&self) -> &'a [Message] {
        &self.messages[self.range.clone()]
    }
}
```

#### Cow (Clone on Write)
```rust
// 实践: 避免不必要克隆
use std::borrow::Cow;

pub fn format_message(msg: &Message) -> Cow<'_, str> {
    if msg.content.len() < 100 {
        // 小消息直接引用
        Cow::Borrowed(&msg.content)
    } else {
        // 大消息需要处理
        Cow::Owned(truncate(&msg.content, 100))
    }
}
```

#### 类型状态模式
```rust
// 实践: 构建器模式
pub struct ConversationBuilder<S> {
    model: String,
    messages: Vec<Message>,
    _state: PhantomData<S>,
}

pub struct Uninitialized;
pub struct Ready;

impl ConversationBuilder<Uninitialized> {
    pub fn new() -> Self {
        Self {
            model: String::new(),
            messages: Vec::new(),
            _state: PhantomData,
        }
    }
    
    pub fn model(self, model: impl Into<String>) -> ConversationBuilder<Ready> {
        ConversationBuilder {
            model: model.into(),
            messages: self.messages,
            _state: PhantomData,
        }
    }
}

impl ConversationBuilder<Ready> {
    pub fn build(self) -> Conversation {
        Conversation {
            model: self.model,
            messages: self.messages,
        }
    }
}
```

### 5.4 Phase 4+ - 高级模式

#### Actor 模式 (多代理)
```rust
// 实践: Agent 协调
use tokio::sync::mpsc;

pub struct Agent {
    id: AgentId,
    inbox: mpsc::Receiver<Message>,
    outbox: mpsc::Sender<AgentEvent>,
}

impl Agent {
    pub async fn run(mut self) {
        while let Some(msg) = self.inbox.recv().await {
            match msg {
                Message::Task { content, reply_to } => {
                    let result = self.process_task(content).await;
                    let _ = reply_to.send(result);
                }
                Message::Shutdown => break,
                _ => {}
            }
        }
    }
}
```

#### 无锁数据结构
```rust
// 实践: 共享状态
use dashmap::DashMap;
use parking_lot::RwLock;

pub struct SharedState {
    // 并发安全的工具缓存
    tool_cache: DashMap<String, ToolResult>,
    // 对话历史 (读多写少)
    conversation: RwLock<Vec<Message>>,
}
```

#### 宏编程
```rust
// 实践: 工具定义宏
#[macro_export]
macro_rules! define_tool {
    (
        name: $name:expr,
        input: $input:ty,
        output: $output:ty,
        |$ctx:ident, $input_var:ident| $body:block
    ) => {
        pub struct $name;
        
        #[async_trait]
        impl Tool for $name {
            fn name(&self) -> &'static str { $name }
            
            async fn call(
                &self,
                $input_var: serde_json::Value,
                $ctx: &ToolContext,
            ) -> Result<ToolResult> {
                let input: $input = serde_json::from_value($input_var)?;
                let result: $output = $body;
                Ok(ToolResult {
                    content: serde_json::to_string(&result)?,
                    is_error: false,
                })
            }
        }
    };
}
```

---

## 六、关键设计决策

### 6.1 错误处理策略

```rust
// 分层错误处理
// 1. 底层: thiserror (可恢复错误)
// 2. 应用层: anyhow (快速传播)
// 3. 用户层: 友好错误消息

// 底层 crate (core, tools)
#[derive(Error, Debug)]
pub enum CoreError { ... }

// 应用层 (cli, tui)
fn main() -> anyhow::Result<()> {
    // 使用 ? 快速传播
    let result = run_app().context("应用运行失败")?;
    Ok(result)
}
```

### 6.2 状态管理

```rust
// 不可变状态 + 消息传递
// 优于: 可变共享状态

pub enum AppEvent {
    UserInput(String),
    ToolStart { id: Uuid, name: String },
    ToolProgress { id: Uuid, data: ProgressData },
    ToolComplete { id: Uuid, result: ToolResult },
    StreamChunk(String),
    StreamComplete,
}

// 单一状态更新点
pub fn update(state: &mut AppState, event: AppEvent) {
    match event {
        AppEvent::UserInput(text) => { ... }
        AppEvent::ToolStart { id, name } => { ... }
        ...
    }
}
```

### 6.3 测试策略

```rust
// 单元测试 (内联)
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_bash_echo() {
        let tool = BashTool::new();
        let result = tool
            .call(
                json!({ "command": "echo hello" }),
                &ToolContext::test(),
            )
            .await
            .unwrap();
        
        assert!(result.content.contains("hello"));
    }
}

// 集成测试 (tests/ 目录)
#[tokio::test]
async fn test_conversation_flow() {
    let mut conv = Conversation::new();
    conv.add_message(Message::user("Hello"));
    
    let response = conv.complete().await.unwrap();
    
    assert!(!response.content.is_empty());
}
```

---

## 七、性能优化建议

### 7.1 编译时间优化

```toml
# Cargo.toml
[profile.dev]
opt-level = 0
debug = true
incremental = true

[profile.dev.package."*"]
opt-level = 2  # 依赖优化

[profile.release]
lto = "thin"
codegen-units = 1
strip = true
```

### 7.2 运行时性能

```rust
// 使用 String::with_capacity 避免重新分配
// 使用 Arc<str> 共享字符串
// 使用 Bytes 处理二进制数据

// 避免锁争用
use crossbeam::channel;  // 优于 std::sync::mpsc
use parking_lot::Mutex;  // 更快，可中毒恢复
```

---

## 八、总结

### 学习检查清单

- [ ] 理解所有权系统
- [ ] 掌握 async/await
- [ ] 熟练使用 trait
- [ ] 掌握生命周期标注
- [ ] 理解并发模型
- [ ] 熟悉错误处理模式
- [ ] 掌握宏编程基础

### 推荐资源

1. **The Rust Programming Language** (官方)
2. **Rust for Rustaceans** (高级)
3. **Async Rust** (异步)
4. **Rust Design Patterns** (设计模式)

### 实践建议

1. 每个 Phase 完成后做 Code Review
2. 写测试验证理解
3. 使用 `cargo clippy` 学习最佳实践
4. 阅读优秀 crate 源码 (tokio, axum, ratatui)
