//! Configuration management for Claude Code CLI
//!
//! Supports loading configuration from:
//! - $HOME/.config/claude/config.toml (Linux/macOS)
//! - %APPDATA%\claude\config.toml (Windows)
//! - Environment variables (CLAUDE_* prefix)
//! - Command line arguments (highest priority)

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Main configuration structure
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct CliConfig {
    /// API configuration
    #[serde(default)]
    pub api: ApiConfig,

    /// Model settings
    #[serde(default)]
    pub model: ModelConfig,

    /// Tool settings
    #[serde(default)]
    pub tools: ToolsConfig,

    /// UI settings
    #[serde(default)]
    pub ui: UiConfig,

    /// Working directory (overrides default)
    pub working_dir: Option<PathBuf>,

    /// Environment variables to pass to tools
    #[serde(default)]
    pub env_vars: HashMap<String, String>,
}

impl CliConfig {
    /// Load configuration from default locations
    pub fn load() -> Result<Self> {
        let config_dir = Self::config_dir()?;
        let config_path = config_dir.join("config.toml");

        // Start with default config
        let mut config = Self::default();

        // Load from file if exists
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)
                .with_context(|| format!("Failed to read config from {:?}", config_path))?;

            let file_config: CliConfig = toml::from_str(&content)
                .with_context(|| format!("Failed to parse config from {:?}", config_path))?;

            config.merge(file_config);
        }

        // Override with environment variables
        config.merge_from_env();

        Ok(config)
    }

    /// Load configuration from a specific file path
    pub fn load_from_path(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config from {:?}", path))?;

        let config: CliConfig = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config from {:?}", path))?;

        Ok(config)
    }

    /// Get the configuration directory
    pub fn config_dir() -> Result<PathBuf> {
        let dirs = directories::ProjectDirs::from("com", "claude-code", "claude")
            .context("Failed to determine config directory")?;

        Ok(dirs.config_dir().to_path_buf())
    }

    /// Get the data directory for persistence
    pub fn data_dir() -> Result<PathBuf> {
        let dirs = directories::ProjectDirs::from("com", "claude-code", "claude")
            .context("Failed to determine data directory")?;

        Ok(dirs.data_dir().to_path_buf())
    }

    /// Create default configuration file if it doesn't exist
    pub fn create_default() -> Result<PathBuf> {
        let config_dir = Self::config_dir()?;
        std::fs::create_dir_all(&config_dir)
            .with_context(|| format!("Failed to create config directory {:?}", config_dir))?;

        let config_path = config_dir.join("config.toml");

        if !config_path.exists() {
            let default_config = include_str!("../config.default.toml");
            std::fs::write(&config_path, default_config)
                .with_context(|| format!("Failed to write default config to {:?}", config_path))?;
        }

        Ok(config_path)
    }

    /// Merge another config into this one
    fn merge(&mut self, other: CliConfig) {
        self.api.merge(other.api);
        self.model.merge(other.model);
        self.tools.merge(other.tools);
        self.ui.merge(other.ui);

        if other.working_dir.is_some() {
            self.working_dir = other.working_dir;
        }

        for (k, v) in other.env_vars {
            self.env_vars.insert(k, v);
        }
    }

    /// Override with environment variables
    fn merge_from_env(&mut self) {
        if let Ok(api_key) = std::env::var("ANTHROPIC_API_KEY") {
            self.api.api_key = Some(api_key);
        }

        if let Ok(model) = std::env::var("CLAUDE_MODEL") {
            self.model.name = model;
        }

        if let Ok(api_url) = std::env::var("CLAUDE_API_URL") {
            self.api.base_url = Some(api_url);
        }

        if let Ok(working_dir) = std::env::var("CLAUDE_WORKING_DIR") {
            self.working_dir = Some(PathBuf::from(working_dir));
        }

        if let Ok(timeout) = std::env::var("CLAUDE_TIMEOUT") {
            if let Ok(t) = timeout.parse::<u64>() {
                self.api.timeout_seconds = t;
            }
        }
    }

    /// Get the API key, checking config and environment
    pub fn get_api_key(&self) -> Option<String> {
        // First check environment (highest priority after CLI args)
        if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
            return Some(key);
        }

        // Then check config file
        self.api.api_key.clone()
    }
}

/// API configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApiConfig {
    /// API key (prefer env var ANTHROPIC_API_KEY)
    pub api_key: Option<String>,

    /// Base URL for API (for custom endpoints)
    pub base_url: Option<String>,

    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,

    /// Maximum retries for failed requests
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// Retry delay in milliseconds
    #[serde(default = "default_retry_delay_ms")]
    pub retry_delay_ms: u64,
}

