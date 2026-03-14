use anyhow::{Context, Result};
use rust_decimal::Decimal;
use serde::Serialize;
use std::str::FromStr;
use tokio_rusqlite::Connection;

#[derive(Debug, Clone, Serialize)]
pub struct PositionView {
    pub ticker: String,
    pub name: String,
    pub quantity: Decimal,
    pub avg_cost: Decimal,
    pub total_invested: Decimal,
    pub current_price: Option<Decimal>,
    pub current_value: Option<Decimal>,
    pub unrealized_pnl: Option<Decimal>,
    pub unrealized_pnl_pct: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PortfolioSummary {
    pub total_invested: Decimal,
    pub total_current_value: Decimal,
    pub total_unrealized_pnl: Decimal,
    pub total_unrealized_pnl_pct: Option<Decimal>,
    pub position_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct TradeRecord {
    pub ticker: String,
    pub side: String,
    pub date: String,
    pub price: Decimal,
    pub quantity: Decimal,
    pub pnl: Option<Decimal>,
}

#[allow(dead_code)] // Used by tests; will be called from execute when Tachibana API is integrated
pub async fn buy(
    conn: &Connection,
    ticker: &str,
    quantity: Decimal,
    price: Decimal,
    strategy: Option<&str>,
) -> Result<()> {
    let ticker = ticker.to_string();
    let ticker_ctx = ticker.clone();
    let quantity_str = quantity.to_string();
    let price_str = price.to_string();
    let strategy = strategy.map(|s| s.to_string());

    conn.call(move |conn| {
        let tx = conn.transaction()?;

        tx.execute(
            "INSERT OR IGNORE INTO stocks (ticker, name, market) VALUES (?1, ?1, 'jp')",
            rusqlite::params![ticker],
        )?;
        let stock_id: i64 = tx.query_row(
            "SELECT id FROM stocks WHERE ticker = ?1",
            [&ticker],
            |row| row.get(0),
        )?;

        let existing: Option<(i64, String, String)> = tx
            .query_row(
                "SELECT id, quantity, avg_cost FROM portfolio_positions
                 WHERE stock_id = ?1 AND is_active = 1",
                [stock_id],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                },
            )
            .ok();

        if let Some((pos_id, old_qty_str, old_avg_str)) = existing {
            let old_qty = Decimal::from_str(&old_qty_str).unwrap_or_default();
            let old_avg = Decimal::from_str(&old_avg_str).unwrap_or_default();
            let buy_qty = Decimal::from_str(&quantity_str).unwrap_or_default();
            let buy_price = Decimal::from_str(&price_str).unwrap_or_default();

            let new_qty = old_qty + buy_qty;
            let new_avg = (old_avg * old_qty + buy_price * buy_qty) / new_qty;
            let new_invested = new_avg * new_qty;

            tx.execute(
                "UPDATE portfolio_positions
                 SET quantity = ?1, avg_cost = ?2, total_invested = ?3, updated_at = datetime('now')
                 WHERE id = ?4",
                rusqlite::params![
                    new_qty.to_string(),
                    new_avg.to_string(),
                    new_invested.to_string(),
                    pos_id
                ],
            )?;
        } else {
            let buy_qty = Decimal::from_str(&quantity_str).unwrap_or_default();
            let buy_price = Decimal::from_str(&price_str).unwrap_or_default();
            let invested = buy_qty * buy_price;

            // Check for inactive position (previously sold) and reactivate it
            let inactive_id: Option<i64> = tx
                .query_row(
                    "SELECT id FROM portfolio_positions WHERE stock_id = ?1 AND is_active = 0",
                    [stock_id],
                    |row| row.get(0),
                )
                .ok();

            if let Some(pos_id) = inactive_id {
                tx.execute(
                    "UPDATE portfolio_positions
                     SET quantity = ?1, avg_cost = ?2, total_invested = ?3, is_active = 1, updated_at = datetime('now')
                     WHERE id = ?4",
                    rusqlite::params![
                        quantity_str,
                        price_str,
                        invested.to_string(),
                        pos_id
                    ],
                )?;
            } else {
                tx.execute(
                    "INSERT INTO portfolio_positions (stock_id, quantity, avg_cost, total_invested)
                     VALUES (?1, ?2, ?3, ?4)",
                    rusqlite::params![stock_id, quantity_str, price_str, invested.to_string()],
                )?;
            }
        }

        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        tx.execute(
            "INSERT INTO trades (stock_id, side, date, price, quantity, strategy)
             VALUES (?1, 'buy', ?2, ?3, ?4, ?5)",
            rusqlite::params![stock_id, today, price_str, quantity_str, strategy],
        )?;

        tx.commit()?;
        Ok::<(), rusqlite::Error>(())
    })
    .await
    .with_context(|| format!("Failed to record buy for {}", ticker_ctx))
}

#[allow(dead_code)] // Used by tests; will be called from execute when Tachibana API is integrated
pub async fn sell(
    conn: &Connection,
    ticker: &str,
    quantity: Decimal,
    price: Decimal,
    strategy: Option<&str>,
) -> Result<()> {
    let ticker = ticker.to_string();
    let ticker_ctx = ticker.clone();
    let quantity_str = quantity.to_string();
    let price_str = price.to_string();
    let strategy = strategy.map(|s| s.to_string());

    conn.call(move |conn| {
        let tx = conn.transaction()?;

        let stock_id: i64 = tx.query_row(
            "SELECT id FROM stocks WHERE ticker = ?1",
            [&ticker],
            |row| row.get(0),
        )?;

        let (pos_id, old_qty_str, old_avg_str): (i64, String, String) = tx.query_row(
            "SELECT id, quantity, avg_cost FROM portfolio_positions
             WHERE stock_id = ?1 AND is_active = 1",
            [stock_id],
            |row| Ok((row.get(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?)),
        )?;

        let old_qty = Decimal::from_str(&old_qty_str).unwrap_or_default();
        let old_avg = Decimal::from_str(&old_avg_str).unwrap_or_default();
        let sell_qty = Decimal::from_str(&quantity_str).unwrap_or_default();
        let sell_price = Decimal::from_str(&price_str).unwrap_or_default();

        if sell_qty > old_qty {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }

        let pnl = (sell_price - old_avg) * sell_qty;
        let new_qty = old_qty - sell_qty;

        if new_qty.is_zero() {
            tx.execute(
                "UPDATE portfolio_positions
                 SET quantity = '0', total_invested = '0', is_active = 0, updated_at = datetime('now')
                 WHERE id = ?1",
                [pos_id],
            )?;

            // Auto-remove from watchlist when position is fully closed
            tx.execute(
                "DELETE FROM watchlist WHERE stock_id = ?1",
                [stock_id],
            )?;
            tx.execute(
                "INSERT INTO watchlist_events (ticker, action, reason) VALUES (?1, 'auto-removed-on-sell', 'Position closed')",
                rusqlite::params![ticker],
            )?;
        } else {
            let new_invested = old_avg * new_qty;
            tx.execute(
                "UPDATE portfolio_positions
                 SET quantity = ?1, total_invested = ?2, updated_at = datetime('now')
                 WHERE id = ?3",
                rusqlite::params![new_qty.to_string(), new_invested.to_string(), pos_id],
            )?;
        }

        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        tx.execute(
            "INSERT INTO trades (stock_id, side, date, price, quantity, pnl, strategy)
             VALUES (?1, 'sell', ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![stock_id, today, price_str, quantity_str, pnl.to_string(), strategy],
        )?;

        tx.commit()?;
        Ok::<(), rusqlite::Error>(())
    })
    .await
    .with_context(|| format!("Failed to record sell for {}", ticker_ctx))
}

pub async fn list_positions(conn: &Connection) -> Result<Vec<PositionView>> {
    conn.call(|conn| {
        let mut stmt = conn.prepare(
            "SELECT s.ticker, s.name, pp.quantity, pp.avg_cost, pp.total_invested,
                    (SELECT p.close FROM prices p WHERE p.stock_id = pp.stock_id ORDER BY p.date DESC LIMIT 1)
             FROM portfolio_positions pp
             JOIN stocks s ON s.id = pp.stock_id
             WHERE pp.is_active = 1
             ORDER BY pp.updated_at DESC",
        )?;

        let rows = stmt
            .query_map([], |row| {
                let qty_str: String = row.get(2)?;
                let avg_str: String = row.get(3)?;
                let invested_str: String = row.get(4)?;
                let latest_str: Option<String> = row.get(5)?;

                let quantity = Decimal::from_str(&qty_str).unwrap_or_default();
                let avg_cost = Decimal::from_str(&avg_str).unwrap_or_default();
                let total_invested = Decimal::from_str(&invested_str).unwrap_or_default();
                let current_price = latest_str.as_deref().and_then(|s| Decimal::from_str(s).ok());
                let current_value = current_price.map(|p| p * quantity);
                let unrealized_pnl = current_value.map(|cv| cv - total_invested);
                let unrealized_pnl_pct = unrealized_pnl.and_then(|pnl| {
                    if total_invested.is_zero() { None }
                    else { Some(pnl / total_invested * Decimal::from(100)) }
                });

                Ok(PositionView {
                    ticker: row.get(0)?,
                    name: row.get(1)?,
                    quantity,
                    avg_cost,
                    total_invested,
                    current_price,
                    current_value,
                    unrealized_pnl,
                    unrealized_pnl_pct,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok::<Vec<PositionView>, rusqlite::Error>(rows)
    })
    .await
    .context("Failed to list positions")
}

pub async fn summary(conn: &Connection) -> Result<PortfolioSummary> {
    let positions = list_positions(conn).await?;
    let position_count = positions.len();
    let total_invested = positions.iter().map(|p| p.total_invested).sum::<Decimal>();
    let total_current_value = positions
        .iter()
        .filter_map(|p| p.current_value)
        .sum::<Decimal>();
    let total_unrealized_pnl = positions
        .iter()
        .filter_map(|p| p.unrealized_pnl)
        .sum::<Decimal>();
    let total_unrealized_pnl_pct = if total_invested.is_zero() {
        None
    } else {
        Some(total_unrealized_pnl / total_invested * Decimal::from(100))
    };

    Ok(PortfolioSummary {
        total_invested,
        total_current_value,
        total_unrealized_pnl,
        total_unrealized_pnl_pct,
        position_count,
    })
}

pub async fn trade_history(conn: &Connection, limit: i64) -> Result<Vec<TradeRecord>> {
    conn.call(move |conn| {
        let mut stmt = conn.prepare(
            "SELECT s.ticker, t.side, t.date, t.price, t.quantity, t.pnl
             FROM trades t
             JOIN stocks s ON s.id = t.stock_id
             ORDER BY t.created_at DESC
             LIMIT ?1",
        )?;
        let rows = stmt
            .query_map([limit], |row| {
                Ok(TradeRecord {
                    ticker: row.get(0)?,
                    side: row.get(1)?,
                    date: row.get(2)?,
                    price: row
                        .get::<_, String>(3)?
                        .parse::<Decimal>()
                        .unwrap_or_default(),
                    quantity: row
                        .get::<_, String>(4)?
                        .parse::<Decimal>()
                        .unwrap_or_default(),
                    pnl: row
                        .get::<_, Option<String>>(5)?
                        .and_then(|s| s.parse::<Decimal>().ok()),
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok::<Vec<TradeRecord>, rusqlite::Error>(rows)
    })
    .await
    .context("Failed to get trade history")
}
