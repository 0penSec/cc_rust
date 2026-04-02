# GitHub 设置指南

## 快速设置步骤

### 1. 使用 GitHub CLI (推荐)

```bash
# 安装 GitHub CLI (如果还没有)
# macOS: brew install gh
# Ubuntu: sudo apt install gh
# 或参考: https://github.com/cli/cli#installation

# 登录 GitHub
github login

# 创建仓库并推送
github repo create 0penSec/cc_rust --public --source=. --push
```

### 2. 使用 SSH 密钥

```bash
# 生成 SSH 密钥 (如果还没有)
ssh-keygen -t ed25519 -C "your-email@example.com"

# 添加公钥到 GitHub
# 1. 复制公钥内容
cat ~/.ssh/id_ed25519.pub

# 2. 访问 https://github.com/settings/keys
# 3. 点击 "New SSH key"
# 4. 粘贴公钥内容

# 测试连接
ssh -T git@github.com

# 更新远程 URL 为 SSH
git remote set-url origin git@github.com:0penSec/cc_rust.git

# 推送
git push -u origin main
```

### 3. 使用 Personal Access Token (PAT)

```bash
# 1. 创建 PAT
# 访问: https://github.com/settings/tokens
# 点击 "Generate new token (classic)"
# 选择 scopes: repo

# 2. 使用 HTTPS 方式推送
git remote set-url origin https://YOUR_TOKEN@github.com/0penSec/cc_rust.git

# 3. 推送
git push -u origin main
```

## CI/CD 工作流程

一旦推送到 GitHub，以下工作流会自动运行：

### CI Workflow (`.github/workflows/ci.yml`)

**触发条件:**
- Push 到 `main` 或 `develop` 分支
- 创建 `v*` 标签
- Pull Request

**任务:**
1. **Check** - 代码格式检查、Clippy 检查
2. **Test** - 在 Linux/macOS/Windows 上运行测试
3. **Build** - 构建多平台二进制文件
4. **Docker** - 构建 Docker 镜像

### Release Workflow (`.github/workflows/release.yml`)

**触发条件:**
- 推送版本标签 (如 `v0.1.0`)

**任务:**
1. 创建 GitHub Release
2. 构建并上传多平台二进制文件:
   - Linux (x86_64, ARM64)
   - macOS (x86_64, Apple Silicon)
   - Windows (x86_64)
3. 生成 SHA256 校验和
4. 发布到 crates.io (可选)

## 发布新版本

```bash
# 1. 更新版本号 (在 Cargo.toml 中)
# 所有 crates 的 version 都要更新

# 2. 提交更改
git add -A
git commit -m "Release v0.1.0"

# 3. 创建标签
git tag -a v0.1.0 -m "Release version 0.1.0"

# 4. 推送标签
git push origin main
git push origin v0.1.0

# 5. CI/CD 会自动触发 release 工作流
```

## 本地测试 CI/CD

### 使用 act 工具

```bash
# 安装 act
# macOS: brew install act
# Linux: curl -s https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash

# 运行 CI 工作流
act -j check
act -j test
act -j build
```

### 使用 Docker

```bash
# 测试 Docker 构建
docker build -t claude-code-rs:test .
docker run --rm claude-code-rs:test --version
```

## 故障排除

### 推送被拒绝

```bash
# 如果提示权限错误，检查远程 URL
git remote -v

# 确保使用正确的认证方式
# SSH: git@github.com:0penSec/cc_rust.git
# HTTPS with PAT: https://TOKEN@github.com/0penSec/cc_rust.git
```

### CI 构建失败

1. 检查 `cargo check` 和 `cargo test` 本地是否通过
2. 查看 GitHub Actions 日志
3. 检查是否有环境变量或 secrets 需要设置

### Release 资产未上传

确保标签格式正确:
- ✅ `v0.1.0`
- ✅ `v1.0.0-alpha.1`
- ❌ `0.1.0` (缺少 v 前缀)

## Secrets 配置

如果需要设置 secrets:

```bash
# 使用 GitHub CLI
github secret set ANTHROPIC_API_KEY --repo 0penSec/cc_rust

# 或者在仓库设置中手动添加
# Settings -> Secrets and variables -> Actions -> New repository secret
```

## 状态徽章

README 中的徽章会自动更新:

- CI 状态: 显示最新 CI 运行结果
- Release: 显示最新发布版本
- License: 显示许可证信息
