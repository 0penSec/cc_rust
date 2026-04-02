# 架构设计文档

## 核心设计决策

### 1. Trait-based 工具系统

参考原 TypeScript 的动态工具注册，Rust 版本使用 trait 系统实现类型安全：

```rust
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn input_schema(&self) -> Value;
    fn permission_mode(&self) -> PermissionMode;
    async fn execute(&self, input: ToolInput, ctx: &ToolContext) -> ToolResult;
}
```

**设计理由**：
- 编译期类型安全
- 零成本抽象
- 易于测试（mock 实现）

### 2. Actor 模型消息处理

对话引擎采用类似 Actor 的设计：

```
┌─────────────┐     ┌──────────────┐     ┌─────────────┐
│   Session   │────▶│  QueryEngine │────▶│  Anthropic  │
│  (Actor)    │◀────│  (Orchestrator)│◀────│    API      │
└─────────────┘     └──────────────┘     └─────────────┘
        │
        ▼
┌─────────────┐
│  ToolLoop   │
│ (Executor)  │
└─────────────┘
```

### 3. 流式响应处理

使用 Rust 的 Stream trait 处理 SSE：

```rust
pub struct EventStream {
    inner: Pin<Box<dyn Stream<Item = Result<StreamEvent, reqwest::Error>> + Send>>,
}
```

对比原版的 Node.js EventEmitter，更加类型安全且内存高效。

### 4. 权限系统

三层权限模型：

1. **Bypass** - 内部工具，无需确认
2. **Auto** - 基于上下文自动决策
3. **Ask** - 总是询问用户

```rust
pub enum PermissionMode {
    Bypass,  // 编译期常量
    Auto,    // 运行时决策
    Ask,     // 用户交互
}
```

## 性能考虑

### 启动优化

原版启动时预取 MDM 设置和 Keychain，Rust 版本同样：

```rust
#[tokio::main]
async fn main() {
    // 并行预取
    let (config, auth) = tokio::join!(
        load_config_async(),
        prefetch_auth_async(),
    );
    // ...
}
```

### 内存管理

- 使用 `Arc<str>` 替代 `String` 减少克隆
- 对话历史使用 LRU 缓存
- 大文件读取使用流式处理

### 并发模型

```
┌─────────────────────────────────────┐
│           tokio runtime             │
│  ┌─────────┐ ┌─────────┐ ┌────────┐ │
│  │ Tool A  │ │ Tool B  │ │ Tool C │ │  // 并行工具执行
│  └────┬────┘ └────┬────┘ └───┬────┘ │
│       └─────────────┴─────────┘      │
│                  │                   │
│           ┌──────┴──────┐            │
│           │  Coordinator │            │
│           └─────────────┘            │
└─────────────────────────────────────┘
```

## 模块依赖关系

```
                    ┌─────────┐
                    │   cli   │
                    └────┬────┘
           ┌─────────────┼─────────────┐
           ▼             ▼             ▼
      ┌─────────┐   ┌─────────┐   ┌─────────┐
      │   tui   │   │ commands│   │  bridge │
      └────┬────┘   └────┬────┘   └────┬────┘
           │             │             │
           └─────────────┼─────────────┘
                         ▼
                    ┌─────────┐
                    │  engine │
                    └────┬────┘
           ┌─────────────┼─────────────┐
           ▼             ▼             ▼
      ┌─────────┐   ┌─────────┐   ┌─────────┐
      │  tools  │   │ services│   │coordinator
      └────┬────┘   └────┬────┘   └────┬────┘
           │             │             │
           └─────────────┴─────────────┘
                         │
                         ▼
                    ┌─────────┐
                    │  core   │
                    └─────────┘
```

## 与原版的差异

### 改进点

| 方面 | TypeScript | Rust |
|------|------------|------|
| 错误处理 | try/catch 运行时 | Result 编译期 |
| 异步 | Promise/callback | async/await + Stream |
| 类型安全 | 运行时检查 | 编译期保证 |
| 内存 | GC 管理 | 显式所有权 |
| 部署 | 依赖 Node/Bun | 单二进制文件 |

### 妥协点

1. **编译时间** - Rust 编译慢于 TypeScript transpile
2. **生态成熟度** - TUI 生态不如 React 成熟
3. **开发速度** - 类型系统更严格，开发初期较慢

## 扩展性设计

### 添加新工具

```rust
// 1. 在 tools/src/ 创建新模块
pub struct MyTool;

#[async_trait]
impl Tool for MyTool {
    fn name(&self) -> &str { "my_tool" }
    // ...
}

// 2. 在 lib.rs 注册
registry.register(Box::new(MyTool));
```

### 添加新命令

```rust
// 使用 clap derive macro
#[derive(Subcommand)]
enum Commands {
    MyCommand { arg: String },
}
```

## 安全考虑

1. **沙箱执行** - Bash 工具限制环境变量和工作目录
2. **路径验证** - 所有文件操作解析绝对路径
3. **输入验证** - JSON Schema 验证所有工具输入
4. **密钥管理** - API Key 使用 keyring 存储

## 测试策略

```
Unit Tests (crate/src/*.rs)
    │
    ▼
Integration Tests (tests/*.rs)
    │
    ▼
E2E Tests (scripts/e2e/)
```

每个 Tool 实现包含：
- 正常流程测试
- 错误处理测试
- 边界条件测试