impl ApiConfig {
    fn merge(&mut self, other: ApiConfig) {
        if other.api_key.is_some() {
            self.api_key = other.api_key;
        }
        if other.base_url.is_some() {
            self.base_url = other.base_url;
        }
        self.timeout_seconds = other.timeout_seconds;
        self.max_retries = other.max_retries;
        self.retry_delay_ms = other.retry_delay_ms;
    }
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            base_url: None,
            timeout_seconds: default_timeout(),
            max_retries: default_max_retries(),
            retry_delay_ms: default_retry_delay_ms(),
        }
    }
}

/// Model configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModelConfig {
    /// Model name
    #[serde(default = "default_model")]
    pub name: String,

    /// Maximum tokens per request
    #[serde(default = "default_max_tokens")]
    pub max_tokens: usize,

    /// Temperature (0.0 - 1.0)
    #[serde(default = "default_temperature")]
    pub temperature: f32,

    /// Top-p sampling
    #[serde(default = "default_top_p")]
    pub top_p: f32,
}

impl ModelConfig {
    fn merge(&mut self, other: ModelConfig) {
        self.name = other.name;
        self.max_tokens = other.max_tokens;
        self.temperature = other.temperature;
        self.top_p = other.top_p;
    }
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            name: default_model(),
            max_tokens: default_max_tokens(),
            temperature: default_temperature(),
            top_p: default_top_p(),
        }
    }
}

/// Tool settings
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToolsConfig {
    /// Bash tool settings
    #[serde(default)]
    pub bash: BashConfig,

    /// File tool settings
    #[serde(default)]
    pub file: FileConfig,

    /// Search tool settings
    #[serde(default)]
    pub search: SearchConfig,

    /// Web tool settings
    #[serde(default)]
    pub web: WebConfig,

    /// Default permission mode
    #[serde(default = "default_permission_mode")]
    pub permission_mode: String,
}

impl ToolsConfig {
    fn merge(&mut self, other: ToolsConfig) {
        self.bash.merge(other.bash);
        self.file.merge(other.file);
        self.search.merge(other.search);
        self.web.merge(other.web);
        self.permission_mode = other.permission_mode;
    }
}

impl Default for ToolsConfig {
    fn default() -> Self {
        Self {
            bash: BashConfig::default(),
            file: FileConfig::default(),
            search: SearchConfig::default(),
            web: WebConfig::default(),
            permission_mode: default_permission_mode(),
        }
    }
}

/// Bash tool configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BashConfig {
    /// Default timeout in seconds
    #[serde(default = "default_bash_timeout")]
    pub timeout_seconds: u64,

    /// Allowed commands (empty = allow all)
    #[serde(default)]
    pub allowed_commands: Vec<String>,

    /// Blocked commands
    #[serde(default = "default_blocked_commands")]
    pub blocked_commands: Vec<String>,
}

impl BashConfig {
    fn merge(&mut self, other: BashConfig) {
        self.timeout_seconds = other.timeout_seconds;
        if !other.allowed_commands.is_empty() {
            self.allowed_commands = other.allowed_commands;
        }
        if !other.blocked_commands.is_empty() {
            self.blocked_commands = other.blocked_commands;
        }
    }
}

impl Default for BashConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: default_bash_timeout(),
            allowed_commands: Vec::new(),
            blocked_commands: default_blocked_commands(),
        }
    }
}

/// File tool configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FileConfig {
    /// Maximum file size to read (in bytes)
    #[serde(default = "default_max_file_size")]
    pub max_read_size: usize,

    /// Maximum file size to write (in bytes)
    #[serde(default = "default_max_file_size")]
    pub max_write_size: usize,

    /// Allowed file extensions (empty = allow all)
    #[serde(default)]
    pub allowed_extensions: Vec<String>,

    /// Blocked paths (glob patterns)
    #[serde(default = "default_blocked_paths")]
    pub blocked_paths: Vec<String>,
}

impl FileConfig {
    fn merge(&mut self, other: FileConfig) {
        self.max_read_size = other.max_read_size;
        self.max_write_size = other.max_write_size;
        if !other.allowed_extensions.is_empty() {
            self.allowed_extensions = other.allowed_extensions;
        }
        if !other.blocked_paths.is_empty() {
            self.blocked_paths = other.blocked_paths;
        }
    }
}

