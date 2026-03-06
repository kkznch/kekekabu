use anyhow::Result;
use serde::Serialize;
use tokio_rusqlite::Connection;
use tracing::{info, warn};

use crate::circuit_breaker;
use crate::config::AppConfig;
use crate::db;

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

    let mut actions = Vec::new();

    for eval in &evals {
        let action = match eval.decision.as_str() {
            "Buy" if eval.score >= 70 => {
                if dry_run {
                    format!("[DRY RUN] Would place buy order for {} (score: {})", eval.ticker, eval.score)
                } else {
                    // TODO: Phase 3 full implementation would call Tachibana API here
                    // For now, we log the intent
                    format!(
                        "Buy signal recorded for {} (score: {}). Tachibana API integration pending.",
                        eval.ticker, eval.score
                    )
                }
            }
            "Buy" => {
                format!("Buy signal for {} but score too low ({} < 70), skipping", eval.ticker, eval.score)
            }
            "Avoid" if eval.score <= 30 => {
                // Check if we have a position to consider selling
                format!(
                    "Avoid signal for {} (score: {}). Review existing positions.",
                    eval.ticker, eval.score
                )
            }
            _ => {
                format!("Hold for {} (score: {})", eval.ticker, eval.score)
            }
        };

        let action_type = if eval.decision == "Buy" && eval.score >= 70 {
            "buy_signal"
        } else if eval.decision == "Avoid" && eval.score <= 30 {
            "sell_signal"
        } else {
            "hold"
        };

        actions.push(ExecuteAction {
            ticker: eval.ticker.clone(),
            name: eval.name.clone(),
            decision: eval.decision.clone(),
            score: eval.score,
            action: action_type.to_string(),
            detail: action,
        });
    }

    Ok(ExecuteResult {
        actions,
        circuit_breaker_triggered: false,
        circuit_breaker_reasons: vec![],
    })
}
