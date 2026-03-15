use anyhow::Result;
use tracing::warn;

use crate::db::DbClient;

#[derive(Debug)]
pub struct CircuitBreakerResult {
    pub safe: bool,
    pub reasons: Vec<String>,
}

pub async fn check(conn: &dyn DbClient) -> Result<CircuitBreakerResult> {
    let mut reasons = Vec::new();

    let watchlist = conn.watchlist_list().await?;

    let mut crash_count = 0;
    let total = watchlist.len();

    for item in &watchlist {
        let Some(stock_id) = conn.get_stock_id(&item.ticker).await? else {
            continue;
        };

        let closes = conn.get_latest_closes(stock_id, 2).await?;
        if closes.len() < 2 {
            continue;
        }

        let current = closes[1];
        let previous = closes[0];

        if previous <= 0.0 {
            continue;
        }

        let change_pct = (current - previous) / previous;

        // Individual stock circuit breaker: >30% move
        if change_pct.abs() > 0.30 {
            reasons.push(format!(
                "{}: abnormal price movement ({:.1}%)",
                item.ticker,
                change_pct * 100.0
            ));
        }

        // Track crashes for market-wide check
        if change_pct < -0.05 {
            crash_count += 1;
        }
    }

    // Market-wide circuit breaker: >50% of stocks down >5%
    if total > 0 && crash_count as f64 / total as f64 > 0.5 {
        reasons.push(format!(
            "Market-wide decline: {}/{} stocks down >5%",
            crash_count, total
        ));
    }

    let safe = reasons.is_empty();
    if !safe {
        for reason in &reasons {
            warn!("Circuit breaker triggered: {}", reason);
        }
    }

    Ok(CircuitBreakerResult { safe, reasons })
}
