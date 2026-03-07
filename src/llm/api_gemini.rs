use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::LlmBackend;

const DEFAULT_MODEL: &str = "gemini-2.5-flash";

pub struct ApiGeminiBackend {
    http: reqwest::Client,
    api_key: String,
    model: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GenerateContentRequest {
    contents: Vec<Content>,
    generation_config: GenerationConfig,
}

#[derive(Serialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Serialize)]
struct Part {
    text: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GenerationConfig {
    max_output_tokens: u32,
}

#[derive(Deserialize)]
struct GenerateContentResponse {
    candidates: Option<Vec<Candidate>>,
}

#[derive(Deserialize)]
struct Candidate {
    content: Option<CandidateContent>,
}

#[derive(Deserialize)]
struct CandidateContent {
    parts: Option<Vec<ResponsePart>>,
}

#[derive(Deserialize)]
struct ResponsePart {
    text: Option<String>,
}

impl ApiGeminiBackend {
    pub fn new(api_key: String, model: Option<String>) -> Self {
        Self {
            http: reqwest::Client::new(),
            api_key,
            model: model.unwrap_or_else(|| DEFAULT_MODEL.to_string()),
        }
    }
}

#[async_trait]
impl LlmBackend for ApiGeminiBackend {
    async fn send_message(&self, prompt: &str, max_tokens: u32) -> Result<String> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let req = GenerateContentRequest {
            contents: vec![Content {
                parts: vec![Part {
                    text: prompt.to_string(),
                }],
            }],
            generation_config: GenerationConfig {
                max_output_tokens: max_tokens,
            },
        };

        let resp = self
            .http
            .post(&url)
            .header("content-type", "application/json")
            .json(&req)
            .send()
            .await
            .context("Failed to call Gemini API")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            bail!("Gemini API error {}: {}", status, body);
        }

        let msg: GenerateContentResponse = resp
            .json()
            .await
            .context("Failed to parse Gemini response")?;

        let text = msg
            .candidates
            .as_deref()
            .unwrap_or_default()
            .iter()
            .filter_map(|c| c.content.as_ref())
            .flat_map(|c| c.parts.as_deref().unwrap_or_default())
            .filter_map(|p| p.text.as_deref())
            .collect::<Vec<_>>()
            .join("");

        if text.is_empty() {
            bail!("Gemini API returned empty response");
        }

        Ok(text)
    }
}
