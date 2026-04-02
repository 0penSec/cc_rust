use anyhow::Result;
use clap::{Parser, Subcommand};
use std::io::Write;
use std::path::{Path, PathBuf};
use tracing::{debug, info};

use claude_core::{SessionId, ToolContext};
use claude_engine::{Conversation, EngineConfig, ToolLoop};
use claude_tools::default_registry;

/// Claude Code - AI-powered coding assistant
#[derive(Parser)]
#[command(name = "claude")]
#[command(about = "AI-powered coding assistant")]
#[command(version)]
struct Cli {
    /// Configuration file path
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Working directory
    #[arg(short, long, value_name = "DIR")]
    working_dir: Option<PathBuf>,

    /// API key (or set ANTHROPIC_API_KEY env var)
    #[arg(long, env = "ANTHROPIC_API_KEY")]
    api_key: String,

    /// Model to use
    #[arg(short, long, default_value = "claude-sonnet-4-6")]
    model: String,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Subcommand to run
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start interactive session
    Chat {
        /// Initial message
        #[arg(short, long)]
        message: Option<String>,
    },
    /// Run a single command
    Run {
        /// The command/prompt to execute
        prompt: String,
    },
    /// List available tools
    Tools,
    /// Show configuration
    Config,
    /// Version information
    Version,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(if cli.verbose { "debug" } else { "info" })
        .with_target(false)
        .with_thread_ids(false)
        .compact()
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    debug!("Starting Claude Code");

    // Determine working directory
    let working_dir = cli
        .working_dir
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."));

    info!("Working directory: {:?}", working_dir);

