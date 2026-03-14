use anyhow::Result;
use serde::Serialize;
use tracing::{info, warn};

use crate::cmd::discover;
use crate::cmd::eval::{self, PositionInfo};
use crate::cmd::fetch;
use crate::config::AppConfig;
use crate::db::{DbClient, WatchlistItem};
use crate::indicators;
use crate::jquants::StockApi;
use crate::llm;
use crate::portfolio::PositionView;
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

impl WorkflowReport {
    fn new() -> Self {
        Self {
            discover: None,
            stocks: Vec::new(),
            errors: Vec::new(),
        }
    }

    fn init_stocks(&mut self, watchlist: &[WatchlistItem]) {
        self.stocks = watchlist
            .iter()
            .map(|item| StockWorkflowStatus {
                ticker: item.ticker.clone(),
                name: item.name.clone(),
                scan: StepStatus::Skipped,
                fetch: StepStatus::Skipped,
                eval: StepStatus::Skipped,
            })
            .collect();
    }

    fn apply_result(&mut self, index: usize, step: &str, ticker: &str, result: Result<()>) {
        let status = match result {
            Ok(()) => StepStatus::Success,
            Err(e) => {
                let error = e.to_string();
                warn!(step, ticker, error = %error, "Step failed");
                self.errors.push(WorkflowError {
                    step: step.to_string(),
                    ticker: Some(ticker.to_string()),
                    error: error.clone(),
                });
                StepStatus::Failed(error)
            }
        };
        match step {
            "scan" => self.stocks[index].scan = status,
            "fetch" => self.stocks[index].fetch = status,
            "eval" => self.stocks[index].eval = status,
            _ => {}
        }
    }

    fn log_summary(&self) {
        let success_count = self
            .stocks
            .iter()
            .filter(|s| matches!(s.eval, StepStatus::Success))
            .count();
        info!(
            total = self.stocks.len(),
            success = success_count,
            errors = self.errors.len(),
            "Workflow complete"
        );
    }
}

// ─── Pipeline ───────────────────────────────────────────────────────

const VALID_SKIP_STEPS: &[&str] = &["discover", "scan", "fetch"];

pub async fn run(
    conn: &dyn DbClient,
    config: &AppConfig,
    stock_api: &dyn StockApi,
    skip: &[String],
) -> Result<WorkflowReport> {
    let mut report = WorkflowReport::new();

    for s in skip {
        if !VALID_SKIP_STEPS.contains(&s.as_str()) {
            warn!(
                step = %s,
                valid = ?VALID_SKIP_STEPS,
                "Unknown --skip value, ignoring"
            );
        }
    }

    step_discover(&mut report, conn, config, skip).await;

    let watchlist = conn.watchlist_list().await?;
    if watchlist.is_empty() {
        info!("Watchlist is empty, nothing to process");
        return Ok(report);
    }
    report.init_stocks(&watchlist);

    step_scan(&mut report, conn, stock_api, &watchlist, skip).await?;
    step_fetch(&mut report, conn, config, &watchlist, skip).await?;
    step_eval(&mut report, conn, config, &watchlist).await?;

    report.log_summary();
    Ok(report)
}

// ─── Steps ──────────────────────────────────────────────────────────

async fn step_discover(
    report: &mut WorkflowReport,
    conn: &dyn DbClient,
    config: &AppConfig,
    skip: &[String],
) {
    if skip.iter().any(|s| s == "discover") {
        info!("Workflow: skipping discover step");
        return;
    }

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
}

