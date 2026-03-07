use anyhow::Result;
use serde::Serialize;
use tokio_rusqlite::Connection;
use tracing::{info, warn};

use crate::circuit_breaker;
use crate::config::AppConfig;
use crate::db;
use crate::portfolio;

#[derive(Debug, Serialize)]
pub struct ExecuteResult {
    pub actions: Vec<ExecuteAction>,
    pub circuit_breaker_triggered: bool,
    pub circuit_breaker_reasons: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ExecuteAction {
    pub ticker: String,
    pub name: String,
    pub decision: String,
    pub score: i32,
    pub action: String,
    pub detail: String,
}

pub async fn run(
    conn: &Connection,
    _config: &AppConfig,
    dry_run: bool,
) -> Result<ExecuteResult> {
    // 1. Circuit breaker check
    let cb = circuit_breaker::check(conn).await?;
    if !cb.safe {
        warn!("Circuit breaker triggered! Aborting execute.");
        return Ok(ExecuteResult {
            actions: vec![],
            circuit_breaker_triggered: true,
            circuit_breaker_reasons: cb.reasons,
        });
    }

    // 2. Get today's evaluations
    let evals = db::get_latest_evaluations_for_today(conn).await?;
    if evals.is_empty() {
        info!("No evaluations for today. Nothing to execute.");
        return Ok(ExecuteResult {
            actions: vec![],
            circuit_breaker_triggered: false,
            circuit_breaker_reasons: vec![],
        });
    }

    // 3. Get positions for Sell checks
    let positions = portfolio::list_positions(conn).await?;
    let held_tickers: std::collections::HashSet<String> =
        positions.iter().map(|p| p.ticker.clone()).collect();

    let mut actions = Vec::new();

    for eval in &evals {
        let (action_type, detail) = match eval.decision.as_str() {
            "Buy" if eval.score >= 70 => {
                let detail = if dry_run {
                    format!("[DRY RUN] Would place buy order for {} (score: {})", eval.ticker, eval.score)
                } else {
                    format!(
                        "Buy signal recorded for {} (score: {}). Tachibana API integration pending.",
                        eval.ticker, eval.score
                    )
                };
                ("buy_signal", detail)
            }
            "Buy" => {
                ("hold", format!("Buy signal for {} but score too low ({} < 70), skipping", eval.ticker, eval.score))
            }
            "Sell" => {
                if held_tickers.contains(&eval.ticker) {
                    let detail = if dry_run {
                        format!("[DRY RUN] Would place sell order for {} (score: {})", eval.ticker, eval.score)
                    } else {
                        format!(
                            "Sell signal recorded for {} (score: {}). Tachibana API integration pending.",
                            eval.ticker, eval.score
                        )
                    };
                    ("sell_signal", detail)
                } else {
                    ("hold", format!("Sell signal for {} but no position held, skipping", eval.ticker))
                }
            }
            "Avoid" if eval.score <= 30 => {
                ("sell_signal", format!(
                    "Avoid signal for {} (score: {}). Review existing positions.",
                    eval.ticker, eval.score
                ))
            }
            _ => {
                ("hold", format!("Hold for {} (score: {})", eval.ticker, eval.score))
            }
        };

        actions.push(ExecuteAction {
            ticker: eval.ticker.clone(),
            name: eval.name.clone(),
            decision: eval.decision.clone(),
            score: eval.score,
            action: action_type.to_string(),
            detail,
        });
    }

    Ok(ExecuteResult {
        actions,
        circuit_breaker_triggered: false,
        circuit_breaker_reasons: vec![],
    })
}