impl Default for FileConfig {
    fn default() -> Self {
        Self {
            max_read_size: default_max_file_size(),
            max_write_size: default_max_file_size(),
            allowed_extensions: Vec::new(),
            blocked_paths: default_blocked_paths(),
        }
    }
}

/// Search tool configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SearchConfig {
    /// Maximum results to return
    #[serde(default = "default_max_search_results")]
    pub max_results: usize,

    /// Respect .gitignore
    #[serde(default = "default_true")]
    pub respect_gitignore: bool,

    /// Case sensitive by default
    #[serde(default)]
    pub case_sensitive: bool,
}

impl SearchConfig {
    fn merge(&mut self, other: SearchConfig) {
        self.max_results = other.max_results;
        self.respect_gitignore = other.respect_gitignore;
        self.case_sensitive = other.case_sensitive;
    }
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            max_results: default_max_search_results(),
            respect_gitignore: default_true(),
            case_sensitive: false,
        }
    }
}

/// Web tool configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WebConfig {
    /// Maximum content length to fetch (in bytes)
    #[serde(default = "default_max_web_content")]
    pub max_content_length: usize,

    /// Request timeout in seconds
    #[serde(default = "default_web_timeout")]
    pub timeout_seconds: u64,

    /// Follow redirects
    #[serde(default = "default_true")]
    pub follow_redirects: bool,

    /// Allowed domains (empty = allow all)
    #[serde(default)]
    pub allowed_domains: Vec<String>,
}

impl WebConfig {
    fn merge(&mut self, other: WebConfig) {
        self.max_content_length = other.max_content_length;
        self.timeout_seconds = other.timeout_seconds;
        self.follow_redirects = other.follow_redirects;
        if !other.allowed_domains.is_empty() {
            self.allowed_domains = other.allowed_domains;
        }
    }
}

impl Default for WebConfig {
    fn default() -> Self {
        Self {
            max_content_length: default_max_web_content(),
            timeout_seconds: default_web_timeout(),
            follow_redirects: default_true(),
            allowed_domains: Vec::new(),
        }
    }
}

/// UI configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UiConfig {
    /// Theme (light/dark/auto)
    #[serde(default = "default_theme")]
    pub theme: String,

    /// Enable animations
    #[serde(default = "default_true")]
    pub animations: bool,

    /// Show token usage
    #[serde(default = "default_true")]
    pub show_token_usage: bool,

    /// Show cost estimation
    #[serde(default = "default_true")]
    pub show_cost: bool,

    /// Enable streaming output
    #[serde(default = "default_true")]
    pub streaming: bool,

    /// Verbose mode
    #[serde(default)]
    pub verbose: bool,
}

impl UiConfig {
    fn merge(&mut self, other: UiConfig) {
        self.theme = other.theme;
        self.animations = other.animations;
        self.show_token_usage = other.show_token_usage;
        self.show_cost = other.show_cost;
        self.streaming = other.streaming;
        self.verbose = other.verbose;
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            animations: default_true(),
            show_token_usage: default_true(),
            show_cost: default_true(),
            streaming: default_true(),
            verbose: false,
        }
    }
}

// Default value functions
fn default_timeout() -> u64 {
    120
}
fn default_max_retries() -> u32 {
    3
}
fn default_retry_delay_ms() -> u64 {
    1000
}
fn default_model() -> String {
    "claude-sonnet-4-6".to_string()
}
fn default_max_tokens() -> usize {
    4096
}
fn default_temperature() -> f32 {
    0.7
}
fn default_top_p() -> f32 {
    0.9
}
fn default_permission_mode() -> String {
    "ask".to_string()
}
fn default_bash_timeout() -> u64 {
    300
}
fn default_max_file_size() -> usize {
    1024 * 1024
} // 1MB
fn default_max_search_results() -> usize {
    100
}
fn default_max_web_content() -> usize {
    100 * 1024
} // 100KB
fn default_web_timeout() -> u64 {
    30
}
fn default_theme() -> String {
    "auto".to_string()
}
fn default_true() -> bool {
    true
}

fn default_blocked_commands() -> Vec<String> {
    vec![
        "rm -rf /".to_string(),
        "dd if=/dev/zero".to_string(),
        ":(){ :|:& };:".to_string(), // Fork bomb
    ]
}

fn default_blocked_paths() -> Vec<String> {
    vec![
        "/etc/passwd".to_string(),
        "/etc/shadow".to_string(),
        "~/.ssh/id_*".to_string(),
        "**/.env".to_string(),
    ]
}