async fn step_scan(
    report: &mut WorkflowReport,
    conn: &dyn DbClient,
    stock_api: &dyn StockApi,
    watchlist: &[WatchlistItem],
    skip: &[String],
) -> Result<()> {
    if skip.iter().any(|s| s == "scan") {
        info!("Workflow: skipping scan step");
        for status in &mut report.stocks {
            status.scan = StepStatus::Success;
        }
        return Ok(());
    }

    info!(count = watchlist.len(), "Workflow: running scan step");

    if !conn.has_any_stocks().await? {
        info!("Stock master empty, refreshing from J-Quants API");
        if let Err(e) = refresh_stock_master(conn, stock_api).await {
            warn!(error = %e, "Failed to refresh stock master");
            report.errors.push(WorkflowError {
                step: "scan".to_string(),
                ticker: None,
                error: format!("Master refresh failed: {e}"),
            });
        }
    }

    let to_date = chrono::Local::now().format("%Y-%m-%d").to_string();
    let from_date = (chrono::Local::now() - chrono::Duration::days(60))
        .format("%Y-%m-%d")
        .to_string();

    for (i, item) in watchlist.iter().enumerate() {
        if i > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        }
        let result = scan_single_stock(conn, stock_api, &item.ticker, &from_date, &to_date).await;
        report.apply_result(i, "scan", &item.ticker, result);
    }
    Ok(())
}

async fn step_fetch(
    report: &mut WorkflowReport,
    conn: &dyn DbClient,
    config: &AppConfig,
    watchlist: &[WatchlistItem],
    skip: &[String],
) -> Result<()> {
    if skip.iter().any(|s| s == "fetch") {
        info!("Workflow: skipping fetch step");
        for status in &mut report.stocks {
            if matches!(status.scan, StepStatus::Success) {
                status.fetch = StepStatus::Success;
            }
        }
        return Ok(());
    }

    let backend = llm::create_backend(
        &config.llm.fetch,
        &config.api,
        config.llm.fetch_model.as_deref(),
    )?;

    info!("Workflow: running fetch step");

    for (i, item) in watchlist.iter().enumerate() {
        if !matches!(report.stocks[i].scan, StepStatus::Success) {
            continue;
        }
        let result = fetch_single_stock(
            conn,
            backend.as_ref(),
            &item.ticker,
            &item.name,
            &config.llm.fetch,
        )
        .await;
        report.apply_result(i, "fetch", &item.ticker, result);
    }
    Ok(())
}

async fn step_eval(
    report: &mut WorkflowReport,
    conn: &dyn DbClient,
    config: &AppConfig,
    watchlist: &[WatchlistItem],
) -> Result<()> {
    let backend = llm::create_backend(
        &config.llm.eval,
        &config.api,
        config.llm.eval_model.as_deref(),
    )?;

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
    let positions = conn.list_positions().await?;
    let held_tickers: std::collections::HashSet<String> =
        positions.iter().map(|p| p.ticker.clone()).collect();

    let budget_context = match budget_initial_cash {
        Some(initial_cash) => {
            let cash_summary = conn.trade_cash_summary().await?;
            Some(spec::build_budget_context(
                initial_cash,
                cash_summary.total_invested,
                cash_summary.total_recovered,
                positions.len(),
            ))
        }
        None => None,
    };

    info!("Workflow: running eval step");

    for (i, item) in watchlist.iter().enumerate() {
        if !matches!(report.stocks[i].fetch, StepStatus::Success) {
            continue;
        }
        let result = eval_single_stock_full(
            conn,
            backend.as_ref(),
            &item.ticker,
            &item.name,
            &held_tickers,
            &positions,
            spec_section.as_deref(),
            spec_hash_val.as_deref(),
            budget_context.as_deref(),
            &config.llm.eval,
        )
        .await;
        report.apply_result(i, "eval", &item.ticker, result);
    }
    Ok(())
}

// ─── Per-stock operations ───────────────────────────────────────────

async fn refresh_stock_master(conn: &dyn DbClient, stock_api: &dyn StockApi) -> Result<()> {
    let stocks = stock_api.get_all_stock_info().await?;
    conn.save_stocks_bulk(&stocks).await?;
    Ok(())
}

async fn scan_single_stock(
    conn: &dyn DbClient,
    stock_api: &dyn StockApi,
    ticker: &str,
    from_date: &str,
    to_date: &str,
) -> Result<()> {
    let stock_id = conn
        .get_stock_id(ticker)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Stock not found in master data"))?;
    let quotes = stock_api
        .get_daily_quotes(ticker, from_date, to_date)
        .await?;
    conn.save_prices(stock_id, &quotes).await?;
    info!(ticker = %ticker, count = quotes.len(), "Scan complete");
    Ok(())
}

