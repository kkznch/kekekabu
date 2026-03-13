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
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<Tool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<ToolChoice>,
}

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct Tool {
    name: String,
    description: String,
    input_schema: serde_json::Value,
}

#[derive(Serialize)]
struct ToolChoice {
    #[serde(rename = "type")]
    choice_type: String,
    name: String,
}

#[derive(Deserialize)]
struct MessageResponse {
    content: Vec<ContentBlock>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse {
        #[allow(dead_code)]
        id: String,
        #[allow(dead_code)]
        name: String,
        input: serde_json::Value,
    },
}

impl ApiAnthropicBackend {
    pub fn new(api_key: String, model: Option<String>) -> Self {
        Self {
            http: reqwest::Client::new(),
            api_key,
            model: model.unwrap_or_else(|| DEFAULT_MODEL.to_string()),
        }
    }

    async fn call_api(&self, req: &MessageRequest) -> Result<MessageResponse> {
        let resp = self
            .http
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(req)
            .send()
            .await
            .context("Failed to call Anthropic API")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            bail!("Anthropic API error {}: {}", status, body);
        }

        resp.json()
            .await
            .context("Failed to parse Anthropic response")
    }
}

#[async_trait]
impl LlmBackend for ApiAnthropicBackend {
    async fn send_message(
        &self,
        prompt: &str,
        max_tokens: u32,
        temperature: Option<f32>,
    ) -> Result<String> {
        let req = MessageRequest {
            model: self.model.clone(),
            max_tokens,
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            temperature,
            tools: None,
            tool_choice: None,
        };

        let msg = self.call_api(&req).await?;

        let text = msg
            .content
            .iter()
            .filter_map(|b| match b {
                ContentBlock::Text { text } => Some(text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("");

        if text.is_empty() {
            bail!("Anthropic API returned empty response");
        }

        Ok(text)
    }

    async fn send_message_with_schema(
        &self,
        prompt: &str,
        max_tokens: u32,
        tool_name: &str,
        tool_description: &str,
        schema: serde_json::Value,
        temperature: Option<f32>,
    ) -> Result<String> {
        let req = MessageRequest {
            model: self.model.clone(),
            max_tokens,
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            temperature,
            tools: Some(vec![Tool {
                name: tool_name.to_string(),
                description: tool_description.to_string(),
                input_schema: schema,
            }]),
            tool_choice: Some(ToolChoice {
                choice_type: "tool".to_string(),
                name: tool_name.to_string(),
            }),
        };

        let msg = self.call_api(&req).await?;

        // Extract the tool_use input as JSON string
        for block in &msg.content {
            if let ContentBlock::ToolUse { input, .. } = block {
                return serde_json::to_string(input).context("Failed to serialize tool_use input");
            }
        }

        bail!("Anthropic API response did not contain a tool_use block")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_block_text_deserialization() {
        let json = r#"{"type": "text", "text": "Hello world"}"#;
        let block: ContentBlock = serde_json::from_str(json).unwrap();
        match block {
            ContentBlock::Text { text } => assert_eq!(text, "Hello world"),
            _ => panic!("Expected Text block"),
        }
    }

    #[test]
    fn test_content_block_tool_use_deserialization() {
        let json = r#"{
            "type": "tool_use",
            "id": "toolu_123",
            "name": "eval_stock",
            "input": {"ticker": "7203", "decision": "Buy", "score": 80}
        }"#;
        let block: ContentBlock = serde_json::from_str(json).unwrap();
        match block {
            ContentBlock::ToolUse { id, name, input } => {
                assert_eq!(id, "toolu_123");
                assert_eq!(name, "eval_stock");
                assert_eq!(input["ticker"], "7203");
                assert_eq!(input["decision"], "Buy");
                assert_eq!(input["score"], 80);
            }
            _ => panic!("Expected ToolUse block"),
        }
    }

    #[test]
    fn test_message_request_temperature_included() {
        let req = MessageRequest {
            model: "claude-sonnet-4-5-20250514".to_string(),
            max_tokens: 1024,
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: "hello".to_string(),
            }],
            temperature: Some(0.0),
            tools: None,
            tool_choice: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"temperature\":0.0"));
    }

    #[test]
    fn test_message_request_temperature_omitted() {
        let req = MessageRequest {
            model: "claude-sonnet-4-5-20250514".to_string(),
            max_tokens: 1024,
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: "hello".to_string(),
            }],
            temperature: None,
            tools: None,
            tool_choice: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(!json.contains("temperature"));
    }

    #[test]
    fn test_message_response_with_tool_use() {
        let json = r#"{
            "content": [
                {
                    "type": "tool_use",
                    "id": "toolu_abc",
                    "name": "discover",
                    "input": {
                        "keep": [{"ticker": "7203", "name": "Toyota", "reason": "solid"}],
                        "add": [],
                        "remove": []
                    }
                }
            ]
        }"#;
        let resp: MessageResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.content.len(), 1);
        match &resp.content[0] {
            ContentBlock::ToolUse { input, .. } => {
                assert!(input["keep"].is_array());
            }
            _ => panic!("Expected ToolUse"),
        }
    }
}
