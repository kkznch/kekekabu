use anyhow::Result;
use rust_decimal::Decimal;
use serde::Serialize;
use std::str::FromStr;
use tokio_rusqlite::Connection;
use tracing::{info, warn};

use crate::circuit_breaker;
use crate::config::AppConfig;
use crate::db;
use crate::portfolio;
use crate::tachibana::TachibanaClient;
use crate::tachibana::order::map_status_code;

#[derive(Debug, Serialize)]
pub struct ExecuteResult {
    pub actions: Vec<ExecuteAction>,
    pub circuit_breaker_triggered: bool,
    pub circuit_breaker_reasons: Vec<String>,
    pub settle_results: Vec<SettleResult>,
    pub order_results: Vec<OrderResult>,
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

#[derive(Debug, Serialize)]
pub struct SettleResult {
    pub ticker: String,
    pub order_id: i64,
    pub old_status: String,
    pub new_status: String,
    pub filled_price: Option<String>,
    pub filled_quantity: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OrderResult {
    pub ticker: String,
    pub side: String,
    pub price: String,
    pub quantity: String,
    pub tachibana_order_id: Option<String>,
    pub status: String,
}

pub async fn run(conn: &Connection, config: &AppConfig, dry_run: bool) -> Result<ExecuteResult> {
    let mut settle_results = Vec::new();
    let mut order_results = Vec::new();

    // Build Tachibana client if non-dry-run
    let mut client = if !dry_run {
        let tc_config = config.tachibana.as_ref().ok_or_else(|| {
            anyhow::anyhow!(
                "[tachibana] config is required for non-dry-run execute. \
                 Set it in ~/.config/kabu/config.toml or use TACHIBANA_* env vars."
            )
        })?;
        Some(TachibanaClient::new(tc_config))
    } else {
        None
    };

    // ── Phase 1: Settle — check previous pending orders ──
    let pending_orders = db::list_pending_orders(conn).await?;
    if !pending_orders.is_empty() {
        if let Some(ref mut client) = client {
            info!(count = pending_orders.len(), "Settling pending orders");
            client.ensure_logged_in().await?;

            for order in &pending_orders {
                let Some(ref tachibana_id) = order.tachibana_order_id else {
                    warn!(
                        order_id = order.id,
                        "Pending order has no tachibana_order_id, skipping settle"
                    );
                    continue;
                };

                match client.query_order(tachibana_id).await {
                    Ok(detail) => {
                        let new_status = map_status_code(&detail.status_code);
                        if new_status != "pending" && new_status != order.status {
                            let filled_at = if new_status == "filled" || new_status == "partial" {
                                Some(chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string())
                            } else {
                                None
                            };

                            db::update_order_status(
                                conn,
                                order.id,
                                new_status,
                                None,
                                detail.filled_price.as_deref(),
                                detail.filled_quantity.as_deref(),
                                filled_at.as_deref(),
                            )
                            .await?;

                            // If filled or partially filled, record in portfolio
                            if new_status == "filled" || new_status == "partial" {
                                record_fill(
                                    conn,
                                    &order.ticker,
                                    &order.side,
                                    detail.filled_price.as_deref(),
                                    detail.filled_quantity.as_deref(),
                                )
                                .await?;
                            }

                            settle_results.push(SettleResult {
                                ticker: order.ticker.clone(),
                                order_id: order.id,
                                old_status: order.status.clone(),
                                new_status: new_status.to_string(),
                                filled_price: detail.filled_price,
                                filled_quantity: detail.filled_quantity,
                            });

                            info!(
                                ticker = %order.ticker,
                                order_id = order.id,
                                new_status,
                                "Order settled"
                            );
                        }
                    }
                    Err(e) => {
                        warn!(
                            ticker = %order.ticker,
                            order_id = order.id,
                            error = %e,
                            "Failed to query order for settle, will retry next run"
                        );
                    }
                }
            }
        } else {
            info!(
                count = pending_orders.len(),
                "Pending orders exist but skipping settle in dry-run mode"
            );
        }
    }

    // ── Phase 2: Circuit breaker check ──
    let cb = circuit_breaker::check(conn).await?;
    if !cb.safe {
        warn!("Circuit breaker triggered! Aborting execute.");
        if let Some(ref mut client) = client {
            let _ = client.logout().await;
        }
        return Ok(ExecuteResult {
            actions: vec![],
            circuit_breaker_triggered: true,
            circuit_breaker_reasons: cb.reasons,
            settle_results,
            order_results,
        });
    }

    // ── Phase 3: Get today's evaluations and generate signals ──
    let evals = db::get_latest_evaluations_for_today(conn).await?;
    if evals.is_empty() {
        info!("No evaluations for today. Nothing to execute.");
        if let Some(ref mut client) = client {
            let _ = client.logout().await;
        }
        return Ok(ExecuteResult {
            actions: vec![],
            circuit_breaker_triggered: false,
            circuit_breaker_reasons: vec![],
            settle_results,
            order_results,
        });
    }

    let positions = portfolio::list_positions(conn).await?;
    let held_tickers: std::collections::HashSet<String> =
        positions.iter().map(|p| p.ticker.clone()).collect();

    let mut actions = Vec::new();
    let mut signals: Vec<Signal> = Vec::new();

    for eval in &evals {
        let (action_type, detail, signal) = match eval.decision.as_str() {
            "Buy" if eval.score >= 70 => {
                // Idempotency check
                if db::order_exists_for_evaluation(conn, eval.id, "buy").await? {
                    info!(
                        ticker = %eval.ticker,
                        eval_id = eval.id,
                        "Buy order already exists for this evaluation, skipping"
                    );
                    (
                        "skip_duplicate",
                        format!(
                            "Buy order already placed for {} (eval_id: {})",
                            eval.ticker, eval.id
                        ),
                        None,
                    )
                } else if dry_run {
                    (
                        "buy_signal",
                        format!(
                            "[DRY RUN] Would place buy order for {} (score: {})",
                            eval.ticker, eval.score
                        ),
                        None,
                    )
                } else {
                    (
                        "buy_signal",
                        format!("Buy signal for {} (score: {})", eval.ticker, eval.score),
                        Some(Signal {
                            ticker: eval.ticker.clone(),
                            side: "buy".to_string(),
                            eval_id: eval.id,
                        }),
                    )
                }
            }
            "Buy" => (
                "hold",
                format!(
                    "Buy signal for {} but score too low ({} < 70), skipping",
                    eval.ticker, eval.score
                ),
                None,
            ),
            "Sell" if held_tickers.contains(&eval.ticker) => {
                if db::order_exists_for_evaluation(conn, eval.id, "sell").await? {
                    info!(
                        ticker = %eval.ticker,
                        eval_id = eval.id,
                        "Sell order already exists for this evaluation, skipping"
                    );
                    (
                        "skip_duplicate",
                        format!(
                            "Sell order already placed for {} (eval_id: {})",
                            eval.ticker, eval.id
                        ),
                        None,
                    )
                } else if dry_run {
                    (
                        "sell_signal",
                        format!(
                            "[DRY RUN] Would place sell order for {} (score: {})",
                            eval.ticker, eval.score
                        ),
                        None,
                    )
                } else {
                    (
                        "sell_signal",
                        format!("Sell signal for {} (score: {})", eval.ticker, eval.score),
                        Some(Signal {
                            ticker: eval.ticker.clone(),
                            side: "sell".to_string(),
                            eval_id: eval.id,
                        }),
                    )
                }
            }
            "Sell" => (
                "hold",
                format!(
                    "Sell signal for {} but no position held, skipping",
                    eval.ticker
                ),
                None,
            ),
            "Avoid" if eval.score <= 30 => (
                "review",
                format!(
                    "Avoid signal for {} (score: {}). Review existing positions.",
                    eval.ticker, eval.score
                ),
                None,
            ),
            _ => (
                "hold",
                format!("Hold for {} (score: {})", eval.ticker, eval.score),
                None,
            ),
        };

        actions.push(ExecuteAction {
            ticker: eval.ticker.clone(),
            name: eval.name.clone(),
            decision: eval.decision.clone(),
            score: eval.score,
            action: action_type.to_string(),
            detail,
        });

        if let Some(sig) = signal {
            signals.push(sig);
        }
    }

    // ── Phase 4: Place orders (non-dry-run only) ──
    let mut new_tachibana_order_ids: Vec<String> = Vec::new();

    if !dry_run && !signals.is_empty() {
        let client = client.as_mut().unwrap();
        client.ensure_logged_in().await?;

        for sig in &signals {
            let stock_id = db::get_stock_id(conn, &sig.ticker).await?.unwrap_or(0);
            if stock_id == 0 {
                warn!(ticker = %sig.ticker, "Stock not found in DB, skipping order");
                continue;
            }

            // Use latest close price as limit price
            let last_close = db::get_latest_close(conn, stock_id).await?.unwrap_or(0.0);
            if last_close <= 0.0 {
                warn!(ticker = %sig.ticker, "No price data available, skipping order");
                continue;
            }
            let price_str = format!("{:.0}", last_close);

            // For sell: use position quantity. For buy: 100 shares (単元株)
            let quantity = if sig.side == "sell" {
                positions
                    .iter()
                    .find(|p| p.ticker == sig.ticker)
                    .map(|p| p.quantity.to_string())
                    .unwrap_or_else(|| "100".to_string())
            } else {
                "100".to_string()
            };

            // Idempotent request_id
            let today = chrono::Local::now().format("%Y-%m-%d").to_string();
            let request_id = format!("{}-{}-{}-{}", today, sig.ticker, sig.side, sig.eval_id);

            // Save order to DB first (idempotent)
            let order_id = db::save_order(
                conn,
                stock_id,
                &sig.side,
                "limit",
                &price_str,
                &quantity,
                &request_id,
                Some(sig.eval_id),
            )
            .await?;

            // Place order via Tachibana API
            match client
                .place_order(&sig.side, &sig.ticker, &price_str, &quantity)
                .await
            {
                Ok(result) => {
                    info!(
                        ticker = %sig.ticker,
                        side = %sig.side,
                        price = %price_str,
                        quantity = %quantity,
                        tachibana_order_id = %result.order_number,
                        "Order placed successfully"
                    );

                    db::update_order_status(
                        conn,
                        order_id,
                        "pending",
                        Some(&result.order_number),
                        None,
                        None,
                        None,
                    )
                    .await?;

                    new_tachibana_order_ids.push(result.order_number.clone());

                    order_results.push(OrderResult {
                        ticker: sig.ticker.clone(),
                        side: sig.side.clone(),
                        price: price_str,
                        quantity,
                        tachibana_order_id: Some(result.order_number),
                        status: "pending".to_string(),
                    });
                }
                Err(e) => {
                    warn!(ticker = %sig.ticker, error = %e, "Failed to place order");
                    db::update_order_status(conn, order_id, "rejected", None, None, None, None)
                        .await?;

                    order_results.push(OrderResult {
                        ticker: sig.ticker.clone(),
                        side: sig.side.clone(),
                        price: price_str,
                        quantity,
                        tachibana_order_id: None,
                        status: format!("rejected: {}", e),
                    });
                }
            }
        }
    }

    // ── Phase 5: Short WebSocket fill wait ──
    if !dry_run
        && !new_tachibana_order_ids.is_empty()
        && let Some(ref client) = client
    {
        info!(
            order_count = new_tachibana_order_ids.len(),
            "Waiting for fill notifications"
        );

        match client.wait_for_fills(&new_tachibana_order_ids).await {
            Ok(fills) => {
                // Fetch pending orders once for all fills
                let pending = db::list_pending_orders(conn).await?;

                for fill in fills {
                    // Update matching order result
                    if let Some(or) = order_results
                        .iter_mut()
                        .find(|o| o.tachibana_order_id.as_deref() == Some(&fill.order_number))
                    {
                        or.status = "filled".to_string();
                    }

                    // Find matching pending order from pre-fetched list
                    if let Some(order) = pending
                        .iter()
                        .find(|o| o.tachibana_order_id.as_deref() == Some(&fill.order_number))
                    {
                        let filled_at =
                            chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

                        db::update_order_status(
                            conn,
                            order.id,
                            "filled",
                            None,
                            Some(&fill.filled_price),
                            Some(&fill.filled_quantity),
                            Some(&filled_at),
                        )
                        .await?;

                        record_fill(
                            conn,
                            &order.ticker,
                            &order.side,
                            Some(&fill.filled_price),
                            Some(&fill.filled_quantity),
                        )
                        .await?;

                        info!(
                            ticker = %order.ticker,
                            price = %fill.filled_price,
                            quantity = %fill.filled_quantity,
                            "Fill recorded in portfolio"
                        );
                    }
                }
            }
            Err(e) => {
                warn!(error = %e, "Error during WebSocket fill wait");
            }
        }
    }

    // ── Phase 6: Logout ──
    if let Some(ref mut client) = client {
        let _ = client.logout().await;
    }

    Ok(ExecuteResult {
        actions,
        circuit_breaker_triggered: false,
        circuit_breaker_reasons: vec![],
        settle_results,
        order_results,
    })
}

/// Record a fill in the portfolio (buy or sell).
async fn record_fill(
    conn: &Connection,
    ticker: &str,
    side: &str,
    filled_price: Option<&str>,
    filled_quantity: Option<&str>,
) -> Result<()> {
    if let (Some(fp), Some(fq)) = (filled_price, filled_quantity) {
        let price = Decimal::from_str(fp).unwrap_or_default();
        let qty = Decimal::from_str(fq).unwrap_or_default();

        match side {
            "buy" => portfolio::buy(conn, ticker, qty, price, Some("tachibana-fill")).await?,
            "sell" => portfolio::sell(conn, ticker, qty, price, Some("tachibana-fill")).await?,
            _ => {}
        }
    }
    Ok(())
}

/// Internal signal for order placement.
struct Signal {
    ticker: String,
    side: String,
    eval_id: i64,
}
