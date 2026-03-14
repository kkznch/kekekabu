use anyhow::Result;
use rust_decimal::Decimal;
use serde::Serialize;
use std::str::FromStr;
use tracing::{info, warn};

use crate::circuit_breaker;
use crate::config::AppConfig;
use crate::db::{DbClient, FillParams};
use crate::spec::InvestmentSpec;
use crate::tachibana::TachibanaClient;
use crate::tachibana::order::map_status_code;

#[derive(Debug, Serialize)]
pub struct ExecuteResult {
    pub actions: Vec<ExecuteAction>,
    pub circuit_breaker_triggered: bool,
    pub circuit_breaker_reasons: Vec<String>,
    pub hard_stop_loss_actions: Vec<HardStopLossAction>,
    pub settle_results: Vec<SettleResult>,
    pub order_results: Vec<OrderResult>,
}

#[derive(Debug, Serialize)]
pub struct HardStopLossAction {
    pub ticker: String,
    pub name: String,
    pub avg_cost: String,
    pub current_price: String,
    pub loss_pct: String,
    pub threshold: String,
    pub action: String,
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

pub async fn run(
    conn: &dyn DbClient,
    config: &AppConfig,
    spec: &InvestmentSpec,
    dry_run: bool,
) -> Result<ExecuteResult> {
    let mut settle_results = Vec::new();
    let mut order_results = Vec::new();
    let mut hard_stop_loss_actions = Vec::new();

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
    let pending_orders = conn.list_pending_orders().await?;
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

                            if new_status == "filled" || new_status == "partial" {
                                conn.update_order_and_record_fill(FillParams {
                                    order_id: order.id,
                                    status: new_status.to_string(),
                                    tachibana_order_id: None,
                                    filled_price: detail.filled_price.clone(),
                                    filled_quantity: detail.filled_quantity.clone(),
                                    filled_at,
                                    ticker: order.ticker.clone(),
                                    side: order.side.clone(),
                                })
                                .await?;
                            } else {
                                conn.update_order_status(
                                    order.id,
                                    new_status,
                                    None,
                                    detail.filled_price.as_deref(),
                                    detail.filled_quantity.as_deref(),
                                    None,
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
            hard_stop_loss_actions: vec![],
            settle_results,
            order_results,
        });
    }

    // ── Phase 3: Hard stop-loss check (rule-based, independent of LLM) ──
    let positions = conn.list_positions().await?;
    let stop_loss_threshold = spec.execution_stop_loss();

    if let Some(threshold) = stop_loss_threshold {
        for pos in &positions {
            let loss_pct = match pos.unrealized_pnl_pct {
                Some(pct) => pct,
                None => continue,
            };

            // threshold is e.g. -0.07, loss_pct is e.g. -8.5 (percentage)
            // Convert threshold to percentage for comparison: -0.07 → -7.0
            let threshold_pct =
                Decimal::from_str(&format!("{}", threshold * 100.0)).unwrap_or_default();

            if loss_pct <= threshold_pct {
                warn!(
                    ticker = %pos.ticker,
                    loss_pct = %loss_pct,
                    threshold = %threshold_pct,
                    "HARD STOP-LOSS triggered — forced sell"
                );

                let action_str = if dry_run {
                    format!(
                        "[DRY RUN] Would force-sell {} (loss: {}%, threshold: {}%)",
                        pos.ticker, loss_pct, threshold_pct
                    )
                } else {
                    format!(
                        "Force-sell {} (loss: {}%, threshold: {}%)",
                        pos.ticker, loss_pct, threshold_pct
                    )
                };

                hard_stop_loss_actions.push(HardStopLossAction {
                    ticker: pos.ticker.clone(),
                    name: pos.name.clone(),
                    avg_cost: pos.avg_cost.to_string(),
                    current_price: pos.current_price.map(|p| p.to_string()).unwrap_or_default(),
                    loss_pct: loss_pct.to_string(),
                    threshold: threshold_pct.to_string(),
                    action: action_str,
                });

                // Generate forced sell signal (non-dry-run only, handled in Phase 5)
                // Signals are injected below after eval signals
            }
        }
    }

    // ── Phase 4: Get today's evaluations and generate signals ──
    let evals = conn.get_latest_evaluations_for_today().await?;
    if evals.is_empty() && hard_stop_loss_actions.is_empty() {
        info!("No evaluations for today and no stop-loss triggers. Nothing to execute.");
        if let Some(ref mut client) = client {
            let _ = client.logout().await;
        }
        return Ok(ExecuteResult {
            actions: vec![],
            circuit_breaker_triggered: false,
            circuit_breaker_reasons: vec![],
            hard_stop_loss_actions,
            settle_results,
            order_results,
        });
    }

    let held_tickers: std::collections::HashSet<String> =
        positions.iter().map(|p| p.ticker.clone()).collect();

    // Pre-compute stop-loss tickers for blocking conflicting buy signals
    let stop_loss_tickers: std::collections::HashSet<String> = hard_stop_loss_actions
        .iter()
        .map(|a| a.ticker.clone())
        .collect();

    let mut actions = Vec::new();
    let mut signals: Vec<Signal> = Vec::new();

