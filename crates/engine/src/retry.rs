use std::time::Duration;
use tokio::time::sleep;
use tracing::warn;

use claude_core::ClaudeError;

/// Retry configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub exponential_base: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(60),
            exponential_base: 2.0,
        }
    }
}

/// Retry a future with exponential backoff
pub async fn retry_with_backoff<T, F, Fut>(
    config: &RetryConfig,
    operation: F,
) -> Result<T, ClaudeError>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, ClaudeError>>,
{
    let mut last_error = None;

    for attempt in 0..=config.max_retries {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(err) => {
                let should_retry = should_retry_error(&err);
                last_error = Some(err);

                if !should_retry || attempt == config.max_retries {
                    break;
                }

                let delay = calculate_delay(config, attempt);
                warn!(
                    "Request failed (attempt {}/{}), retrying in {:?}...",
                    attempt + 1,
                    config.max_retries + 1,
                    delay
                );
                sleep(delay).await;
            }
        }
    }

    Err(last_error.unwrap_or_else(|| ClaudeError::Internal("Unknown error".to_string())))
}

fn should_retry_error(error: &ClaudeError) -> bool {
    match error {
        ClaudeError::Api { status, .. } => {
            // Retry on rate limits (429) and server errors (5xx)
            *status == 429 || (*status >= 500 && *status < 600)
        }
        ClaudeError::Network(_) => true,
        ClaudeError::Timeout => true,
        _ => false,
    }
}

fn calculate_delay(config: &RetryConfig, attempt: u32) -> Duration {
    let exponential =
        config.base_delay.as_millis() as f64 * config.exponential_base.powi(attempt as i32);
    let delay_ms = exponential.min(config.max_delay.as_millis() as f64) as u64;
    // Add jitter (±25%)
    let jitter = (delay_ms as f64 * 0.25) as u64;
    let jittered = delay_ms + rand::random::<u64>() % (jitter * 2);
    Duration::from_millis(jittered)
}
