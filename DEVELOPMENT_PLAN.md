# Claude Code RS 渐进式开发计划

## 开发理念

采用**增量式、可验证**的开发策略，每个阶段都有可运行的里程碑，而非一次性重写所有功能。

---

## Phase 1: 核心基础 (Week 1-2)

### 目标
建立可编译、可测试的基础架构，实现最基本的文件操作工具。

### 任务清单

- [x] 1.1 项目脚手架搭建
  - [x] Cargo workspace 配置
  - [x] 各 crate 目录结构
  - [x] 依赖版本锁定

- [x] 1.2 Core 类型系统
  - [x] Error 类型定义
  - [x] Tool trait 设计
  - [x] Message 类型
  - [x] Permission 模型

- [x] 1.3 基础工具实现
  - [x] FileReadTool
  - [x] FileWriteTool
  - [x] FileEditTool
  - [x] BashTool

- [x] 1.4 工具注册表
  - [x] ToolRegistry 实现
  - [x] 工具发现机制

- [x] 1.5 测试框架
  - [x] 单元测试基础设施
  - [x] 临时文件 fixtures
  - [x] CI 配置

### 验收标准
```bash
# 必须能编译成功
cargo build

# 必须通过所有测试
cargo test

# 必须能运行 CLI
cargo run -- --help
```

---

## Phase 2: 查询引擎 (Week 3-4)

### 目标
实现与 Anthropic API 的通信和流式响应处理。

### 任务清单

- [ ] 2.1 API 客户端
  - [ ] HTTP 客户端封装
  - [ ] 认证处理
  - [ ] 请求/响应序列化

- [ ] 2.2 流式处理
  - [ ] SSE 解析
  - [ ] 增量内容接收
  - [ ] Token 计数追踪

- [ ] 2.3 对话管理
  - [ ] Conversation 状态机
  - [ ] 消息历史维护
  - [ ] 上下文窗口管理

- [ ] 2.4 工具调用循环
  - [ ] ToolLoop 实现
  - [ ] 并行工具执行
  - [ ] 错误恢复

- [ ] 2.5 重试机制
  - [ ] 指数退避
  - [ ] 可重试错误识别
  - [ ] 熔断机制

### 验收标准
```bash
# API 客户端测试（需要 mock）
cargo test --package claude-engine

# 能进行简单对话（非交互式）
echo "Hello" | cargo run -- run "Say hi"
```

---

## Phase 3: 终端 UI (Week 5-6)

### 目标
实现基本的交互式终端界面。

### 任务清单

- [ ] 3.1 TUI 框架
  - [ ] ratatui 集成
  - [ ] 事件循环
  - [ ] 组件系统

- [ ] 3.2 核心组件
  - [ ] 输入框（带语法高亮）
  - [ ] 消息显示区域
  - [ ] 加载动画

- [ ] 3.3 会话界面
  - [ ] 对话历史滚动
  - [ ] 代码块渲染
  - [ ] 工具调用展示

- [ ] 3.4 快捷键
  - [ ] Ctrl+C 处理
  - [ ] Ctrl+D 退出
  - [ ] 历史导航

- [ ] 3.5 权限提示
  - [ ] 工具执行确认弹窗
  - [ ] 记住选择功能

### 验收标准
```bash
# 能启动交互式会话
cargo run -- chat

# 界面响应流畅
# 支持基本快捷键
```

---

## Phase 4: 搜索与高级工具 (Week 7-8)

### 目标
完善工具系统，实现代码搜索能力。

### 任务清单

- [x] 4.1 搜索工具
  - [x] GlobTool（文件模式匹配）
  - [x] GrepTool（内容搜索）

- [ ] 4.2 Git 集成
  - [ ] git2 库集成
  - [ ] 状态检测
  - [ ] diff 生成

- [ ] 4.3 高级文件操作
  - [ ] 图片/PDF 读取
  - [ ] Notebook 编辑
  - [ ] 大文件分块

- [ ] 4.4 Web 工具
  - [x] WebFetchTool
  - [ ] WebSearchTool（集成搜索 API）

- [ ] 4.5 性能优化
  - [ ] 并行文件读取
  - [ ] 结果缓存
  - [ ] 增量更新

---

## Phase 5: 服务层 (Week 9-10)

### 目标
实现外部服务集成。

