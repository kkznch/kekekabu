mod api_anthropic;
mod api_gemini;
mod cli_claude;
mod cli_gemini;

use anyhow::{Result, bail};
use async_trait::async_trait;

use crate::config::ApiConfig;

pub use api_anthropic::ApiAnthropicBackend;
pub use api_gemini::ApiGeminiBackend;
pub use cli_claude::CliClaudeBackend;
pub use cli_gemini::CliGeminiBackend;

#[async_trait]
pub trait LlmBackend: Send + Sync {
    async fn send_message(&self, prompt: &str, max_tokens: u32) -> Result<String>;

    /// Send a message with a JSON schema for structured output.
    /// Backends that support structured output (e.g. Anthropic tool_use) override this.
    /// Default: falls back to send_message(), ignoring the schema.
    async fn send_message_with_schema(
        &self,
        prompt: &str,
        max_tokens: u32,
        _tool_name: &str,
        _tool_description: &str,
        _schema: serde_json::Value,
    ) -> Result<String> {
        self.send_message(prompt, max_tokens).await
    }
}

pub fn create_backend(
    backend_type: &str,
    api_config: &ApiConfig,
    model: Option<&str>,
) -> Result<Box<dyn LlmBackend>> {
    match backend_type {
        "api-anthropic" => {
            let key = require_key(
                &api_config.anthropic_api_key,
                "ANTHROPIC_API_KEY",
                "api-anthropic",
            )?;
            Ok(Box::new(ApiAnthropicBackend::new(
                key,
                model.map(|s| s.to_string()),
            )))
        }
        "api-gemini" => {
            let key = require_key(&api_config.gemini_api_key, "GEMINI_API_KEY", "api-gemini")?;
            Ok(Box::new(ApiGeminiBackend::new(
                key,
                model.map(|s| s.to_string()),
            )))
        }
        "cli-claude" => {
            CliClaudeBackend::check_available()?;
            Ok(Box::new(CliClaudeBackend::new(
                model.map(|s| s.to_string()),
            )))
        }
        "cli-gemini" => {
            CliGeminiBackend::check_available()?;
            Ok(Box::new(CliGeminiBackend::new(
                model.map(|s| s.to_string()),
            )))
        }
        _ => {
            bail!(
                "Unknown LLM backend: '{}'\nAvailable: api-anthropic, api-gemini, cli-claude, cli-gemini",
                backend_type
            );
        }
    }
}

fn require_key(value: &Option<String>, key_name: &str, backend: &str) -> Result<String> {
    value
        .as_ref()
        .filter(|k| !k.is_empty())
        .cloned()
        .ok_or_else(|| {
            anyhow::anyhow!(
                "{} backend requires {}.\n\
                 Set it in ~/.config/kabu/config.toml [api] or as an environment variable.",
                backend,
                key_name
            )
        })
}
