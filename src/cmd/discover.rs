use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio_rusqlite::Connection;
use tracing::{info, warn};

use crate::config::AppConfig;
use crate::db;
use crate::llm;
use crate::portfolio;
use crate::spec;

#[derive(Debug, Serialize, Deserialize)]
struct DiscoverResponse {
    candidates: Vec<DiscoverCandidate>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DiscoverCandidate {
    ticker: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    reason: String,
}

#[derive(Debug, Serialize)]
pub struct DiscoverResult {
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub kept: Vec<String>,
}

pub async fn run(conn: &Connection, config: &AppConfig) -> Result<DiscoverResult> {
    let backend = llm::create_backend(
        &config.llm.fetch,
        &config.api,
        config.llm.fetch_model.as_deref(),
    )?;

    let spec_section = spec::load_spec(&config.spec.path)
        .ok()
        .map(|s| s.to_prompt_section());

    let prompt = build_discover_prompt(spec_section.as_deref());

    info!(backend = %config.llm.fetch, "Discovering stock candidates");

    let response_text = backend.send_message(&prompt, 8192).await?;
    let candidates = parse_discover_response(&response_text)?;

    let valid_tickers: Vec<&DiscoverCandidate> = candidates
        .iter()
        .filter(|c| {
            if is_valid_ticker(&c.ticker) {
                true
            } else {
                warn!(ticker = %c.ticker, "Invalid ticker format, skipping");
                false
            }
        })
        .collect();

    // Diff management
    let current_watchlist = db::watchlist_list(conn).await?;
    let current_tickers: std::collections::HashSet<String> = current_watchlist
        .iter()
        .map(|w| w.ticker.clone())
        .collect();
    let new_tickers: std::collections::HashSet<String> =
        valid_tickers.iter().map(|c| c.ticker.clone()).collect();

    // Get held tickers to protect from removal
    let positions = portfolio::list_positions(conn).await?;
    let held_tickers: std::collections::HashSet<String> =
        positions.iter().map(|p| p.ticker.clone()).collect();

    // Add new tickers
    let mut added = Vec::new();
    for candidate in &valid_tickers {
        if !current_tickers.contains(&candidate.ticker) {
            let notes = if candidate.reason.is_empty() {
                None
            } else {
                Some(candidate.reason.as_str())
            };
            db::watchlist_add(conn, &candidate.ticker, notes).await?;
            info!(ticker = %candidate.ticker, "Added to watchlist");
            added.push(candidate.ticker.clone());
        }
    }

    // Remove tickers no longer in discover list (but keep held ones)
    let mut removed = Vec::new();
    let mut kept = Vec::new();
    for item in &current_watchlist {
        if !new_tickers.contains(&item.ticker) {
            if held_tickers.contains(&item.ticker) {
                info!(ticker = %item.ticker, "Kept in watchlist (has active position)");
                kept.push(item.ticker.clone());
            } else {
                db::watchlist_remove(conn, &item.ticker).await?;
                info!(ticker = %item.ticker, "Removed from watchlist");
                removed.push(item.ticker.clone());
            }
        }
    }

    info!(
        added = added.len(),
        removed = removed.len(),
        kept = kept.len(),
        total = new_tickers.len(),
        "Discovery complete"
    );

    Ok(DiscoverResult {
        added,
        removed,
        kept,
    })
}

pub async fn list(conn: &Connection) -> Result<Vec<db::WatchlistItem>> {
    db::watchlist_list(conn).await
}

fn build_discover_prompt(spec_section: Option<&str>) -> String {
    let spec_part =
        spec_section.unwrap_or("No investment spec loaded. Use general best practices for JP stocks.");

    format!(
        r#"You are a Japanese stock market research analyst. Your task is to discover promising investment candidates based on the investment policy below.

## Investment Policy
{spec_part}

## Instructions
Based on the investment policy above, identify 10-20 promising Japanese stock candidates that match the criteria.

Consider:
1. Fundamental value (PBR, PER, ROE as specified in policy)
2. Market cap and liquidity requirements from the policy
3. Recent catalysts (earnings surprises, restructuring, new products)
4. Sector trends and macro environment
5. Technical momentum

For each candidate, provide:
- ticker: 4-digit ticker code (e.g., "7203")
- name: Company name in Japanese
- reason: Brief explanation of why this stock fits the policy (1-2 sentences)

Respond ONLY with a JSON object in this exact format (no markdown, no code blocks):
{{
  "candidates": [
    {{
      "ticker": "7203",
      "name": "トヨタ自動車",
      "reason": "PBR 1.0倍割れで割安、ROE改善トレンド"
    }}
  ]
}}"#
    )
}

fn parse_discover_response(text: &str) -> Result<Vec<DiscoverCandidate>> {
    let json_str = extract_json(text);
    let response: DiscoverResponse = serde_json::from_str(json_str)
        .map_err(|e| anyhow::anyhow!("Failed to parse discover response: {}\nRaw: {}", e, text))?;
    Ok(response.candidates)
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

fn is_valid_ticker(ticker: &str) -> bool {
    ticker.len() == 4 && ticker.chars().all(|c| c.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_discover_response() {
        let json = r#"{"candidates": [{"ticker": "7203", "name": "トヨタ自動車", "reason": "割安"}]}"#;
        let candidates = parse_discover_response(json).unwrap();
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].ticker, "7203");
        assert_eq!(candidates[0].name, "トヨタ自動車");
    }

    #[test]
    fn test_parse_discover_response_with_markdown() {
        let text = "```json\n{\"candidates\": [{\"ticker\": \"6758\", \"name\": \"ソニー\", \"reason\": \"成長\"}]}\n```";
        let candidates = parse_discover_response(text).unwrap();
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].ticker, "6758");
    }

    #[test]
    fn test_parse_discover_response_minimal() {
        let json = r#"{"candidates": [{"ticker": "9984"}]}"#;
        let candidates = parse_discover_response(json).unwrap();
        assert_eq!(candidates.len(), 1);
        assert!(candidates[0].name.is_empty());
        assert!(candidates[0].reason.is_empty());
    }

    #[test]
    fn test_parse_discover_response_invalid() {
        let text = "This is not JSON";
        assert!(parse_discover_response(text).is_err());
    }

    #[test]
    fn test_is_valid_ticker() {
        assert!(is_valid_ticker("7203"));
        assert!(is_valid_ticker("9984"));
        assert!(!is_valid_ticker("723"));
        assert!(!is_valid_ticker("72034"));
        assert!(!is_valid_ticker("AAPL"));
        assert!(!is_valid_ticker("720A"));
        assert!(!is_valid_ticker(""));
    }
}
