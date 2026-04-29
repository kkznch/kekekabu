use anyhow::Result;
use rust_decimal::Decimal;
use serde::Serialize;
use std::str::FromStr;
use tracing::{info, warn};

use crate::circuit_breaker;
use crate::config::AppConfig;
use crate::db::{DbClient, FillParams};
use crate::spec::InvestmentSpec;
use crate::tachibana::order::map_status_code;
use crate::tachibana::{BrokerClient, Side};

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
    mut broker: Option<&mut dyn BrokerClient>,
    dry_run: bool,
) -> Result<ExecuteResult> {
    let mut order_results = Vec::new();
    let mut hard_stop_loss_actions = Vec::new();

    // ── Phase 1: Settle — check previous pending orders ──
    let settle_results = settle_pending_orders(conn, &mut broker).await?;

    // ── Phase 2: Circuit breaker check ──
    let cb = circuit_breaker::check(conn).await?;
    if !cb.safe {
        warn!("Circuit breaker triggered! Aborting execute.");
        logout_broker(&mut broker).await;
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
    if let Some(threshold) = spec.execution_stop_loss() {
        hard_stop_loss_actions = check_hard_stop_loss(&positions, threshold, dry_run);
    }

    // ── Phase 4: Get today's evaluations and generate signals ──
    let evals = conn.get_latest_evaluations_for_today().await?;
    if evals.is_empty() && hard_stop_loss_actions.is_empty() {
        info!("No evaluations for today and no stop-loss triggers. Nothing to execute.");
        logout_broker(&mut broker).await;
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
                            side: Side::Buy,
                            eval_id: Some(eval.id),
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
                            side: Side::Sell,
                            eval_id: Some(eval.id),
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
    if !dry_run {
        for sl in &hard_stop_loss_actions {
            // Skip if an eval-based sell signal already exists for this ticker
            if signals
                .iter()
                .any(|s| s.ticker == sl.ticker && s.side == Side::Sell)
            {
                info!(
                    ticker = %sl.ticker,
                    "Eval already generated sell signal, skipping stop-loss forced sell"
                );
                continue;
            }
            signals.push(Signal {
                ticker: sl.ticker.clone(),
                side: Side::Sell,
                eval_id: None,
                force_market: true,
            });
        }
    }

    // ── Phase 6: Place orders (non-dry-run only) ──
    let max_position_size = spec.execution_max_position_size();
    // Use broker-synced cash_available for max exposure check (kabu sync required)
    let cash_available: Option<f64> = match conn.get_latest_balance().await? {
        Some(b) => b.cash_available.parse().ok(),
        None => {
            if max_position_size.is_some() {
                warn!(
                    "No broker balance synced; max_position_size check disabled. Run `kabu sync` first."
                );
            }
            None
        }
    };
    if !dry_run && !signals.is_empty() {
        let client = broker
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("BrokerClient required for non-dry-run execution"))?;
        client.ensure_logged_in().await?;

        for sig in &signals {
            let stock_id = conn.get_stock_id(&sig.ticker).await?.unwrap_or(0);
            if stock_id == 0 {
                warn!(ticker = %sig.ticker, "Stock not found in DB, skipping order");
                continue;
            }

            let last_close = conn.get_latest_close(stock_id).await?.unwrap_or(0.0);
            if last_close <= 0.0 {
                warn!(ticker = %sig.ticker, "No price data available, skipping order");
                continue;
            }

            let price_str = if sig.force_market {
                "0".to_string()
            } else {
                format!("{:.0}", last_close)
            };

            let quantity = if sig.side == Side::Sell {
                positions
                    .iter()
                    .find(|p| p.ticker == sig.ticker)
                    .map(|p| p.quantity.to_string())
                    .unwrap_or_else(|| "100".to_string())
            } else {
                "100".to_string()
            };

            // Max position size check for buy orders (uses broker-synced cash_available)
            if sig.side == Side::Buy
                && let (Some(max_size), Some(cash)) = (max_position_size, cash_available)
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
                        side: sig.side.as_str().to_string(),
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

            let today = chrono::Local::now().format("%Y-%m-%d").to_string();
            let request_id = match sig.eval_id {
                Some(id) => format!("{}-{}-{}-{}", today, sig.ticker, sig.side, id),
                None => format!("{}-{}-{}-stoploss", today, sig.ticker, sig.side),
            };

            let order_type = if sig.force_market { "market" } else { "limit" };

            let order_id = conn
                .save_order(
                    stock_id,
                    sig.side.as_str(),
                    order_type,
                    &price_str,
                    &quantity,
                    &request_id,
                    sig.eval_id,
                )
                .await?;

            match client
                .place_order(
                    sig.side,
                    &sig.ticker,
                    &price_str,
                    &quantity,
                    config
                        .tachibana
                        .as_ref()
                        .and_then(|t| t.second_password.as_deref())
                        .unwrap_or(""),
                )
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

                    order_results.push(OrderResult {
                        ticker: sig.ticker.clone(),
                        side: sig.side.as_str().to_string(),
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
                        side: sig.side.as_str().to_string(),
                        price: price_str,
                        quantity,
                        tachibana_order_id: None,
                        status: format!("rejected: {}", e),
                    });
                }
            }
        }
    }

    // ── Phase 7: Logout ──
    logout_broker(&mut broker).await;

    Ok(ExecuteResult {
        actions,
        circuit_breaker_triggered: false,
        circuit_breaker_reasons: vec![],
        hard_stop_loss_actions,
        settle_results,
        order_results,
    })
}

