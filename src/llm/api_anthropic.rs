use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::LlmBackend;

const DEFAULT_MODEL: &str = "claude-sonnet-4-5-20250514";

pub struct ApiAnthropicBackend {
    http: reqwest::Client,
    api_key: String,
    model: String,
}

#[derive(Serialize)]
struct MessageRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<ChatMessage>,
}

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct MessageResponse {
    content: Vec<ContentBlock>,
}

#[derive(Deserialize)]
struct ContentBlock {
    text: Option<String>,
}

impl ApiAnthropicBackend {
    pub fn new(api_key: String, model: Option<String>) -> Self {
        Self {
            http: reqwest::Client::new(),
            api_key,
            model: model.unwrap_or_else(|| DEFAULT_MODEL.to_string()),
        }
    }
}

#[async_trait]
impl LlmBackend for ApiAnthropicBackend {
    async fn send_message(&self, prompt: &str, max_tokens: u32) -> Result<String> {
        let req = MessageRequest {
            model: self.model.clone(),
            max_tokens,
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
        };

        let resp = self
            .http
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&req)
            .send()
            .await
            .context("Failed to call Anthropic API")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            bail!("Anthropic API error {}: {}", status, body);
        }

        let msg: MessageResponse = resp
            .json()
            .await
            .context("Failed to parse Anthropic response")?;

        let text = msg
            .content
            .iter()
            .filter_map(|b| b.text.as_deref())
            .collect::<Vec<_>>()
            .join("");

        if text.is_empty() {
            bail!("Anthropic API returned empty response");
        }

        Ok(text)
    }
}
