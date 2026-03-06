use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use tokio::process::Command;

use super::LlmBackend;

pub struct CliClaudeBackend {
    model: Option<String>,
}

impl CliClaudeBackend {
    pub fn new(model: Option<String>) -> Self {
        Self { model }
    }

    pub fn check_available() -> Result<()> {
        which::which("claude").map_err(|_| {
            anyhow::anyhow!(
                "claude command not found.\n\
                 Install Claude Code CLI: npm install -g @anthropic-ai/claude-code"
            )
        })?;
        Ok(())
    }
}

#[async_trait]
impl LlmBackend for CliClaudeBackend {
    async fn send_message(&self, prompt: &str, _max_tokens: u32) -> Result<String> {
        let mut cmd = Command::new("claude");
        cmd.arg("-p").arg(prompt);

        if let Some(ref model) = self.model {
            cmd.arg("--model").arg(model);
        }

        let output = cmd
            .output()
            .await
            .context("Failed to execute claude command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("claude CLI error: {}", stderr);
        }

        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();

        if text.is_empty() {
            bail!("claude CLI returned empty response");
        }

        Ok(text)
    }
}
