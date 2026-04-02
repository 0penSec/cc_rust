# Claude Code RS

[![CI](https://github.com/0penSec/cc_rust/actions/workflows/ci.yml/badge.svg)](https://github.com/0penSec/cc_rust/actions/workflows/ci.yml)
[![Release](https://github.com/0penSec/cc_rust/actions/workflows/release.yml/badge.svg)](https://github.com/0penSec/cc_rust/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

AI-powered coding assistant written in Rust. Inspired by Anthropic's Claude Code.

## Features

- 🤖 **AI-Powered**: Integrates with Anthropic's Claude API
- 🛠️ **Tool System**: Extensible tool architecture with built-in support for:
  - Bash command execution
  - File operations (read, write, edit)
  - Code search (grep, glob)
  - Web fetching
- 💬 **Interactive Chat**: Terminal-based conversational interface
- 🚀 **Fast**: Native Rust performance
- 🔒 **Safe**: Sandboxed tool execution with permission controls

## Installation

### Quick Install (Linux/macOS)

```bash
curl -sSL https://raw.githubusercontent.com/0penSec/cc_rust/main/install.sh | bash
```

### Manual Installation

Download the latest release for your platform from the [releases page](https://github.com/0penSec/cc_rust/releases).

#### Linux/macOS
```bash
tar xzf claude-code-*-$(uname -m).tar.gz
chmod +x claude
sudo mv claude /usr/local/bin/
```

#### Windows
Extract the zip file and add the directory to your PATH.

### Build from Source

Requirements:
- Rust 1.80+
- OpenSSL development libraries

```bash
git clone https://github.com/0penSec/cc_rust.git
cd cc_rust
cargo build --release
sudo cp target/release/claude /usr/local/bin/
```

### Docker

```bash
docker pull ghcr.io/0penSec/cc_rust:latest
docker run -it --rm -e ANTHROPIC_API_KEY=$ANTHROPIC_API_KEY ghcr.io/0penSec/cc_rust:latest
```

## Quick Start

1. Set your Anthropic API key:
```bash
export ANTHROPIC_API_KEY="your-api-key"
```

2. Start interactive mode:
```bash
claude
```

3. Or run a single command:
```bash
claude run "Explain this codebase"
```

## Usage

### Interactive Mode

```
╔══════════════════════════════════════════╗
║       Claude Code - Interactive Mode     ║
╚══════════════════════════════════════════╝

> 你好！
Claude: 你好！很高兴帮助你。

> 查看当前目录的文件
Claude: [Tool: bash]
[Tool result]: ...
```

### Built-in Commands

- `help` - Show available commands
- `tools` - List available tools
- `context` - Show conversation context
- `clear` - Clear screen
- `exit` or `quit` - Exit the program

### Command Line Options

```bash
claude --help
claude run "your prompt here"
claude tools
```

## Architecture

```
crates/
├── core/       # Core types and traits
├── engine/     # API client and conversation management
├── tools/      # Tool implementations
└── cli/        # Command-line interface
```

## Development

### Setup

```bash
git clone https://github.com/0penSec/cc_rust.git
cd cc_rust
```

### Build

```bash
cargo build --release
```

### Test

```bash
cargo test --all
```

### Format and Lint

```bash
cargo fmt --all
cargo clippy --all-targets --all-features
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Inspired by [Anthropic's Claude Code](https://github.com/anthropics/claude-code)
- Built with [Rust](https://www.rust-lang.org/)
- Uses [Anthropic's Claude API](https://www.anthropic.com/)

## Disclaimer

This is an independent project and is not officially affiliated with Anthropic.