    for eval in &evals {
        let (action_type, detail, signal) = match eval.decision.as_str() {
            // Block buy signals for tickers being force-sold by stop-loss
            "Buy" if stop_loss_tickers.contains(&eval.ticker) => (
                "blocked_by_stop_loss",
                format!(
                    "Buy signal for {} blocked — hard stop-loss active",
                    eval.ticker
                ),
                None,
            ),
            "Buy" if eval.score >= 70 => {
                // Idempotency check
                if conn.order_exists_for_evaluation(eval.id, "buy").await? {
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
                            force_market: false,
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
                if conn.order_exists_for_evaluation(eval.id, "sell").await? {
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
                            force_market: false,
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

    // ── Phase 5: Inject hard stop-loss forced sell signals ──
    // Stop-loss sells are eval-independent (eval_id = 0) and use market order semantics
    if !dry_run {
        for sl in &hard_stop_loss_actions {
            // Skip if an eval-based sell signal already exists for this ticker
            if signals
                .iter()
                .any(|s| s.ticker == sl.ticker && s.side == "sell")
            {
                info!(
                    ticker = %sl.ticker,
                    "Eval already generated sell signal, skipping stop-loss forced sell"
                );
                continue;
            }
            signals.push(Signal {
                ticker: sl.ticker.clone(),
                side: "sell".to_string(),
                eval_id: 0, // 0 indicates rule-based, not eval-based
                force_market: true,
            });
        }
    }

    // ── Phase 6: Place orders (non-dry-run only) ──
    let max_position_size = spec.execution_max_position_size();
    let initial_cash = spec.budget_initial_cash();
    let mut new_tachibana_order_ids: Vec<String> = Vec::new();

    if !dry_run && !signals.is_empty() {
        let client = client.as_mut().unwrap();
        client.ensure_logged_in().await?;

        for sig in &signals {
            let stock_id = conn.get_stock_id(&sig.ticker).await?.unwrap_or(0);
            if stock_id == 0 {
                warn!(ticker = %sig.ticker, "Stock not found in DB, skipping order");
                continue;
            }

            // Use latest close price as limit price
            let last_close = conn.get_latest_close(stock_id).await?.unwrap_or(0.0);
            if last_close <= 0.0 {
                warn!(ticker = %sig.ticker, "No price data available, skipping order");
                continue;
            }

            // Stop-loss sells use market order (price "0" signals market order to Tachibana)
            let use_market = sig.force_market;
            let price_str = if use_market {
                "0".to_string()
            } else {
                format!("{:.0}", last_close)
            };

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

            // Max position size check for buy orders
            if sig.side == "buy"
                && let (Some(max_size), Some(cash)) = (max_position_size, initial_cash)
            {
                let order_value = last_close * quantity.parse::<f64>().unwrap_or(100.0);
                let max_allowed = cash * max_size;
                if order_value > max_allowed {
                    warn!(
                        ticker = %sig.ticker,
                        order_value,
                        max_allowed,
                        max_position_size = max_size,
                        "Buy order exceeds max position size, rejecting"
                    );
                    order_results.push(OrderResult {
                        ticker: sig.ticker.clone(),
                        side: sig.side.clone(),
                        price: price_str,
                        quantity,
                        tachibana_order_id: None,
                        status: format!(
                            "rejected: exceeds max position size ({:.0} > {:.0})",
                            order_value, max_allowed
                        ),
                    });
                    continue;
                }
            }

            // Idempotent request_id
            let today = chrono::Local::now().format("%Y-%m-%d").to_string();
            let request_id = if sig.eval_id == 0 {
                // Stop-loss: use special prefix to avoid collision with eval-based orders
                format!("{}-{}-{}-stoploss", today, sig.ticker, sig.side)
            } else {
                format!("{}-{}-{}-{}", today, sig.ticker, sig.side, sig.eval_id)
            };

            let order_type = if use_market { "market" } else { "limit" };

            // Save order to DB first (idempotent)
            let eval_id_opt = if sig.eval_id == 0 {
                None
            } else {
                Some(sig.eval_id)
            };
            let order_id = conn
                .save_order(
                    stock_id,
                    &sig.side,
                    order_type,
                    &price_str,
                    &quantity,
                    &request_id,
                    eval_id_opt,
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

                    conn.update_order_status(
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
                    conn.update_order_status(order_id, "rejected", None, None, None, None)
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

    // ── Phase 7: Short WebSocket fill wait ──
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
                let pending = conn.list_pending_orders().await?;

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

                        conn.update_order_and_record_fill(FillParams {
                            order_id: order.id,
                            status: "filled".to_string(),
                            tachibana_order_id: None,
                            filled_price: Some(fill.filled_price.clone()),
                            filled_quantity: Some(fill.filled_quantity.clone()),
                            filled_at: Some(filled_at),
                            ticker: order.ticker.clone(),
                            side: order.side.clone(),
                        })
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

    // ── Phase 8: Logout ──
    if let Some(ref mut client) = client {
        let _ = client.logout().await;
    }

    Ok(ExecuteResult {
        actions,
        circuit_breaker_triggered: false,
        circuit_breaker_reasons: vec![],
        hard_stop_loss_actions,
        settle_results,
        order_results,
    })
}

/// Internal signal for order placement.
struct Signal {
    ticker: String,
    side: String,
    eval_id: i64,
    /// Use market order (for hard stop-loss sells).
    force_market: bool,
}