    // Handle subcommands
    match cli.command {
        Some(Commands::Tools) => {
            list_tools().await;
            return Ok(());
        }
        Some(Commands::Config) => {
            show_config(&cli.api_key, &cli.model, &working_dir)?;
            return Ok(());
        }
        Some(Commands::Version) => {
            println!("Claude Code {}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
        Some(Commands::Run { prompt }) => {
            run_single_prompt(&cli.api_key, &cli.model, &working_dir, &prompt).await?;
        }
        Some(Commands::Chat { message }) => {
            run_interactive(&cli.api_key, &cli.model, &working_dir, message).await?;
        }
        None => {
            // Default: interactive mode
            run_interactive(&cli.api_key, &cli.model, &working_dir, None).await?;
        }
    }

    Ok(())
}

async fn list_tools() {
    let registry = default_registry();
    println!("Available tools:\n");

    for name in registry.list() {
        if let Some(tool) = registry.get(name) {
            println!("  {} - {}", name, tool.description());
        }
    }
}

fn show_config(api_key: &str, model: &str, working_dir: &Path) -> Result<()> {
    println!("Configuration:");
    println!("  API Key: {}...", &api_key[..api_key.len().min(10)]);
    println!("  Model: {}", model);
    println!("  Working Directory: {:?}", working_dir);
    Ok(())
}

async fn run_single_prompt(
    api_key: &str,
    model: &str,
    working_dir: &Path,
    prompt: &str,
) -> Result<()> {
    println!("Running: {}\n", prompt);

    // Initialize engine
    let mut config = EngineConfig::default();
    config.client.api_key = api_key.to_string();

    let client = claude_engine::AnthropicClient::new(config.client)?;

    // Create conversation
    let mut conversation = Conversation::builder()
        .system_prompt(get_system_prompt())
        .model(model.to_string())
        .build();

    // Add user message
    conversation.add_user_message(prompt);

    // Initialize tool registry
    let registry = default_registry();

    // Set up tool loop
    let mut tool_loop = ToolLoop::new(client);
    for name in registry.list() {
        if let Some(_tool) = registry.get(name) {
            // We need to clone the tool, but Tool trait doesn't support cloning directly
            // For now, we'll just register by creating new instances
            match name {
                "bash" => tool_loop.register_tool(Box::new(claude_tools::BashTool::new())),
                "file_read" => tool_loop.register_tool(Box::new(claude_tools::FileReadTool)),
                "file_write" => tool_loop.register_tool(Box::new(claude_tools::FileWriteTool)),
                "file_edit" => tool_loop.register_tool(Box::new(claude_tools::FileEditTool)),
                "glob" => tool_loop.register_tool(Box::new(claude_tools::GlobTool)),
                "grep" => tool_loop.register_tool(Box::new(claude_tools::GrepTool)),
                "web_fetch" => tool_loop.register_tool(Box::new(claude_tools::WebFetchTool)),
                _ => {}
            }
        }
    }

    // Create tool context
    let tool_ctx = ToolContext {
        session_id: SessionId::new(),
        working_directory: working_dir.to_path_buf(),
        env_vars: std::env::vars().collect(),
    };

    // Run the conversation
    match tool_loop.run(&mut conversation, &tool_ctx).await {
        Ok(usage) => {
            println!("\n[Conversation complete]");
            println!(
                "Tokens used: input={}, output={}",
                usage.input_tokens, usage.output_tokens
            );
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }

    Ok(())
}

async fn run_interactive(
    api_key: &str,
    model: &str,
    working_dir: &Path,
    initial_message: Option<String>,
) -> Result<()> {
    println!("╔══════════════════════════════════════════╗");
    println!("║       Claude Code - Interactive Mode     ║");
    println!("╚══════════════════════════════════════════╝");
    println!("Type 'exit' or press Ctrl+C to quit\n");

    // Initialize engine
    let mut config = EngineConfig::default();
    config.client.api_key = api_key.to_string();

    let client = claude_engine::AnthropicClient::new(config.client)?;

    // Create conversation with system prompt
    let mut conversation = Conversation::builder()
        .system_prompt(get_system_prompt())
        .model(model.to_string())
        .build();

    // Initialize tool registry and loop
    let _registry = default_registry();
    let mut tool_loop = ToolLoop::new(client);

    // Register tools
    tool_loop.register_tool(Box::new(claude_tools::BashTool::new()));
    tool_loop.register_tool(Box::new(claude_tools::FileReadTool));
    tool_loop.register_tool(Box::new(claude_tools::FileWriteTool));
    tool_loop.register_tool(Box::new(claude_tools::FileEditTool));
    tool_loop.register_tool(Box::new(claude_tools::GlobTool));
    tool_loop.register_tool(Box::new(claude_tools::GrepTool));
    tool_loop.register_tool(Box::new(claude_tools::WebFetchTool));

    // Tool context
    let tool_ctx = ToolContext {
        session_id: SessionId::new(),
        working_directory: working_dir.to_path_buf(),
        env_vars: std::env::vars().collect(),
    };

    // Handle initial message if provided
    if let Some(msg) = initial_message {
        println!("You: {}", msg);
        conversation.add_user_message(msg);

        print!("Claude: ");
        if let Err(e) = tool_loop.run(&mut conversation, &tool_ctx).await {
            eprintln!("Error: {}", e);
        }
        println!();
    }

    // Interactive loop
    loop {
        print!("\n> ");
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        // Handle built-in commands
        match input {
            "exit" | "quit" | ":q" => {
                println!("Goodbye!");
                break;
            }
            "help" | ":h" => {
                print_help();
                continue;
            }
            "tools" | ":t" => {
                list_tools().await;
                continue;
            }
            "clear" | ":c" => {
                // Clear screen
                print!("\x1B[2J\x1B[1;1H");
                continue;
            }
            "context" | ":ctx" => {
                println!("Messages in context: {}", conversation.messages.len());
                continue;
            }
            _ => {}
        }

        // Add user message to conversation
        conversation.add_user_message(input.to_string());

        // Run the conversation loop
        print!("Claude: ");
        std::io::stdout().flush()?;

        if let Err(e) = tool_loop.run(&mut conversation, &tool_ctx).await {
            eprintln!("\nError: {}", e);
        }

        println!();
    }

    Ok(())
}

fn print_help() {
    println!("Commands:");
    println!("  help     - Show this help");
    println!("  exit     - Exit the program");
    println!("  tools    - List available tools");
    println!("  clear    - Clear screen");
    println!("  context  - Show conversation context");
}

fn get_system_prompt() -> String {
    r#"You are Claude Code, an AI coding assistant. You help users with software engineering tasks.

When working with code:
1. Use the provided tools to explore, read, and modify files
2. Run bash commands when needed (with user confirmation for destructive operations)
3. Provide clear explanations of what you're doing
4. Ask for clarification when requirements are unclear

You have access to these tools:
- bash: Execute shell commands
- file_read: Read file contents
- file_write: Write files
- file_edit: Edit files
- glob: Find files by pattern
- grep: Search file contents
- web_fetch: Fetch web pages

Always confirm before making destructive changes."#
        .to_string()
}
