use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio_rusqlite::Connection;
use tracing::info;

use crate::config::AppConfig;
use crate::db;
use crate::indicators;
use crate::llm;
use crate::portfolio;
use crate::spec;
use tracing::warn;

#[derive(Debug, Serialize, Deserialize)]
pub struct EvalResponse {
    pub ticker: String,
    pub status: String,
    pub decision: String,
    pub score: i32,
    pub analysis: Analysis,
    #[serde(default)]
    pub execution_instruction: ExecutionInstruction,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Analysis {
    pub catalyst_check: String,
    pub risk_assessment: String,
    pub spec_compliance: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ExecutionInstruction {
    #[serde(default)]
    pub action: String,
    #[serde(default)]
    pub reason_for_exit: String,
}

#[derive(Debug, Serialize)]
pub struct EvalResult {
    pub ticker: String,
    pub name: String,
    pub status: String,
    pub decision: String,
    pub score: i32,
    pub analysis: Analysis,
    pub execution_instruction: ExecutionInstruction,
}

/// Represents a stock to evaluate with its context
struct EvalTarget {
    ticker: String,
    name: String,
    stock_id: i64,
    status: String,
    position_info: Option<PositionInfo>,
}

pub(crate) struct PositionInfo {
    pub(crate) quantity: String,
    pub(crate) avg_cost: String,
    pub(crate) unrealized_pnl_pct: String,
}

pub async fn run(
    conn: &Connection,
    config: &AppConfig,
    tickers: &[String],
) -> Result<Vec<EvalResult>> {
    let backend = llm::create_backend(
        &config.llm.eval,
        &config.api,
        config.llm.eval_model.as_deref(),
    )?;

    let targets = build_eval_targets(conn, tickers).await?;

    if targets.is_empty() {
        anyhow::bail!("No stocks to evaluate. Check your watchlist and portfolio.");
    }

    // Load spec if available
    let loaded_spec = match spec::load_spec(&config.spec.path) {
        Ok(s) => Some(s),
        Err(e) => {
            warn!(path = %config.spec.path, error = %e, "Failed to load spec, using defaults");
            None
        }
    };
    let spec_section = loaded_spec.as_ref().map(|s| s.to_prompt_section());
    let spec_hash_val = spec::spec_hash(&config.spec.path).ok();
    let budget_initial_cash = loaded_spec.as_ref().and_then(|s| s.budget_initial_cash());

    // Build budget context if initial_cash is configured
    let budget_context = if let Some(initial_cash) = budget_initial_cash {
        let cash_summary = db::trade_cash_summary(conn).await?;
        let positions = portfolio::list_positions(conn).await?;
        Some(spec::build_budget_context(
            initial_cash,
            cash_summary.total_invested,
            cash_summary.total_recovered,
            positions.len(),
        ))
    } else {
        None
    };

    let mut results = Vec::new();

    for target in &targets {
        let price_data = db::fetch_price_data(conn, target.stock_id).await?;
        if price_data.closes.len() < 14 {
            tracing::warn!(
                ticker = %target.ticker,
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

        // Get fetch results if available
        let fetch_results = db::get_fetch_results_for_stock(conn, target.stock_id).await?;
        let fetch_section = if fetch_results.is_empty() {
            "No recent information available.".to_string()
        } else {
            let mut s = String::new();
            for fr in fetch_results.iter().take(10) {
                s.push_str(&format!("- [{}] {}", fr.category, fr.title));
                if let Some(ref body) = fr.body {
                    s.push_str(&format!(": {}", body));
                }
                s.push('\n');
            }
            s
        };

        // Get recent evaluation history for this stock
        let recent_evals = db::get_recent_evaluations_by_stock(conn, target.stock_id, 3).await?;
        let history_section = if recent_evals.is_empty() {
            None
        } else {
            let mut s = String::from("## Past Evaluations (most recent first)\n");
            for eval in &recent_evals {
                let date = eval
                    .evaluated_at
                    .split('T')
                    .next()
                    .unwrap_or(&eval.evaluated_at);
                s.push_str(&format!(
                    "- {}: {} (score: {}) — {}\n",
                    date, eval.decision, eval.score, eval.rationale
                ));
            }
            Some(s)
        };

        let prompt = build_eval_prompt(
            &target.ticker,
            &target.name,
            &target.status,
            &ta_json,
            &signals_str,
            &fetch_section,
            spec_section.as_deref(),
            target.position_info.as_ref(),
            budget_context.as_deref(),
            history_section.as_deref(),
        );

        info!(ticker = %target.ticker, status = %target.status, backend = %config.llm.eval, "Running evaluation");

        let response_text = backend
            .send_message_with_schema(
                &prompt,
                4096,
                "eval_stock",
                "Evaluate a stock and return structured judgment",
                eval_response_schema(),
                Some(0.0),
            )
            .await?;

        if let Err(e) = db::save_llm_log(
            conn,
            "eval",
            Some(&target.ticker),
            &config.llm.eval,
            None,
            Some(0.0),
            &prompt,
            &response_text,
        )
        .await
        {
            warn!(error = %e, "Failed to save LLM log");
        }

        let eval_response = parse_eval_response(&response_text)?;

        db::save_evaluation(
            conn,
            target.stock_id,
            &eval_response.decision,
            eval_response.score,
            &serde_json::to_string(&eval_response.analysis)?,
            Some(&ta_json),
            spec_hash_val.as_deref(),
            Some(&config.llm.eval),
        )
        .await?;

        info!(
            ticker = %target.ticker,
            status = %target.status,
            decision = %eval_response.decision,
            score = eval_response.score,
            "Evaluation complete"
        );

        results.push(EvalResult {
            ticker: target.ticker.clone(),
            name: target.name.clone(),
            status: eval_response.status,
            decision: eval_response.decision,
            score: eval_response.score,
            analysis: eval_response.analysis,
            execution_instruction: eval_response.execution_instruction,
        });
    }

    Ok(results)
}

async fn build_eval_targets(conn: &Connection, tickers: &[String]) -> Result<Vec<EvalTarget>> {
    let watchlist = db::watchlist_list(conn).await?;
    let positions = portfolio::list_positions(conn).await?;

    let held_tickers: std::collections::HashSet<String> =
        positions.iter().map(|p| p.ticker.clone()).collect();

    let mut targets = Vec::new();
    let mut seen = std::collections::HashSet::new();

    // Watchlist items (Hunting or Farming if also held)
    for item in &watchlist {
        if !tickers.is_empty() && !tickers.iter().any(|t| t == &item.ticker) {
            continue;
        }
        seen.insert(item.ticker.clone());

        let position_info =
            positions
                .iter()
                .find(|p| p.ticker == item.ticker)
                .map(|p| PositionInfo {
                    quantity: p.quantity.to_string(),
                    avg_cost: p.avg_cost.to_string(),
                    unrealized_pnl_pct: p
                        .unrealized_pnl_pct
                        .map(|v| format!("{:.1}%", v))
                        .unwrap_or_else(|| "N/A".to_string()),
                });

        let status = if held_tickers.contains(&item.ticker) {
            "ExistingHolding"
        } else {
            "NewTarget"
        };

        targets.push(EvalTarget {
            ticker: item.ticker.clone(),
            name: item.name.clone(),
            stock_id: item.stock_id,
            status: status.to_string(),
            position_info,
        });
    }

    // Portfolio positions not in watchlist (Farming only)
    for pos in &positions {
        if seen.contains(&pos.ticker) {
            continue;
        }
        if !tickers.is_empty() && !tickers.iter().any(|t| t == &pos.ticker) {
            continue;
        }

        let stock_id = match db::get_stock_id(conn, &pos.ticker).await? {
            Some(id) => id,
            None => continue,
        };

        targets.push(EvalTarget {
            ticker: pos.ticker.clone(),
            name: pos.name.clone(),
            stock_id,
            status: "ExistingHolding".to_string(),
            position_info: Some(PositionInfo {
                quantity: pos.quantity.to_string(),
                avg_cost: pos.avg_cost.to_string(),
                unrealized_pnl_pct: pos
                    .unrealized_pnl_pct
                    .map(|v| format!("{:.1}%", v))
                    .unwrap_or_else(|| "N/A".to_string()),
            }),
        });
    }

    Ok(targets)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn build_eval_prompt(
    ticker: &str,
    name: &str,
    status: &str,
    ta_json: &str,
    signals: &str,
    fetch_info: &str,
    spec_section: Option<&str>,
    position_info: Option<&PositionInfo>,
    budget_context: Option<&str>,
    history_section: Option<&str>,
) -> String {
    let spec_part =
        spec_section.unwrap_or("No investment spec loaded. Use general best practices.");

    let budget_part = budget_context
        .map(|b| format!("\n{b}\n"))
        .unwrap_or_default();

    let history_part = history_section
        .map(|h| format!("\n{h}\n"))
        .unwrap_or_default();

    let status_section = match status {
        "ExistingHolding" => {
            let pos = position_info
                .map(|p| {
                    format!(
                        "- Status: ExistingHolding (保有中)\n- Quantity: {}\n- Average Cost: {}\n- Unrealized P&L: {}",
                        p.quantity, p.avg_cost, p.unrealized_pnl_pct
                    )
                })
                .unwrap_or_else(|| "- Status: ExistingHolding (保有中)".to_string());
            format!(
                "{}\n- Task: Decide whether to Hold or Sell. Check if the original investment thesis still holds.",
                pos
            )
        }
        _ => "- Status: NewTarget (新規候補)\n- Task: Decide whether to Buy or Avoid.".to_string(),
    };

    let decisions = if status == "ExistingHolding" {
        "Hold|Sell"
    } else {
        "Buy|Avoid"
    };

    format!(
        r#"You are an investment committee (AI投資委員会) evaluating a Japanese stock.

## Stock
- Ticker: {ticker}
- Name: {name}
{status_section}

## Technical Indicators (Latest Values)
{ta_json}

## Detected Signals
{signals}

## Recent Information
{fetch_info}

## Investment Policy
{spec_part}
{budget_part}{history_part}## Instructions
**IMPORTANT**: Past evaluations are reference context only. Your decision MUST be based on current data and fundamentals. If your decision differs from recent history, explain why.
Analyze this stock and provide your evaluation. Consider:
1. Catalyst validity — Is the original investment thesis or new catalyst still valid?
2. Risk assessment — Downside revision risk, FX sensitivity, technical overheating
3. Spec compliance — Does this stock meet the investment policy criteria?
4. Technical trend — Moving averages, momentum, volume patterns
5. Overall risk/reward

Respond ONLY with a JSON object in this exact format (no markdown, no code blocks):
{{
  "ticker": "{ticker}",
  "status": "{status}",
  "decision": "{decisions}",
  "score": 0-100,
  "analysis": {{
    "catalyst_check": "Investment thesis validity or new catalyst assessment",
    "risk_assessment": "Key downside risks and concerns",
    "spec_compliance": "How well this stock fits the investment policy"
  }},
  "execution_instruction": {{
    "action": "Specific action instruction",
    "reason_for_exit": "If Sell, why exit now (scenario breakdown, profit target, etc.)"
  }}
}}"#
    )
}

pub(crate) fn eval_response_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "required": ["ticker", "status", "decision", "score", "analysis"],
        "properties": {
            "ticker": { "type": "string" },
            "status": { "type": "string", "enum": ["NewTarget", "ExistingHolding"] },
            "decision": { "type": "string", "enum": ["Buy", "Avoid", "Hold", "Sell"] },
            "score": { "type": "integer", "minimum": 0, "maximum": 100 },
            "analysis": {
                "type": "object",
                "required": ["catalyst_check", "risk_assessment", "spec_compliance"],
                "properties": {
                    "catalyst_check": { "type": "string" },
                    "risk_assessment": { "type": "string" },
                    "spec_compliance": { "type": "string" }
                }
            },
            "execution_instruction": {
                "type": "object",
                "properties": {
                    "action": { "type": "string" },
                    "reason_for_exit": { "type": "string" }
                }
            }
        }
    })
}

pub(crate) fn parse_eval_response(text: &str) -> Result<EvalResponse> {
    let json_str = extract_json(text);
    serde_json::from_str(json_str).map_err(|e| {
        anyhow::anyhow!(
            "Failed to parse eval response as JSON: {}\nRaw: {}",
            e,
            text
        )
    })
}

pub(crate) fn extract_json(text: &str) -> &str {
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
    fn test_parse_eval_response() {
        let json = r#"{
            "ticker": "7203",
            "status": "NewTarget",
            "decision": "Buy",
            "score": 75,
            "analysis": {
                "catalyst_check": "Strong earnings growth",
                "risk_assessment": "FX exposure",
                "spec_compliance": "Meets all criteria"
            },
            "execution_instruction": {
                "action": "Buy at market open",
                "reason_for_exit": ""
            }
        }"#;
        let result = parse_eval_response(json).unwrap();
        assert_eq!(result.decision, "Buy");
        assert_eq!(result.score, 75);
        assert_eq!(result.status, "NewTarget");
        assert_eq!(result.analysis.catalyst_check, "Strong earnings growth");
    }

    #[test]
    fn test_parse_eval_response_sell() {
        let json = r#"{
            "ticker": "6758",
            "status": "ExistingHolding",
            "decision": "Sell",
            "score": 25,
            "analysis": {
                "catalyst_check": "Original thesis invalidated",
                "risk_assessment": "Earnings miss, guidance cut",
                "spec_compliance": "No longer meets ROE criteria"
            },
            "execution_instruction": {
                "action": "Sell all shares",
                "reason_for_exit": "Investment thesis broken — ROE dropped below threshold"
            }
        }"#;
        let result = parse_eval_response(json).unwrap();
        assert_eq!(result.decision, "Sell");
        assert_eq!(result.status, "ExistingHolding");
        assert!(!result.execution_instruction.reason_for_exit.is_empty());
    }

