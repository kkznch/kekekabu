use anyhow::{Context, Result};
use rust_decimal::Decimal;
use serde::Serialize;
use std::str::FromStr;
use tracing::{info, warn};

use crate::db::DbClient;
use crate::tachibana::BrokerClient;

#[derive(Debug, Serialize)]
pub struct SyncResult {
    pub cash_available: String,
    pub broker_position_count: usize,
    pub db_position_count: usize,
    pub mismatches: Vec<PositionMismatch>,
    pub fixed: bool,
}

#[derive(Debug, Serialize)]
pub struct PositionMismatch {
    pub ticker: String,
    pub db_quantity: String,
    pub broker_quantity: i64,
    pub kind: MismatchKind,
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MismatchKind {
    /// In DB but not on broker — DB has stale position to remove
    DbOnly,
    /// On broker but not in DB — DB is missing the position
    BrokerOnly,
    /// Quantity differs
    QuantityDiff,
}

pub async fn run(
    conn: &dyn DbClient,
    broker: &mut dyn BrokerClient,
    fix: bool,
) -> Result<SyncResult> {
    broker.ensure_logged_in().await?;

    // 1. Fetch and persist balance
    let balance = broker
        .query_balance()
        .await
        .context("query_balance failed")?;
    info!(cash_available = %balance.cash_available, "Account balance fetched");
    conn.save_balance_snapshot(&balance.cash_available).await?;

    // 2. Fetch broker positions and DB positions
    let broker_positions = broker
        .query_positions()
        .await
        .context("query_positions failed")?;
    let db_positions = conn.list_positions().await?;

    info!(
        broker_count = broker_positions.len(),
        db_count = db_positions.len(),
        "Positions fetched"
    );

    // 3. Compute mismatches
    let mut mismatches: Vec<PositionMismatch> = Vec::new();

    use std::collections::HashMap;
    let broker_map: HashMap<&str, i64> = broker_positions
        .iter()
        .map(|p| (p.ticker.as_str(), p.quantity))
        .collect();
    let db_map: HashMap<&str, &Decimal> = db_positions
        .iter()
        .map(|p| (p.ticker.as_str(), &p.quantity))
        .collect();

    for db_pos in &db_positions {
        match broker_map.get(db_pos.ticker.as_str()) {
            Some(&broker_qty) => {
                let db_qty_int: i64 = db_pos
                    .quantity
                    .to_string()
                    .parse::<f64>()
                    .map(|f| f as i64)
                    .unwrap_or(0);
                if db_qty_int != broker_qty {
                    mismatches.push(PositionMismatch {
                        ticker: db_pos.ticker.clone(),
                        db_quantity: db_pos.quantity.to_string(),
                        broker_quantity: broker_qty,
                        kind: MismatchKind::QuantityDiff,
                    });
                }
            }
            None => {
                if !db_pos.quantity.is_zero() {
                    mismatches.push(PositionMismatch {
                        ticker: db_pos.ticker.clone(),
                        db_quantity: db_pos.quantity.to_string(),
                        broker_quantity: 0,
                        kind: MismatchKind::DbOnly,
                    });
                }
            }
        }
    }

    for broker_pos in &broker_positions {
        if !db_map.contains_key(broker_pos.ticker.as_str()) && broker_pos.quantity > 0 {
            mismatches.push(PositionMismatch {
                ticker: broker_pos.ticker.clone(),
                db_quantity: "0".to_string(),
                broker_quantity: broker_pos.quantity,
                kind: MismatchKind::BrokerOnly,
            });
        }
    }

    if !mismatches.is_empty() {
        warn!(
            count = mismatches.len(),
            "Position mismatches detected between DB and broker"
        );
        for m in &mismatches {
            warn!(
                ticker = %m.ticker,
                db_quantity = %m.db_quantity,
                broker_quantity = m.broker_quantity,
                kind = ?m.kind,
                "Position mismatch"
            );
        }
    }

    // 4. Apply fix if requested
    let mut fixed = false;
    if fix && !mismatches.is_empty() {
        info!("Applying --fix to align DB with broker positions");
        for m in &mismatches {
            match m.kind {
                MismatchKind::DbOnly => {
                    conn.delete_position(&m.ticker).await?;
                    info!(ticker = %m.ticker, "Deleted stale position from DB");
                }
                MismatchKind::BrokerOnly | MismatchKind::QuantityDiff => {
                    let bp = broker_positions
                        .iter()
                        .find(|p| p.ticker == m.ticker)
                        .expect("broker position must exist for non-DbOnly mismatch");
                    let qty = Decimal::from(bp.quantity);
                    let avg_cost = Decimal::from_str(&bp.avg_cost).unwrap_or(Decimal::ZERO);
                    conn.set_position_quantity(&m.ticker, qty, avg_cost).await?;
                    info!(
                        ticker = %m.ticker,
                        quantity = bp.quantity,
                        "Updated DB position to match broker"
                    );
                }
            }
        }
        fixed = true;
    }

    let _ = broker.logout().await;

    Ok(SyncResult {
        cash_available: balance.cash_available,
        broker_position_count: broker_positions.len(),
        db_position_count: db_positions.len(),
        mismatches,
        fixed,
    })
}
