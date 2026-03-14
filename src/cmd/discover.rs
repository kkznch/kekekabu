use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::config::AppConfig;
use crate::db::DbClient;
use crate::llm;
use crate::spec;

#[derive(Debug, Serialize, Deserialize)]
struct DiscoverResponse {
    #[serde(default)]
    keep: Vec<DiscoverAction>,
    #[serde(default)]
    add: Vec<DiscoverAction>,
    #[serde(default)]
    remove: Vec<DiscoverAction>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DiscoverAction {
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

pub async fn run(conn: &dyn DbClient, config: &AppConfig) -> Result<DiscoverResult> {
    let backend = llm::create_backend(
        &config.llm.fetch,
        &config.api,
        config.llm.fetch_model.as_deref(),
    )?;

    let loaded_spec = match spec::load_spec(&config.spec.path) {
        Ok(s) => Some(s),
        Err(e) => {
            warn!(path = %config.spec.path, error = %e, "Failed to load spec, using defaults");
            None
        }
    };
    let spec_section = loaded_spec.as_ref().map(|s| s.to_prompt_section());
    let budget_initial_cash = loaded_spec.as_ref().and_then(|s| s.budget_initial_cash());

    // Build budget context if initial_cash is configured
    let budget_context = if let Some(initial_cash) = budget_initial_cash {
        let cash_summary = conn.trade_cash_summary().await?;
        let positions = conn.list_positions().await?;
        Some(spec::build_budget_context(
            initial_cash,
            cash_summary.total_invested,
            cash_summary.total_recovered,
            positions.len(),
        ))
    } else {
        None
    };

    // Build watchlist context from current watchlist
    let current_watchlist = conn.watchlist_list().await?;
    let watchlist_context = if current_watchlist.is_empty() {
        None
    } else {
        let mut ctx = String::from("## Current Watchlist\n\n");
        for item in &current_watchlist {
            ctx.push_str(&format!("- {} ({})\n", item.ticker, item.name));
        }
        Some(ctx)
    };

    let prompt = build_discover_prompt(
        spec_section.as_deref(),
        budget_context.as_deref(),
        watchlist_context.as_deref(),
    );

    info!(backend = %config.llm.fetch, "Discovering stock candidates");

    let response_text = backend
        .send_message_with_schema(
            &prompt,
            8192,
            "discover_stocks",
            "Discover stock candidates and return structured watchlist actions",
            discover_response_schema(),
            None,
        )
        .await?;

    if let Err(e) = conn
        .save_llm_log(
            "discover",
            None,
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

    let response = parse_discover_response(&response_text)?;

    // Deduplicate: if a ticker appears in multiple lists, skip conflicting actions
    let mut seen_tickers: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut conflicts: std::collections::HashSet<String> = std::collections::HashSet::new();
    for action in response
        .add
        .iter()
        .chain(response.remove.iter())
        .chain(response.keep.iter())
    {
        if !seen_tickers.insert(action.ticker.clone()) {
            conflicts.insert(action.ticker.clone());
        }
    }
    for ticker in &conflicts {
        warn!(ticker = %ticker, "Ticker appears in multiple lists (add/remove/keep), skipping");
    }

    // Get held tickers to protect from removal
    let positions = conn.list_positions().await?;
    let held_tickers: std::collections::HashSet<String> =
        positions.iter().map(|p| p.ticker.clone()).collect();

    // Process add actions
    let mut added = Vec::new();
    for action in &response.add {
        if conflicts.contains(&action.ticker) {
            continue;
        }
        if !is_valid_ticker(&action.ticker) {
            warn!(ticker = %action.ticker, "Invalid ticker format, skipping");
            continue;
        }
        if !action.name.is_empty() {
            conn.save_stock(&action.ticker, &action.name, None).await?;
        }
        let notes = if action.reason.is_empty() {
            None
        } else {
            Some(action.reason.as_str())
        };
        conn.watchlist_add(&action.ticker, notes).await?;
        conn.save_watchlist_event(&action.ticker, "add", Some(&action.reason))
            .await?;
        info!(ticker = %action.ticker, name = %action.name, "Added to watchlist");
        added.push(action.ticker.clone());
    }

    // Process remove actions (protect held positions)
    let mut removed = Vec::new();
    let mut kept = Vec::new();
    for action in &response.remove {
        if conflicts.contains(&action.ticker) {
            continue;
        }
        if !is_valid_ticker(&action.ticker) {
            warn!(ticker = %action.ticker, "Invalid ticker format, skipping");
            continue;
        }
        if held_tickers.contains(&action.ticker) {
            info!(ticker = %action.ticker, "Kept in watchlist (has active position)");
            conn.save_watchlist_event(&action.ticker, "keep", Some("has active position"))
                .await?;
            kept.push(action.ticker.clone());
        } else {
            conn.watchlist_remove(&action.ticker).await?;
            conn.save_watchlist_event(&action.ticker, "remove", Some(&action.reason))
                .await?;
            info!(ticker = %action.ticker, "Removed from watchlist");
            removed.push(action.ticker.clone());
        }
    }

    // Process keep actions (log events)
    for action in &response.keep {
        if conflicts.contains(&action.ticker) {
            continue;
        }
        if !is_valid_ticker(&action.ticker) {
            warn!(ticker = %action.ticker, "Invalid ticker format, skipping");
            continue;
        }
        conn.save_watchlist_event(&action.ticker, "keep", Some(&action.reason))
            .await?;
        kept.push(action.ticker.clone());
    }

    info!(
        added = added.len(),
        removed = removed.len(),
        kept = kept.len(),
        "Discovery complete"
    );

    Ok(DiscoverResult {
        added,
        removed,
        kept,
    })
}

fn build_discover_prompt(
    spec_section: Option<&str>,
    budget_context: Option<&str>,
    watchlist_context: Option<&str>,
) -> String {
    let spec_part = spec_section
        .unwrap_or("No investment spec loaded. Use general best practices for JP stocks.");

    let budget_part = budget_context
        .map(|b| format!("\n{b}\n"))
        .unwrap_or_default();

    let watchlist_part = watchlist_context
        .map(|w| format!("\n{w}\n"))
        .unwrap_or_else(|| {
            "\n## Current Watchlist\n\nNo stocks currently tracked.\n\n".to_string()
        });

    format!(
        r#"You are a Japanese stock market research analyst. Your task is to review and update the investment watchlist based on the investment policy below.

## Investment Policy
{spec_part}
{budget_part}{watchlist_part}## Instructions
Review the current watchlist and investment policy above. Decide for each tracked stock whether to keep or remove it, and identify new promising candidates to add. Aim for a total of 10-20 stocks in the watchlist.

For each action, provide:
- ticker: 4-digit ticker code (e.g., "7203")
- name: Company name in Japanese (required for new additions)
- reason: Brief explanation of your decision (1-2 sentences)

Consider:
1. Fundamental value (PBR, PER, ROE as specified in policy)
2. Market cap and liquidity requirements from the policy
3. Recent catalysts (earnings surprises, restructuring, new products)
4. Sector trends and macro environment
5. Technical momentum

Respond ONLY with a JSON object in this exact format (no markdown, no code blocks):
{{
  "keep": [
    {{
      "ticker": "7203",
      "reason": "ROE改善トレンド継続中"
    }}
  ],
  "add": [
    {{
      "ticker": "6758",
      "name": "ソニーグループ",
      "reason": "新規カタリスト発生、PBR割安圏"
    }}
  ],
  "remove": [
    {{
      "ticker": "9984",
      "reason": "PBR基準を満たさなくなった"
    }}
  ]
}}"#
    )
}

fn discover_response_schema() -> serde_json::Value {
    let action_schema = serde_json::json!({
        "type": "object",
        "required": ["ticker"],
        "properties": {
            "ticker": { "type": "string" },
            "name": { "type": "string" },
            "reason": { "type": "string" }
        }
    });
    serde_json::json!({
        "type": "object",
        "properties": {
            "keep": {
                "type": "array",
                "items": action_schema
            },
            "add": {
                "type": "array",
                "items": action_schema
            },
            "remove": {
                "type": "array",
                "items": action_schema
            }
        }
    })
}

fn parse_discover_response(text: &str) -> Result<DiscoverResponse> {
    let json_str = extract_json(text);
    let response: DiscoverResponse = serde_json::from_str(json_str)
        .map_err(|e| anyhow::anyhow!("Failed to parse discover response: {}\nRaw: {}", e, text))?;
    Ok(response)
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
    (ticker.len() == 4 || ticker.len() == 5) && ticker.chars().all(|c| c.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_discover_response() {
        let json = r#"{"keep": [{"ticker": "7203", "reason": "継続"}], "add": [{"ticker": "6758", "name": "ソニー", "reason": "割安"}], "remove": [{"ticker": "9984", "reason": "基準外"}]}"#;
        let response = parse_discover_response(json).unwrap();
        assert_eq!(response.keep.len(), 1);
        assert_eq!(response.keep[0].ticker, "7203");
        assert_eq!(response.add.len(), 1);
        assert_eq!(response.add[0].ticker, "6758");
        assert_eq!(response.add[0].name, "ソニー");
        assert_eq!(response.remove.len(), 1);
        assert_eq!(response.remove[0].ticker, "9984");
    }

    #[test]
    fn test_parse_discover_response_with_markdown() {
        let text = "```json\n{\"keep\": [], \"add\": [{\"ticker\": \"6758\", \"name\": \"ソニー\", \"reason\": \"成長\"}], \"remove\": []}\n```";
        let response = parse_discover_response(text).unwrap();
        assert_eq!(response.add.len(), 1);
        assert_eq!(response.add[0].ticker, "6758");
    }

    #[test]
    fn test_parse_discover_response_partial() {
        let json = r#"{"add": [{"ticker": "9984", "name": "SBG", "reason": "割安"}]}"#;
        let response = parse_discover_response(json).unwrap();
        assert_eq!(response.add.len(), 1);
        assert!(response.keep.is_empty());
        assert!(response.remove.is_empty());
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
        assert!(is_valid_ticker("13060")); // ETF
        assert!(is_valid_ticker("25935")); // REIT
        assert!(!is_valid_ticker("723"));
        assert!(!is_valid_ticker("720345"));
        assert!(!is_valid_ticker("AAPL"));
        assert!(!is_valid_ticker("720A"));
        assert!(!is_valid_ticker(""));
    }
}