    #[test]
    fn test_parse_eval_response_with_markdown() {
        let text = r#"```json
{
    "ticker": "9984",
    "status": "ExistingHolding",
    "decision": "Hold",
    "score": 50,
    "analysis": {
        "catalyst_check": "Thesis intact",
        "risk_assessment": "Mixed signals",
        "spec_compliance": "Within bounds"
    }
}
```"#;
        let result = parse_eval_response(text).unwrap();
        assert_eq!(result.decision, "Hold");
    }

    #[test]
    fn test_extract_json() {
        let text = "Here is the result: {\"key\": \"value\"} done.";
        assert_eq!(extract_json(text), r#"{"key": "value"}"#);
    }

    #[test]
    fn test_build_eval_prompt_includes_history_section() {
        let history = "## Past Evaluations (most recent first)\n\
                       - 2026-03-07: Buy (score: 80) — Strong earnings\n\
                       - 2026-03-06: Avoid (score: 30) — High risk\n";
        let prompt = build_eval_prompt(
            "7203",
            "Toyota",
            "NewTarget",
            "{}",
            "None",
            "No recent information available.",
            None,
            None,
            None,
            Some(history),
        );
        assert!(prompt.contains("## Past Evaluations (most recent first)"));
        assert!(prompt.contains("Buy (score: 80)"));
        assert!(prompt.contains("Avoid (score: 30)"));
        assert!(prompt.contains("Past evaluations are reference context only"));
    }

    #[test]
    fn test_build_eval_prompt_no_history() {
        let prompt = build_eval_prompt(
            "7203",
            "Toyota",
            "NewTarget",
            "{}",
            "None",
            "No recent information available.",
            None,
            None,
            None,
            None,
        );
        assert!(!prompt.contains("## Past Evaluations"));
        // Anti-anchoring instruction is always present
        assert!(prompt.contains("Past evaluations are reference context only"));
    }

    #[test]
    fn test_build_eval_prompt_existing_holding_with_position() {
        let pos = PositionInfo {
            quantity: "100".to_string(),
            avg_cost: "2000".to_string(),
            unrealized_pnl_pct: "5.0%".to_string(),
        };
        let prompt = build_eval_prompt(
            "7203",
            "Toyota",
            "ExistingHolding",
            "{}",
            "None",
            "No recent information available.",
            None,
            Some(&pos),
            None,
            None,
        );
        assert!(prompt.contains("Hold|Sell"));
        assert!(prompt.contains("Quantity: 100"));
        assert!(prompt.contains("Average Cost: 2000"));
        assert!(prompt.contains("Unrealized P&L: 5.0%"));
    }
}
