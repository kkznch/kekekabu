use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use tokio::process::Command;

use super::LlmBackend;

pub struct CliGeminiBackend {
    model: Option<String>,
}

impl CliGeminiBackend {
    pub fn new(model: Option<String>) -> Self {
        Self { model }
    }

    pub fn check_available() -> Result<()> {
        which::which("gemini").map_err(|_| {
            anyhow::anyhow!(
                "gemini command not found.\n\
                 Install Gemini CLI: npm install -g @anthropic-ai/gemini-cli"
            )
        })?;
        Ok(())
    }
}

#[async_trait]
impl LlmBackend for CliGeminiBackend {
    async fn send_message(&self, prompt: &str, _max_tokens: u32) -> Result<String> {
        let mut cmd = Command::new("gemini");
        cmd.arg("-p").arg(prompt);

        if let Some(ref model) = self.model {
            cmd.arg("-m").arg(model);
        }

        let output = cmd
            .output()
            .await
            .context("Failed to execute gemini command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("gemini CLI error: {}", stderr);
        }

        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();

        if text.is_empty() {
            bail!("gemini CLI returned empty response");
        }

        Ok(text)
    }
}
