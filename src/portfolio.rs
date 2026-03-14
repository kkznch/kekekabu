use rust_decimal::Decimal;
use serde::Serialize;
use std::str::FromStr;

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

/// Synchronous buy logic. Caller is responsible for transaction management.
pub(crate) fn buy_sync(
    conn: &rusqlite::Connection,
    ticker: &str,
    quantity: Decimal,
    price: Decimal,
    strategy: Option<&str>,
) -> std::result::Result<(), rusqlite::Error> {
    let quantity_str = quantity.to_string();
    let price_str = price.to_string();

    conn.execute(
        "INSERT OR IGNORE INTO stocks (ticker, name, market) VALUES (?1, ?1, 'jp')",
        rusqlite::params![ticker],
    )?;
    let stock_id: i64 =
        conn.query_row("SELECT id FROM stocks WHERE ticker = ?1", [ticker], |row| {
            row.get(0)
        })?;

    let existing: Option<(i64, String, String)> = conn
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

        let new_qty = old_qty + quantity;
        let new_avg = (old_avg * old_qty + price * quantity) / new_qty;
        let new_invested = new_avg * new_qty;

        conn.execute(
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
        let invested = quantity * price;

        // Check for inactive position (previously sold) and reactivate it
        let inactive_id: Option<i64> = conn
            .query_row(
                "SELECT id FROM portfolio_positions WHERE stock_id = ?1 AND is_active = 0",
                [stock_id],
                |row| row.get(0),
            )
            .ok();

        if let Some(pos_id) = inactive_id {
            conn.execute(
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
            conn.execute(
                "INSERT INTO portfolio_positions (stock_id, quantity, avg_cost, total_invested)
                 VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![stock_id, quantity_str, price_str, invested.to_string()],
            )?;
        }
    }

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    conn.execute(
        "INSERT INTO trades (stock_id, side, date, price, quantity, strategy)
         VALUES (?1, 'buy', ?2, ?3, ?4, ?5)",
        rusqlite::params![stock_id, today, price_str, quantity_str, strategy],
    )?;

    Ok(())
}

/// Synchronous sell logic. Caller is responsible for transaction management.
pub(crate) fn sell_sync(
    conn: &rusqlite::Connection,
    ticker: &str,
    quantity: Decimal,
    price: Decimal,
    strategy: Option<&str>,
) -> std::result::Result<(), rusqlite::Error> {
    let quantity_str = quantity.to_string();
    let price_str = price.to_string();

    let stock_id: i64 =
        conn.query_row("SELECT id FROM stocks WHERE ticker = ?1", [ticker], |row| {
            row.get(0)
        })?;

    let (pos_id, old_qty_str, old_avg_str): (i64, String, String) = conn.query_row(
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
    )?;

    let old_qty = Decimal::from_str(&old_qty_str).unwrap_or_default();
    let old_avg = Decimal::from_str(&old_avg_str).unwrap_or_default();

    if quantity > old_qty {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }

    let pnl = (price - old_avg) * quantity;
    let new_qty = old_qty - quantity;

    if new_qty.is_zero() {
        conn.execute(
            "UPDATE portfolio_positions
             SET quantity = '0', total_invested = '0', is_active = 0, updated_at = datetime('now')
             WHERE id = ?1",
            [pos_id],
        )?;

        // Auto-remove from watchlist when position is fully closed
        conn.execute("DELETE FROM watchlist WHERE stock_id = ?1", [stock_id])?;
        conn.execute(
            "INSERT INTO watchlist_events (ticker, action, reason) VALUES (?1, 'auto-removed-on-sell', 'Position closed')",
            rusqlite::params![ticker],
        )?;
    } else {
        let new_invested = old_avg * new_qty;
        conn.execute(
            "UPDATE portfolio_positions
             SET quantity = ?1, total_invested = ?2, updated_at = datetime('now')
             WHERE id = ?3",
            rusqlite::params![new_qty.to_string(), new_invested.to_string(), pos_id],
        )?;
    }

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    conn.execute(
        "INSERT INTO trades (stock_id, side, date, price, quantity, pnl, strategy)
         VALUES (?1, 'sell', ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            stock_id,
            today,
            price_str,
            quantity_str,
            pnl.to_string(),
            strategy
        ],
    )?;

    Ok(())
}