// ─── Helper functions ────────────────────────────────────────────────

/// Internal signal for order placement.
struct Signal {
    ticker: String,
    side: Side,
    eval_id: Option<i64>,
    /// Use market order (for hard stop-loss sells).
    force_market: bool,
}

async fn logout_broker(broker: &mut Option<&mut dyn BrokerClient>) {
    if let Some(client) = broker {
        let _ = client.logout().await;
    }
}

async fn settle_pending_orders(
    conn: &dyn DbClient,
    broker: &mut Option<&mut dyn BrokerClient>,
) -> Result<Vec<SettleResult>> {
    let pending_orders = conn.list_pending_orders().await?;
    if pending_orders.is_empty() {
        return Ok(vec![]);
    }

    let Some(client) = broker else {
        info!(
            count = pending_orders.len(),
            "Pending orders exist but skipping settle in dry-run mode"
        );
        return Ok(vec![]);
    };

    info!(count = pending_orders.len(), "Settling pending orders");
    client.ensure_logged_in().await?;

    let mut results = Vec::new();
    for order in &pending_orders {
        let Some(ref tachibana_id) = order.tachibana_order_id else {
            warn!(
                order_id = order.id,
                "Pending order has no tachibana_order_id, skipping settle"
            );
            continue;
        };

        let detail = match client.query_order(tachibana_id).await {
            Ok(d) => d,
            Err(e) => {
                warn!(
                    ticker = %order.ticker,
                    order_id = order.id,
                    error = %e,
                    "Failed to query order for settle, will retry next run"
                );
                continue;
            }
        };

        let new_status = map_status_code(&detail.status_code);
        if new_status == "pending" || new_status == order.status {
            continue;
        }

        let is_fill = new_status == "filled" || new_status == "partial";
        if is_fill {
            let filled_at = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            conn.update_order_and_record_fill(FillParams {
                order_id: order.id,
                status: new_status.to_string(),
                tachibana_order_id: None,
                filled_price: detail.filled_price.clone(),
                filled_quantity: detail.filled_quantity.clone(),
                filled_at: Some(filled_at),
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

        results.push(SettleResult {
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

    Ok(results)
}

fn check_hard_stop_loss(
    positions: &[crate::portfolio::PositionView],
    threshold: f64,
    dry_run: bool,
) -> Vec<HardStopLossAction> {
    let threshold_pct = Decimal::from_str(&format!("{}", threshold * 100.0)).unwrap_or_default();
    let mut actions = Vec::new();

    for pos in positions {
        let Some(loss_pct) = pos.unrealized_pnl_pct else {
            continue;
        };

        if loss_pct > threshold_pct {
            continue;
        }

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

        actions.push(HardStopLossAction {
            ticker: pos.ticker.clone(),
            name: pos.name.clone(),
            avg_cost: pos.avg_cost.to_string(),
            current_price: pos.current_price.map(|p| p.to_string()).unwrap_or_default(),
            loss_pct: loss_pct.to_string(),
            threshold: threshold_pct.to_string(),
            action: action_str,
        });
    }

    actions
}
