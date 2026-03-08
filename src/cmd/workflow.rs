use anyhow::Result;
use serde::Serialize;
use tokio_rusqlite::Connection;
use tracing::{info, warn};

use crate::cmd::discover;
use crate::cmd::eval::{self, PositionInfo};
use crate::cmd::fetch;
use crate::config::AppConfig;
use crate::db;
use crate::indicators;
use crate::jquants::StockApi;
use crate::llm;
use crate::portfolio::{self, PositionView};
use crate::spec;

#[derive(Debug, Serialize)]
pub struct WorkflowReport {
    pub discover: Option<DiscoverStepResult>,
    pub stocks: Vec<StockWorkflowStatus>,
    pub errors: Vec<WorkflowError>,
}

#[derive(Debug, Serialize)]
pub struct DiscoverStepResult {
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub kept: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct StockWorkflowStatus {
    pub ticker: String,
    pub name: String,
    pub scan: StepStatus,
    pub fetch: StepStatus,
    pub eval: StepStatus,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum StepStatus {
    Success,
    Skipped,
    Failed(String),
}

#[derive(Debug, Serialize)]
pub struct WorkflowError {
    pub step: String,
    pub ticker: Option<String>,
    pub error: String,
}

pub async fn run(
    conn: &Connection,
    config: &AppConfig,
    stock_api: &dyn StockApi,
    skip: &[String],
) -> Result<WorkflowReport> {
    let mut report = WorkflowReport {
        discover: None,
        stocks: Vec::new(),
        errors: Vec::new(),
    };

    let skip_discover = skip.iter().any(|s| s == "discover");
    let skip_scan = skip.iter().any(|s| s == "scan");
    let skip_fetch = skip.iter().any(|s| s == "fetch");

    // === Discover ===
    if !skip_discover {
        info!("Workflow: running discover step");
        match discover::run(conn, config).await {
            Ok(result) => {
                info!(
                    added = result.added.len(),
                    removed = result.removed.len(),
                    kept = result.kept.len(),
                    "Discover step complete"
                );
                report.discover = Some(DiscoverStepResult {
                    added: result.added,
                    removed: result.removed,
                    kept: result.kept,
                });
            }
            Err(e) => {
                warn!(error = %e, "Discover step failed");
                report.errors.push(WorkflowError {
                    step: "discover".to_string(),
                    ticker: None,
                    error: e.to_string(),
                });
            }
        }
    } else {
        info!("Workflow: skipping discover step");
    }

    // Get watchlist for remaining steps
    let watchlist = db::watchlist_list(conn).await?;
    if watchlist.is_empty() {
        info!("Watchlist is empty, nothing to process");
        return Ok(report);
    }

    // Build initial stock statuses
    for item in &watchlist {
        report.stocks.push(StockWorkflowStatus {
            ticker: item.ticker.clone(),
            name: item.name.clone(),
            scan: if skip_scan {
                StepStatus::Skipped
            } else {
                StepStatus::Skipped
            },
            fetch: StepStatus::Skipped,
            eval: StepStatus::Skipped,
        });
    }

    // === Scan ===
    if !skip_scan {
        info!(count = watchlist.len(), "Workflow: running scan step");

        // Refresh master if needed
        if !db::has_any_stocks(conn).await? {
            info!("Stock master empty, refreshing from J-Quants API");
            match stock_api.get_all_stock_info().await {
                Ok(stocks) => {
                    let count = db::save_stocks_bulk(conn, &stocks).await?;
                    info!(count, "Stock master refreshed");
                }
                Err(e) => {
                    warn!(error = %e, "Failed to refresh stock master");
                    report.errors.push(WorkflowError {
                        step: "scan".to_string(),
                        ticker: None,
                        error: format!("Master refresh failed: {e}"),
                    });
                }
            }
        }

        let to_date = chrono::Local::now().format("%Y-%m-%d").to_string();
        let from_date = (chrono::Local::now() - chrono::Duration::days(60))
            .format("%Y-%m-%d")
            .to_string();

        for (i, item) in watchlist.iter().enumerate() {
            let stock_id = match db::get_stock_id(conn, &item.ticker).await? {
                Some(id) => id,
                None => {
                    let msg = "Stock not found in master data".to_string();
                    warn!(ticker = %item.ticker, "{}", msg);
                    report.stocks[i].scan = StepStatus::Failed(msg.clone());
                    report.errors.push(WorkflowError {
                        step: "scan".to_string(),
                        ticker: Some(item.ticker.clone()),
                        error: msg,
                    });
                    continue;
                }
            };

            if i > 0 {
                tokio::time::sleep(std::time::Duration::from_millis(300)).await;
            }

            match stock_api
                .get_daily_quotes(&item.ticker, &from_date, &to_date)
                .await
            {
                Ok(quotes) => {
                    if let Err(e) = db::save_prices(conn, stock_id, &quotes).await {
                        let msg = format!("Failed to save prices: {e}");
                        warn!(ticker = %item.ticker, "{}", msg);
                        report.stocks[i].scan = StepStatus::Failed(msg.clone());
                        report.errors.push(WorkflowError {
                            step: "scan".to_string(),
                            ticker: Some(item.ticker.clone()),
                            error: msg,
                        });
                    } else {
                        info!(ticker = %item.ticker, count = quotes.len(), "Scan complete");
                        report.stocks[i].scan = StepStatus::Success;
                    }
                }
                Err(e) => {
                    let msg = format!("API error: {e}");
                    warn!(ticker = %item.ticker, "{}", msg);
                    report.stocks[i].scan = StepStatus::Failed(msg.clone());
                    report.errors.push(WorkflowError {
                        step: "scan".to_string(),
                        ticker: Some(item.ticker.clone()),
                        error: msg,
                    });
                }
            }
        }
    } else {
        info!("Workflow: skipping scan step");
        // Mark all as Success to allow fetch/eval to proceed
        for status in &mut report.stocks {
            status.scan = StepStatus::Success;
        }
    }

    // === Fetch ===
    if !skip_fetch {
        let fetch_backend = llm::create_backend(
            &config.llm.fetch,
            &config.api,
            config.llm.fetch_model.as_deref(),
        )?;

        info!("Workflow: running fetch step");

        for (i, item) in watchlist.iter().enumerate() {
            if !matches!(report.stocks[i].scan, StepStatus::Success) {
                report.stocks[i].fetch = StepStatus::Skipped;
                continue;
            }

            let stock_id = match db::get_stock_id(conn, &item.ticker).await? {
                Some(id) => id,
                None => {
                    report.stocks[i].fetch = StepStatus::Skipped;
                    continue;
                }
            };

            let prompt = fetch::build_fetch_prompt(&item.ticker, &item.name);

            match fetch_backend.send_message(&prompt, 8192).await {
                Ok(response_text) => match fetch::parse_fetch_response(&response_text) {
                    Ok(items) => {
                        let mut saved = 0;
                        for fi in &items {
                            if let Err(e) = db::save_fetch_result(
                                conn,
                                stock_id,
                                &config.llm.fetch,
                                &fi.category,
                                &fi.title,
                                fi.url.as_deref(),
                                fi.body.as_deref(),
                                fi.published_at.as_deref(),
                            )
                            .await
                            {
                                warn!(ticker = %item.ticker, error = %e, "Failed to save fetch result");
                            } else {
                                saved += 1;
                            }
                        }
                        info!(ticker = %item.ticker, count = saved, "Fetch complete");
                        report.stocks[i].fetch = StepStatus::Success;
                    }
                    Err(e) => {
                        let msg = format!("Parse error: {e}");
                        warn!(ticker = %item.ticker, "{}", msg);
                        report.stocks[i].fetch = StepStatus::Failed(msg.clone());
                        report.errors.push(WorkflowError {
                            step: "fetch".to_string(),
                            ticker: Some(item.ticker.clone()),
                            error: msg,
                        });
                    }
                },
                Err(e) => {
                    let msg = format!("LLM error: {e}");
                    warn!(ticker = %item.ticker, "{}", msg);
                    report.stocks[i].fetch = StepStatus::Failed(msg.clone());
                    report.errors.push(WorkflowError {
                        step: "fetch".to_string(),
                        ticker: Some(item.ticker.clone()),
                        error: msg,
                    });
                }
            }
        }
    } else {
        info!("Workflow: skipping fetch step");
        // Mark scan-succeeded stocks as Success for fetch to allow eval
        for status in &mut report.stocks {
            if matches!(status.scan, StepStatus::Success) {
                status.fetch = StepStatus::Success;
            }
        }
    }

    // === Eval ===
    let eval_backend = llm::create_backend(
        &config.llm.eval,
        &config.api,
        config.llm.eval_model.as_deref(),
    )?;

    let loaded_spec = spec::load_spec(&config.spec.path).ok();
    let spec_section = loaded_spec.as_ref().map(|s| s.to_prompt_section());
    let spec_hash_val = spec::spec_hash(&config.spec.path).ok();
    let budget_initial_cash = loaded_spec.as_ref().and_then(|s| s.budget_initial_cash());
    let positions = portfolio::list_positions(conn).await?;
    let held_tickers: std::collections::HashSet<String> =
        positions.iter().map(|p| p.ticker.clone()).collect();

    let budget_context = if let Some(initial_cash) = budget_initial_cash {
        let cash_summary = db::trade_cash_summary(conn).await?;
        Some(spec::build_budget_context(
            initial_cash,
            cash_summary.total_invested,
            cash_summary.total_recovered,
            positions.len(),
        ))
    } else {
        None
    };

    info!("Workflow: running eval step");

    for (i, item) in watchlist.iter().enumerate() {
        if !matches!(report.stocks[i].fetch, StepStatus::Success) {
            report.stocks[i].eval = StepStatus::Skipped;
            continue;
        }

        let stock_id = match db::get_stock_id(conn, &item.ticker).await? {
            Some(id) => id,
            None => {
                report.stocks[i].eval = StepStatus::Skipped;
                continue;
            }
        };

        // Attempt eval for this stock, isolating errors
        match eval_single_stock(
            conn,
            &eval_backend,
            stock_id,
            &item.ticker,
            &item.name,
            &held_tickers,
            &positions,
            spec_section.as_deref(),
            spec_hash_val.as_deref(),
            budget_context.as_deref(),
            &config.llm.eval,
        )
        .await
        {
            Ok(()) => {
                info!(ticker = %item.ticker, "Eval complete");
                report.stocks[i].eval = StepStatus::Success;
            }
            Err(e) => {
                let msg = e.to_string();
                warn!(ticker = %item.ticker, error = %msg, "Eval failed");
                report.stocks[i].eval = StepStatus::Failed(msg.clone());
                report.errors.push(WorkflowError {
                    step: "eval".to_string(),
                    ticker: Some(item.ticker.clone()),
                    error: msg,
                });
            }
        }
    }

    // Summary
    let success_count = report
        .stocks
        .iter()
        .filter(|s| matches!(s.eval, StepStatus::Success))
        .count();
    let failed_count = report.errors.len();
    info!(
        total = report.stocks.len(),
        success = success_count,
        errors = failed_count,
        "Workflow complete"
    );

    Ok(report)
}

#[allow(clippy::too_many_arguments)]
async fn eval_single_stock(
    conn: &Connection,
    backend: &Box<dyn llm::LlmBackend>,
    stock_id: i64,
    ticker: &str,
    name: &str,
    held_tickers: &std::collections::HashSet<String>,
    positions: &[PositionView],
    spec_section: Option<&str>,
    spec_hash_val: Option<&str>,
    budget_context: Option<&str>,
    llm_backend_name: &str,
) -> Result<()> {
    let price_data = db::fetch_price_data(conn, stock_id).await?;
    if price_data.closes.len() < 14 {
        anyhow::bail!(
            "Insufficient data for evaluation (need >= 14 days, got {})",
            price_data.closes.len()
        );
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

    let fetch_results = db::get_fetch_results_for_stock(conn, stock_id).await?;
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

    let recent_evals = db::get_recent_evaluations_by_stock(conn, stock_id, 3).await?;
    let history_section = if recent_evals.is_empty() {
        None
    } else {
        let mut s = String::from("## Past Evaluations (most recent first)\n");
        for e in &recent_evals {
            let date = e.evaluated_at.split('T').next().unwrap_or(&e.evaluated_at);
            s.push_str(&format!(
                "- {}: {} (score: {}) — {}\n",
                date, e.decision, e.score, e.rationale
            ));
        }
        Some(s)
    };

    let status = if held_tickers.contains(ticker) {
        "ExistingHolding"
    } else {
        "NewTarget"
    };

    let position_info = positions.iter().find(|p| p.ticker == ticker).map(|p| {
        PositionInfo {
            quantity: p.quantity.to_string(),
            avg_cost: p.avg_cost.to_string(),
            unrealized_pnl_pct: p
                .unrealized_pnl_pct
                .map(|v| format!("{:.1}%", v))
                .unwrap_or_else(|| "N/A".to_string()),
        }
    });

    let prompt = eval::build_eval_prompt(
        ticker,
        name,
        status,
        &ta_json,
        &signals_str,
        &fetch_section,
        spec_section,
        position_info.as_ref(),
        budget_context,
        history_section.as_deref(),
    );

    let response_text = backend
        .send_message_with_schema(
            &prompt,
            4096,
            "eval_stock",
            "Evaluate a stock and return structured judgment",
            eval::eval_response_schema(),
        )
        .await?;
    let eval_response = eval::parse_eval_response(&response_text)?;

    db::save_evaluation(
        conn,
        stock_id,
        &eval_response.decision,
        eval_response.score,
        &serde_json::to_string(&eval_response.analysis)?,
        Some(&ta_json),
        spec_hash_val,
        Some(llm_backend_name),
    )
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_report_serialization() {
        let report = WorkflowReport {
            discover: Some(DiscoverStepResult {
                added: vec!["7203".to_string()],
                removed: vec![],
                kept: vec!["6758".to_string()],
            }),
            stocks: vec![
                StockWorkflowStatus {
                    ticker: "7203".to_string(),
                    name: "Toyota".to_string(),
                    scan: StepStatus::Success,
                    fetch: StepStatus::Success,
                    eval: StepStatus::Success,
                },
                StockWorkflowStatus {
                    ticker: "6758".to_string(),
                    name: "Sony".to_string(),
                    scan: StepStatus::Failed("API error".to_string()),
                    fetch: StepStatus::Skipped,
                    eval: StepStatus::Skipped,
                },
            ],
            errors: vec![WorkflowError {
                step: "scan".to_string(),
                ticker: Some("6758".to_string()),
                error: "API error".to_string(),
            }],
        };

        let json = serde_json::to_string_pretty(&report).unwrap();
        assert!(json.contains("\"ticker\": \"7203\""));
        assert!(json.contains("\"success\""));
        assert!(json.contains("\"skipped\""));
        assert!(json.contains("API error"));
    }

    #[test]
    fn test_step_status_serialization() {
        assert_eq!(
            serde_json::to_string(&StepStatus::Success).unwrap(),
            "\"success\""
        );
        assert_eq!(
            serde_json::to_string(&StepStatus::Skipped).unwrap(),
            "\"skipped\""
        );

        let failed = StepStatus::Failed("timeout".to_string());
        let json = serde_json::to_string(&failed).unwrap();
        assert!(json.contains("timeout"));
    }
}
