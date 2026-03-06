use anyhow::Result;
use tokio_rusqlite::Connection;
use tracing::warn;

use crate::db;

#[derive(Debug)]
pub struct CircuitBreakerResult {
    pub safe: bool,
    pub reasons: Vec<String>,
}

pub async fn check(conn: &Connection) -> Result<CircuitBreakerResult> {
    let mut reasons = Vec::new();

    // Check for abnormal price movements across watchlist
    let watchlist = db::watchlist_list(conn).await?;

    let mut crash_count = 0;
    let total = watchlist.len();

    for item in &watchlist {
        let stock_id = match db::get_stock_id(conn, &item.ticker).await? {
            Some(id) => id,
            None => continue,
        };

        let price_data = db::fetch_price_data(conn, stock_id).await?;
        if price_data.closes.len() < 2 {
            continue;
        }

        let len = price_data.closes.len();
        let current = price_data.closes[len - 1];
        let previous = price_data.closes[len - 2];

        if previous > 0.0 {
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