### 任务清单

- [ ] 5.1 认证服务
  - [ ] OAuth 流程
  - [ ] API Key 管理
  - [ ] 密钥链集成

- [ ] 5.2 MCP 服务
  - [ ] MCP 客户端
  - [ ] 服务器发现
  - [ ] 工具桥接

- [ ] 5.3 LSP 集成
  - [ ] LSP 客户端
  - [ ] 符号查询
  - [ ] 代码补全

- [ ] 5.4 遥测
  - [ ] tracing 集成
  - [ ] OpenTelemetry 导出
  - [ ] 性能指标

---

## Phase 6: 命令系统 (Week 11-12)

### 目标
实现所有斜杠命令。

### 任务清单

- [ ] 6.1 核心命令
  - [ ] /commit
  - [ ] /cost
  - [ ] /config

- [ ] 6.2 会话命令
  - [ ] /compact
  - [ ] /resume
  - [ ] /clear

- [ ] 6.3 开发命令
  - [ ] /doctor
  - [ ] /review
  - [ ] /diff

- [ ] 6.4 扩展命令
  - [ ] /memory
  - [ ] /skills
  - [ ] /tasks

---

## Phase 7: IDE 桥接 (Week 13-14)

### 目标
实现与 VS Code/JetBrains 的集成。

### 任务清单

- [ ] 7.1 协议实现
  - [ ] WebSocket 服务器
  - [ ] 消息协议
  - [ ] 心跳机制

- [ ] 7.2 会话管理
  - [ ] 多会话支持
  - [ ] 会话恢复
  - [ ] 状态同步

- [ ] 7.3 VS Code 扩展
  - [ ] 扩展 API
  - [ ] 消息传递
  - [ ] UI 集成

---

## Phase 8: 多代理协调 (Week 15-16)

### 目标
实现 Agent Swarm 功能。

### 任务清单

- [ ] 8.1 Agent 管理
  - [ ] Agent 定义
  - [ ] 生命周期管理
  - [ ] 资源分配

- [ ] 8.2 消息路由
  - [ ] 消息总线
  - [ ] 广播/单播
  - [ ] 优先级队列

- [ ] 8.3 团队协作
  - [ ] Team 创建
  - [ ] 任务分配
  - [ ] 结果聚合

---

## Phase 9: 优化与发布 (Week 17-18)

### 目标
性能优化和发布准备。

### 任务清单

- [ ] 9.1 性能优化
  - [ ] 启动时间
  - [ ] 内存占用
  - [ ] 编译时间

- [ ] 9.2 测试覆盖
  - [ ] 集成测试
  - [ ] 端到端测试
  - [ ] 性能基准

- [ ] 9.3 发布准备
  - [ ] 安装脚本
  - [ ] 文档完善
  - [ ] 签名/分发

---

## 开发流程

### 1. 分支策略

```
main          # 稳定分支
  │
  ├── develop # 开发分支
  │     │
  │     ├── feature/phase-1-core
  │     ├── feature/phase-2-engine
  │     └── ...
  │
  └── release/v0.1.0
```

### 2. 提交规范

```
type(scope): subject

body

footer
```

Types:
- `feat`: 新功能
- `fix`: 修复
- `test`: 测试
- `docs`: 文档
- `refactor`: 重构
- `perf`: 性能

### 3. 代码审查

每个 PR 必须：
1. 通过 CI 检查
2. 有至少一个 Reviewer 批准
3. 更新相关文档
4. 包含测试

### 4. 测试策略

```rust
// 单元测试（每个模块）
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_feature() {
        // given
        let input = ...;
        
        // when
        let result = function(input).await;
        
        // then
        assert!(result.is_ok());
    }
}

// 集成测试（tests/ 目录）
#[tokio::test]
async fn test_end_to_end() {
    // 完整流程测试
}
```

### 5. 调试技巧

```bash
# 运行特定测试
cargo test --package claude-tools test_bash_echo -- --nocapture

# 启用详细日志
RUST_LOG=debug cargo run

# 检查编译时间
cargo build --timings

# 运行基准测试
cargo bench
```

---

## 参考文档

- [Original Source](../../claude-code-main/)
- [Architecture Notes](./docs/ARCHITECTURE.md)
- [API Reference](./docs/API.md)
