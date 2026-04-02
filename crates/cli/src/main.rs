mod config;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::io::Write;
use std::path::{Path, PathBuf};
use tracing::{debug, info};

use claude_core::{SessionId, ToolContext};
use claude_engine::{Conversation, EngineConfig, ToolLoop};
use claude_tools::default_registry;

use crate::config::CliConfig;

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

    /// API key (or set ANTHROPIC_API_KEY env var or config file)
    #[arg(long, env = "ANTHROPIC_API_KEY")]
    api_key: Option<String>,

    /// Model to use
    #[arg(short, long)]
    model: Option<String>,

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
    /// Initialize configuration file
    Init {
        /// Force overwrite existing config
        #[arg(short, long)]
        force: bool,
    },
    /// Version information
    Version,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Load configuration (from file or defaults)
    let mut config = if let Some(config_path) = &cli.config {
        CliConfig::load_from_path(config_path)?
    } else {
        CliConfig::load().unwrap_or_default()
    };

    // Apply CLI overrides (CLI args have highest priority)
    if let Some(api_key) = cli.api_key {
        config.api.api_key = Some(api_key);
    }
    if let Some(model) = cli.model {
        config.model.name = model;
    }
    if let Some(working_dir) = cli.working_dir {
        config.working_dir = Some(working_dir);
    }
    if cli.verbose {
        config.ui.verbose = true;
    }

    // Initialize logging
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(if config.ui.verbose { "debug" } else { "info" })
        .with_target(false)
        .with_thread_ids(false)
        .compact()
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    debug!("Starting Claude Code");

    // Determine working directory
    let working_dir = config
        .working_dir
        .clone()
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
            show_config(&config, &working_dir)?;
            return Ok(());
        }
        Some(Commands::Init { force }) => {
            init_config(force)?;
            return Ok(());
        }
        Some(Commands::Version) => {
            println!("Claude Code {}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
        Some(Commands::Run { prompt }) => {
            let api_key = config.get_api_key()
                .context("API key not found. Set ANTHROPIC_API_KEY environment variable or add to config file")?;
            run_single_prompt(&api_key, &config.model.name, &working_dir, &prompt, &config).await?;
        }
        Some(Commands::Chat { message }) => {
            let api_key = config.get_api_key()
                .context("API key not found. Set ANTHROPIC_API_KEY environment variable or add to config file")?;
            run_interactive(&api_key, &config.model.name, &working_dir, message, &config).await?;
        }
        None => {
            // Default: interactive mode
            let api_key = config.get_api_key()
                .context("API key not found. Set ANTHROPIC_API_KEY environment variable or add to config file")?;
            run_interactive(&api_key, &config.model.name, &working_dir, None, &config).await?;
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

fn show_config(config: &CliConfig, working_dir: &Path) -> Result<()> {
    println!("Configuration:");
    println!("  Config Directory: {:?}", CliConfig::config_dir()?);
    println!("  Data Directory: {:?}", CliConfig::data_dir()?);
    println!("  Working Directory: {:?}", working_dir);
    println!();
    println!("  API:");
    println!(
        "    API Key: {}",
        config
            .api
            .api_key
            .as_ref()
            .map(|k| format!("{}...", &k[..k.len().min(10)]))
            .unwrap_or_else(|| "not set".to_string())
    );
    println!(
        "    Base URL: {}",
        config.api.base_url.as_deref().unwrap_or("default")
    );
    println!("    Timeout: {}s", config.api.timeout_seconds);
    println!("    Max Retries: {}", config.api.max_retries);
    println!();
    println!("  Model:");
    println!("    Name: {}", config.model.name);
    println!("    Max Tokens: {}", config.model.max_tokens);
    println!("    Temperature: {}", config.model.temperature);
    println!("    Top P: {}", config.model.top_p);
    println!();
    println!("  Tools:");
    println!("    Permission Mode: {}", config.tools.permission_mode);
    println!("    Bash Timeout: {}s", config.tools.bash.timeout_seconds);
    println!(
        "    File Max Read: {} bytes",
        config.tools.file.max_read_size
    );
    println!(
        "    Search Max Results: {}",
        config.tools.search.max_results
    );
    println!();
    println!("  UI:");
    println!("    Theme: {}", config.ui.theme);
    println!("    Animations: {}", config.ui.animations);
    println!("    Show Token Usage: {}", config.ui.show_token_usage);
    println!("    Show Cost: {}", config.ui.show_cost);
    println!("    Streaming: {}", config.ui.streaming);
    println!("    Verbose: {}", config.ui.verbose);
    Ok(())
}

fn init_config(force: bool) -> Result<()> {
    let config_path = CliConfig::create_default()?;

    if config_path.exists() && !force {
        println!("Configuration file already exists at: {:?}", config_path);
        println!("Use --force to overwrite.");
        return Ok(());
    }

    println!("Created default configuration file at: {:?}", config_path);
    println!("\nEdit this file to customize your Claude Code settings.");
    Ok(())
}

async fn run_single_prompt(
    api_key: &str,
    model: &str,
    working_dir: &Path,
    prompt: &str,
    cli_config: &CliConfig,
) -> Result<()> {
    println!("Running: {}\n", prompt);

    // Initialize engine
    let mut engine_config = EngineConfig::default();
    engine_config.client.api_key = api_key.to_string();

    let client = claude_engine::AnthropicClient::new(engine_config.client)?;

    // Create conversation
    let mut conversation = Conversation::builder()
        .system_prompt(get_system_prompt())
        .model(model.to_string())
        .build();

    // Add user message
    conversation.add_user_message(prompt);

    // Initialize tool registry
    let registry = default_registry();

    // Set up tool loop with config-aware tools
    let mut tool_loop = ToolLoop::new(client);
    for name in registry.list() {
        if let Some(_tool) = registry.get(name) {
            match name {
                "bash" => tool_loop.register_tool(Box::new(
                    claude_tools::BashTool::new()
                        .with_timeout(cli_config.tools.bash.timeout_seconds),
                )),
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
    let mut tool_ctx = ToolContext {
        session_id: SessionId::new(),
        working_directory: working_dir.to_path_buf(),
        env_vars: std::env::vars().collect(),
    };
    // Add config env vars
    for (k, v) in &cli_config.env_vars {
        tool_ctx.env_vars.insert(k.clone(), v.clone());
    }

    // Run the conversation
    match tool_loop.run(&mut conversation, &tool_ctx).await {
        Ok(usage) => {
            println!("\n[Conversation complete]");
            if cli_config.ui.show_token_usage {
                println!(
                    "Tokens used: input={}, output={}",
                    usage.input_tokens, usage.output_tokens
                );
            }
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
    cli_config: &CliConfig,
) -> Result<()> {
    println!("╔══════════════════════════════════════════╗");
    println!("║       Claude Code - Interactive Mode     ║");
    println!("╚══════════════════════════════════════════╝");
    println!("Type 'exit' or press Ctrl+C to quit\n");

    // Initialize engine
    let mut engine_config = EngineConfig::default();
    engine_config.client.api_key = api_key.to_string();

    let client = claude_engine::AnthropicClient::new(engine_config.client)?;

    // Create conversation with system prompt
    let mut conversation = Conversation::builder()
        .system_prompt(get_system_prompt())
        .model(model.to_string())
        .build();

    // Initialize tool registry and loop
    let _registry = default_registry();
    let mut tool_loop = ToolLoop::new(client);

    // Register tools with config
    tool_loop.register_tool(Box::new(
        claude_tools::BashTool::new().with_timeout(cli_config.tools.bash.timeout_seconds),
    ));
    tool_loop.register_tool(Box::new(claude_tools::FileReadTool));
    tool_loop.register_tool(Box::new(claude_tools::FileWriteTool));
    tool_loop.register_tool(Box::new(claude_tools::FileEditTool));
    tool_loop.register_tool(Box::new(claude_tools::GlobTool));
    tool_loop.register_tool(Box::new(claude_tools::GrepTool));
    tool_loop.register_tool(Box::new(claude_tools::WebFetchTool));

    // Tool context with config env vars
    let mut tool_ctx = ToolContext {
        session_id: SessionId::new(),
        working_directory: working_dir.to_path_buf(),
        env_vars: std::env::vars().collect(),
    };
    // Add config env vars
    for (k, v) in &cli_config.env_vars {
        tool_ctx.env_vars.insert(k.clone(), v.clone());
    }

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
