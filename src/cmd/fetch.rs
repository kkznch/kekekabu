use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::config::AppConfig;
use crate::db::DbClient;
use crate::llm;

#[derive(Debug, Serialize, Deserialize)]
struct GeminiFetchResponse {
    items: Vec<GeminiFetchItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct GeminiFetchItem {
    pub(crate) category: String,
    pub(crate) title: String,
    #[serde(default)]
    pub(crate) url: Option<String>,
    #[serde(default)]
    pub(crate) body: Option<String>,
    #[serde(default)]
    pub(crate) published_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct FetchSummary {
    pub ticker: String,
    pub name: String,
    pub items_saved: usize,
}

pub async fn run(
    conn: &dyn DbClient,
    config: &AppConfig,
    tickers: &[String],
) -> Result<Vec<FetchSummary>> {
    let backend = llm::create_backend(
        &config.llm.fetch,
        &config.api,
        config.llm.fetch_model.as_deref(),
    )?;

    let watchlist = conn.watchlist_list().await?;
    let targets: Vec<_> = if tickers.is_empty() {
        watchlist
    } else {
        watchlist
            .into_iter()
            .filter(|item| tickers.iter().any(|t| t == &item.ticker))
            .collect()
    };

    if targets.is_empty() {
        anyhow::bail!("No stocks to fetch. Check your watchlist or ticker arguments.");
    }

    let mut results = Vec::new();

    for item in &targets {
        let stock_id = match conn.get_stock_id(&item.ticker).await? {
            Some(id) => id,
            None => {
                tracing::warn!(ticker = %item.ticker, "Stock not found in DB, skipping");
                continue;
            }
        };

        let prompt = build_fetch_prompt(&item.ticker, &item.name);

        info!(ticker = %item.ticker, backend = %config.llm.fetch, "Fetching information");

        let response_text = backend.send_message(&prompt, 8192, None).await?;

        if let Err(e) = conn
            .save_llm_log(
                "fetch",
                Some(&item.ticker),
                &config.llm.fetch,
                None,
                None,
                &prompt,
                &response_text,
            )
            .await
        {
            warn!(error = %e, "Failed to save LLM log");
        }

        let items = parse_fetch_response(&response_text)?;

        let mut saved_count = 0;
        for fi in &items {
            conn.save_fetch_result(
                stock_id,
                &config.llm.fetch,
                &fi.category,
                &fi.title,
                fi.url.as_deref(),
                fi.body.as_deref(),
                fi.published_at.as_deref(),
            )
            .await?;
            saved_count += 1;
        }

        info!(ticker = %item.ticker, count = saved_count, "Saved fetch results");

        results.push(FetchSummary {
            ticker: item.ticker.clone(),
            name: item.name.clone(),
            items_saved: saved_count,
        });
    }

    Ok(results)
}

pub(crate) fn build_fetch_prompt(ticker: &str, name: &str) -> String {
    format!(
        r#"You are a financial research analyst. Gather the latest information about the following Japanese stock.

## Stock
- Ticker: {ticker}
- Name: {name}

## Instructions
Search for and summarize the following categories of information:
1. **news** - Recent news articles about this company
2. **disclosure** - Recent TDnet filings, earnings reports, IR releases
3. **sentiment** - Market sentiment, analyst opinions, social media buzz
4. **competitor** - Competitor movements and industry trends

For each item found, provide:
- category: one of "news", "disclosure", "sentiment", "competitor"
- title: brief title
- url: source URL if available
- body: summary text (2-3 sentences)
- published_at: publication date in YYYY-MM-DD format if known

Respond ONLY with a JSON object in this exact format (no markdown, no code blocks):
{{
  "items": [
    {{
      "category": "news",
      "title": "...",
      "url": "...",
      "body": "...",
      "published_at": "..."
    }}
  ]
}}"#
    )
}

pub(crate) fn parse_fetch_response(text: &str) -> Result<Vec<GeminiFetchItem>> {
    let json_str = extract_json(text);
    let response: GeminiFetchResponse = serde_json::from_str(json_str)
        .map_err(|e| anyhow::anyhow!("Failed to parse fetch response: {}\nRaw: {}", e, text))?;
    Ok(response.items)
}

fn extract_json(text: &str) -> &str {
    let text = text.trim();
    if let Some(start) = text.find('{')
        && let Some(end) = text.rfind('}')
    {
        return &text[start..=end];
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_fetch_response() {
        let json = r#"{"items": [{"category": "news", "title": "Test News", "url": "https://example.com", "body": "Some news body", "published_at": "2024-01-01"}]}"#;
        let items = parse_fetch_response(json).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].category, "news");
        assert_eq!(items[0].title, "Test News");
    }

    #[test]
    fn test_parse_fetch_response_minimal() {
        let json = r#"{"items": [{"category": "sentiment", "title": "Bullish outlook"}]}"#;
        let items = parse_fetch_response(json).unwrap();
        assert_eq!(items.len(), 1);
        assert!(items[0].url.is_none());
    }
}
