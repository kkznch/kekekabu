use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio_rusqlite::Connection;
use tracing::info;

use crate::config::AppConfig;
use crate::db;
use crate::indicators;
use crate::llm;

#[derive(Debug, Serialize, Deserialize)]
pub struct EvalResponse {
    pub decision: String,
    pub score: i32,
    pub rationale: Rationale,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Rationale {
    pub summary: String,
    pub technical: String,
    pub risks: String,
}

#[derive(Debug, Serialize)]
pub struct EvalResult {
    pub ticker: String,
    pub name: String,
    pub decision: String,
    pub score: i32,
    pub rationale: Rationale,
}

pub async fn run(
    conn: &Connection,
    config: &AppConfig,
    tickers: &[String],
) -> Result<Vec<EvalResult>> {
    let backend = llm::create_backend(
        &config.llm.eval,
        config.api.anthropic_api_key.as_deref(),
        config.llm.eval_model.as_deref(),
    )?;

    let eval_tickers: Vec<db::WatchlistItem> = if tickers.is_empty() {
        db::watchlist_list(conn).await?
    } else {
        let all = db::watchlist_list(conn).await?;
        all.into_iter()
            .filter(|item| tickers.iter().any(|t| t == &item.ticker))
            .collect()
    };

    if eval_tickers.is_empty() {
        anyhow::bail!("No stocks to evaluate. Check your watchlist or ticker arguments.");
    }

    let mut results = Vec::new();

    for item in &eval_tickers {
        let stock_id = match db::get_stock_id(conn, &item.ticker).await? {
            Some(id) => id,
            None => {
                tracing::warn!(ticker = %item.ticker, "Stock not found in DB, skipping");
                continue;
            }
        };

        let price_data = db::fetch_price_data(conn, stock_id).await?;
        if price_data.closes.len() < 14 {
            tracing::warn!(
                ticker = %item.ticker,
                data_points = price_data.closes.len(),
                "Insufficient data for evaluation (need >= 14 days)"
            );
            continue;
        }

        let ta = indicators::calculate_indicators(
            &price_data.closes,
            &price_data.highs,
            &price_data.lows,
            &price_data.volumes,
        )?;

        let ta_json = serde_json::to_string_pretty(&ta.latest)?;
        let signals_str = if ta.signals.is_empty() {
            "None".to_string()
        } else {
            ta.signals.join(", ")
        };

        let prompt = build_eval_prompt(&item.ticker, &item.name, &ta_json, &signals_str);

        info!(ticker = %item.ticker, backend = %config.llm.eval, "Running evaluation");

        let response_text = backend.send_message(&prompt, 4096).await?;
        let eval_response = parse_eval_response(&response_text)?;

        db::save_evaluation(
            conn,
            stock_id,
            &eval_response.decision,
            eval_response.score,
            &serde_json::to_string(&eval_response.rationale)?,
            Some(&ta_json),
            None,
            Some(&config.llm.eval),
        )
        .await?;

        info!(
            ticker = %item.ticker,
            decision = %eval_response.decision,
            score = eval_response.score,
            "Evaluation complete"
        );

        results.push(EvalResult {
            ticker: item.ticker.clone(),
            name: item.name.clone(),
            decision: eval_response.decision,
            score: eval_response.score,
            rationale: eval_response.rationale,
        });
    }

    Ok(results)
}

fn build_eval_prompt(ticker: &str, name: &str, ta_json: &str, signals: &str) -> String {
    format!(
        r#"You are an investment committee evaluating a Japanese stock for potential investment.

## Stock
- Ticker: {ticker}
- Name: {name}

## Technical Indicators (Latest Values)
{ta_json}

## Detected Signals
{signals}

## Instructions
Analyze this stock and provide your evaluation. Consider:
1. Technical trend (moving averages, momentum)
2. Volatility (Bollinger Bands, ATR)
3. Volume patterns
4. Overall risk/reward

Respond ONLY with a JSON object in this exact format (no markdown, no code blocks):
{{
  "decision": "Buy|Hold|Avoid",
  "score": 0-100,
  "rationale": {{
    "summary": "One sentence overall assessment",
    "technical": "Technical analysis reasoning",
    "risks": "Key risks to consider"
  }}
}}"#
    )
}

fn parse_eval_response(text: &str) -> Result<EvalResponse> {
    let json_str = extract_json(text);
    serde_json::from_str(json_str)
        .map_err(|e| anyhow::anyhow!("Failed to parse eval response as JSON: {}\nRaw: {}", e, text))
}

fn extract_json(text: &str) -> &str {
    let text = text.trim();
    if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            return &text[start..=end];
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_eval_response() {
        let json = r#"{"decision": "Buy", "score": 75, "rationale": {"summary": "Good", "technical": "Bullish", "risks": "None"}}"#;
        let result = parse_eval_response(json).unwrap();
        assert_eq!(result.decision, "Buy");
        assert_eq!(result.score, 75);
    }

    #[test]
    fn test_parse_eval_response_with_markdown() {
        let text = "```json\n{\"decision\": \"Hold\", \"score\": 50, \"rationale\": {\"summary\": \"Neutral\", \"technical\": \"Mixed\", \"risks\": \"Some\"}}\n```";
        let result = parse_eval_response(text).unwrap();
        assert_eq!(result.decision, "Hold");
    }

    #[test]
    fn test_extract_json() {
        let text = "Here is the result: {\"key\": \"value\"} done.";
        assert_eq!(extract_json(text), r#"{"key": "value"}"#);
    }
}
