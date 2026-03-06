mod api_anthropic;
mod cli_claude;
mod cli_gemini;

use anyhow::{Result, bail};
use async_trait::async_trait;

pub use api_anthropic::ApiAnthropicBackend;
pub use cli_claude::CliClaudeBackend;
pub use cli_gemini::CliGeminiBackend;

#[async_trait]
pub trait LlmBackend: Send + Sync {
    async fn send_message(&self, prompt: &str, max_tokens: u32) -> Result<String>;
}

pub fn create_backend(
    backend_type: &str,
    api_key: Option<&str>,
    model: Option<&str>,
) -> Result<Box<dyn LlmBackend>> {
    let api_key = api_key.filter(|k| !k.is_empty());

    match backend_type {
        "api-anthropic" => {
            let key = api_key.ok_or_else(|| {
                anyhow::anyhow!(
                    "api-anthropic backend requires ANTHROPIC_API_KEY.\n\
                     Set it in ~/.config/kktd/config.toml [api] or as an environment variable."
                )
            })?;
            Ok(Box::new(ApiAnthropicBackend::new(
                key.to_string(),
                model.map(|s| s.to_string()),
            )))
        }
        "cli-claude" => {
            CliClaudeBackend::check_available()?;
            Ok(Box::new(CliClaudeBackend::new(model.map(|s| s.to_string()))))
        }
        "cli-gemini" => {
            CliGeminiBackend::check_available()?;
            Ok(Box::new(CliGeminiBackend::new(model.map(|s| s.to_string()))))
        }
        _ => {
            bail!(
                "Unknown LLM backend: '{}'\nAvailable: api-anthropic, cli-claude, cli-gemini",
                backend_type
            );
        }
    }
}
