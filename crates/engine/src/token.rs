//! Token counting and estimation
//!
//! Provides utilities for counting tokens in messages and estimating costs.

/// Tokenizer for counting tokens
pub struct Tokenizer;

impl Tokenizer {
    /// Create a new tokenizer
    pub fn new() -> Self {
        Self
    }

    /// Count tokens in text (simplified estimation)
    pub fn count(&self, text: &str) -> usize {
        // Simple estimation: ~4 characters per token on average
        text.len() / 4
    }

    /// Count tokens with a specific model
    pub fn count_for_model(&self, text: &str, model: &str) -> usize {
        let _ = model; // Currently ignored
        self.count(text)
    }
}

impl Default for Tokenizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Cost calculator
pub struct CostCalculator {
    input_price_per_token: f64,
    output_price_per_token: f64,
}

impl CostCalculator {
    /// Create a new cost calculator for a specific model
    pub fn for_model(model: &str) -> Self {
        // Default pricing (Claude Sonnet)
        let (input_price, output_price) = match model {
            "claude-opus" => (15.0 / 1_000_000.0, 75.0 / 1_000_000.0),
            "claude-sonnet" => (3.0 / 1_000_000.0, 15.0 / 1_000_000.0),
            "claude-haiku" => (0.25 / 1_000_000.0, 1.25 / 1_000_000.0),
            _ => (3.0 / 1_000_000.0, 15.0 / 1_000_000.0),
        };

        Self {
            input_price_per_token: input_price,
            output_price_per_token: output_price,
        }
    }

    /// Calculate cost for token usage
    pub fn calculate(&self, input_tokens: usize, output_tokens: usize) -> f64 {
        (input_tokens as f64 * self.input_price_per_token)
            + (output_tokens as f64 * self.output_price_per_token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_count() {
        let tokenizer = Tokenizer::new();
        let text = "Hello, world! This is a test.";
        let count = tokenizer.count(text);
        assert!(count > 0);
    }

    #[test]
    fn test_cost_calculation() {
        let calc = CostCalculator::for_model("claude-sonnet");
        let cost = calc.calculate(1000, 500);
        assert!(cost > 0.0);
    }
}
