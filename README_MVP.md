# Claude Code RS - MVP 版本

## 🎉 已实现功能

MVP 版本已经实现以下核心功能：

### ✅ 核心组件

1. **SSE 流式 API 处理**
   - 实时流式响应
   - 增量文本输出
   - 工具调用检测

2. **Tool Loop 对话循环**
   - 自动工具调用
   - 多轮对话支持
   - 流式输出到终端

3. **基础工具集**
   - BashTool - 执行 shell 命令
   - FileReadTool - 读取文件
   - FileWriteTool - 写入文件
   - FileEditTool - 编辑文件
   - GlobTool - 文件搜索
   - GrepTool - 内容搜索
   - WebFetchTool - 网页抓取

4. **交互式 CLI**
   - 交互模式
   - 单命令模式
   - 内置帮助系统

## 📦 构建状态

Release 构建正在进行中（Rust 编译优化需要几分钟）。

### 快速开始

```bash
# 1. 设置 API Key
export ANTHROPIC_API_KEY="your-api-key"

# 2. 等待构建完成后运行
./target/release/claude --help

# 3. 交互式模式
./target/release/claude

# 4. 单命令模式
./target/release/claude run "解释这个目录的代码结构"
```

## 💬 使用示例

### 交互式会话
```
╔══════════════════════════════════════════╗
║       Claude Code - Interactive Mode     ║
╚══════════════════════════════════════════╝
Type 'exit' or press Ctrl+C to quit

> 你好！
Claude: 你好！很高兴见到你。我可以帮你：
- 编写和修改代码
- 分析项目结构
- 运行命令和脚本
- 回答技术问题

请问有什么我可以帮助你的吗？

> 查看当前目录的文件
Claude: [Tool: bash]
[Tool result]: file1.txt file2.rs src/
...
```

### 单命令执行
```bash
$ ./target/release/claude run "统计代码行数"
Running: 统计代码行数

Claude: [Tool: bash]
> find . -name "*.rs" | xargs wc -l
...
[Conversation complete]
Tokens used: input=150, output=280
```

## 🔧 架构说明

```
┌─────────────────────────────────────────────────────────────┐
│                         CLI Layer                           │
│                    (crates/cli/src/main.rs)                  │
├─────────────────────────────────────────────────────────────┤
│                      Tool Loop                               │
│              (crates/engine/src/loop.rs)                     │
│  while conversation_active:                                  │
│    1. Send messages to API                                   │
│    2. Stream response events                                 │
│    3. Execute tools when requested                           │
│    4. Send tool results back                                 │
├─────────────────────────────────────────────────────────────┤
│                    Event Stream                              │
│            (crates/engine/src/stream.rs)                     │
│  - SSE parsing                                               │
│  - Text deltas                                               │
│  - Tool use events                                           │
├─────────────────────────────────────────────────────────────┤
│                   Tool Registry                              │
│              (crates/tools/src/registry.rs)                  │
│  - Bash, File, Search, Web tools                            │
└─────────────────────────────────────────────────────────────┘
```

## 📝 项目结构

```
crates/
├── core/       # 核心类型和 trait
├── engine/     # API 客户端和 Tool Loop
├── tools/      # 工具实现
└── cli/        # 命令行入口
```

## 🚧 待实现功能

### 高优先级
- [ ] 工具执行确认提示
- [ ] 错误重试机制
- [ ] 对话历史保存

### 中优先级
- [ ] 完整 TUI 界面 (ratatui)
- [ ] 语法高亮
- [ ] Token 成本控制

### 低优先级
- [ ] MCP 集成
- [ ] LSP 集成
- [ ] IDE 桥接

## 📚 参考文档

- [重构指南](./docs/REFACTORING_GUIDE.md)
- [MVP 实现详情](./docs/MVP_IMPLEMENTATION.md)
- [开发计划](./DEVELOPMENT_PLAN.md)

## 🤝 贡献

这是一个学习 Rust 的项目，欢迎提出问题或建议！