async fn fetch_single_stock(
    conn: &dyn DbClient,
    backend: &dyn llm::LlmBackend,
    ticker: &str,
    name: &str,
    llm_backend_name: &str,
) -> Result<()> {
    let stock_id = conn
        .get_stock_id(ticker)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Stock not found"))?;
    let prompt = fetch::build_fetch_prompt(ticker, name);
    let response_text = backend.send_message(&prompt, 8192, None).await?;

    if let Err(e) = conn
        .save_llm_log(
            "fetch",
            Some(ticker),
            llm_backend_name,
            None,
            None,
            &prompt,
            &response_text,
        )
        .await
    {
        warn!(error = %e, "Failed to save LLM log");
    }

    let items = fetch::parse_fetch_response(&response_text)?;
    for fi in &items {
        if let Err(e) = conn
            .save_fetch_result(
                stock_id,
                llm_backend_name,
                &fi.category,
                &fi.title,
                fi.url.as_deref(),
                fi.body.as_deref(),
                fi.published_at.as_deref(),
            )
            .await
        {
            warn!(ticker = %ticker, error = %e, "Failed to save fetch result");
        }
    }
    info!(ticker = %ticker, count = items.len(), "Fetch complete");
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn eval_single_stock_full(
    conn: &dyn DbClient,
    backend: &dyn llm::LlmBackend,
    ticker: &str,
    name: &str,
    held_tickers: &std::collections::HashSet<String>,
    positions: &[PositionView],
    spec_section: Option<&str>,
    spec_hash_val: Option<&str>,
    budget_context: Option<&str>,
    llm_backend_name: &str,
) -> Result<()> {
    let stock_id = conn
        .get_stock_id(ticker)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Stock not found"))?;

    let price_data = conn.fetch_price_data(stock_id).await?;
    anyhow::ensure!(
        price_data.closes.len() >= 14,
        "Insufficient data for evaluation (need >= 14 days, got {})",
        price_data.closes.len()
    );

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

    let fetch_results = conn.get_fetch_results_for_stock(stock_id).await?;
    let fetch_section = if fetch_results.is_empty() {
        "No recent information available.".to_string()
    } else {
        fetch_results
            .iter()
            .take(10)
            .map(|fr| match &fr.body {
                Some(body) => format!("- [{}] {}: {}", fr.category, fr.title, body),
                None => format!("- [{}] {}", fr.category, fr.title),
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    let history_section = conn
        .get_recent_evaluations_by_stock(stock_id, 3)
        .await?
        .iter()
        .fold(None, |acc, e| {
            let date = e.evaluated_at.split('T').next().unwrap_or(&e.evaluated_at);
            let line = format!(
                "- {}: {} (score: {}) — {}\n",
                date, e.decision, e.score, e.rationale
            );
            Some(match acc {
                None => format!("## Past Evaluations (most recent first)\n{line}"),
                Some(s) => format!("{s}{line}"),
            })
        });

    let status = if held_tickers.contains(ticker) {
        "ExistingHolding"
    } else {
        "NewTarget"
    };

    let position_info = positions
        .iter()
        .find(|p| p.ticker == ticker)
        .map(|p| PositionInfo {
            quantity: p.quantity.to_string(),
            avg_cost: p.avg_cost.to_string(),
            unrealized_pnl_pct: p
                .unrealized_pnl_pct
                .map(|v| format!("{:.1}%", v))
                .unwrap_or_else(|| "N/A".to_string()),
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
            Some(0.0),
        )
        .await?;

    if let Err(e) = conn
        .save_llm_log(
            "eval",
            Some(ticker),
            llm_backend_name,
            None,
            Some(0.0),
            &prompt,
            &response_text,
        )
        .await
    {
        warn!(error = %e, "Failed to save LLM log");
    }

    let eval_response = eval::parse_eval_response(&response_text)?;

    conn.save_evaluation(
        stock_id,
        &eval_response.decision,
        eval_response.score,
        &serde_json::to_string(&eval_response.analysis)?,
        Some(&ta_json),
        spec_hash_val,
        Some(llm_backend_name),
    )
    .await?;

    info!(ticker = %ticker, "Eval complete");
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
