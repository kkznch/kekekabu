pub mod schema;

use anyhow::{Context, Result};
use rust_decimal::Decimal;
use std::str::FromStr;
use tokio_rusqlite::Connection;

use self::schema::ALL_SCHEMAS;

fn db_path() -> std::path::PathBuf {
    if let Some(dir) = crate::config::config_dir() {
        return dir.join("kekekabu.db");
    }
    std::path::PathBuf::from("./kekekabu.db")
}

pub async fn create_tables(conn: &Connection) -> Result<()> {
    conn.call(|conn| {
        for sql in ALL_SCHEMAS {
            conn.execute_batch(sql)?;
        }
        Ok::<(), rusqlite::Error>(())
    })
    .await
    .context("Failed to create tables")?;
    Ok(())
}

pub async fn init_db() -> Result<Connection> {
    let path = db_path();
    let conn = Connection::open(&path)
        .await
        .with_context(|| format!("Failed to open database at {:?}", path))?;
    create_tables(&conn).await?;
    Ok(conn)
}

// -- Stock operations --

pub async fn save_stock(
    conn: &Connection,
    ticker: &str,
    name: &str,
    sector: Option<&str>,
) -> Result<i64> {
    let ticker = ticker.to_string();
    let name = name.to_string();
    let sector = sector.map(|s| s.to_string());

    conn.call(move |conn| {
        conn.execute(
            "INSERT INTO stocks (ticker, name, market, sector)
             VALUES (?1, ?2, 'jp', ?3)
             ON CONFLICT(ticker) DO UPDATE SET
               name = excluded.name,
               sector = excluded.sector,
               updated_at = datetime('now')",
            rusqlite::params![ticker, name, sector],
        )?;
        let id: i64 = conn.query_row(
            "SELECT id FROM stocks WHERE ticker = ?1",
            rusqlite::params![ticker],
            |row| row.get(0),
        )?;
        Ok::<i64, rusqlite::Error>(id)
    })
    .await
    .context("Failed to save stock")
}

pub async fn save_prices(
    conn: &Connection,
    stock_id: i64,
    quotes: &[crate::jquants::DailyQuote],
) -> Result<()> {
    let quotes: Vec<crate::jquants::DailyQuote> = quotes.to_vec();

    conn.call(move |conn| {
        let tx = conn.transaction()?;
        for q in &quotes {
            let open = q.open.and_then(Decimal::from_f64_retain);
            let high = q.high.and_then(Decimal::from_f64_retain);
            let low = q.low.and_then(Decimal::from_f64_retain);
            let close = q.close.and_then(Decimal::from_f64_retain);
            let volume = q.volume.map(|v| v as i64).unwrap_or(0);
            let adj_close = q.adjustment_close.and_then(Decimal::from_f64_retain);

            let (Some(open), Some(high), Some(low), Some(close)) = (open, high, low, close)
            else {
                continue;
            };

            tx.execute(
                "INSERT OR IGNORE INTO prices (stock_id, date, open, high, low, close, volume, adjusted_close)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                rusqlite::params![
                    stock_id,
                    q.date,
                    open.to_string(),
                    high.to_string(),
                    low.to_string(),
                    close.to_string(),
                    volume,
                    adj_close.map(|d| d.to_string()),
                ],
            )?;
        }
        tx.commit()?;
        Ok::<(), rusqlite::Error>(())
    })
    .await
    .context("Failed to save prices")
}

// -- Price data fetch for TA --

pub fn decimal_str_to_f64(s: &str) -> f64 {
    Decimal::from_str(s)
        .map(|d| d.to_string().parse::<f64>().unwrap_or(0.0))
        .unwrap_or(0.0)
}

pub struct PriceData {
    pub closes: Vec<f64>,
    pub highs: Vec<f64>,
    pub lows: Vec<f64>,
    pub volumes: Vec<f64>,
}

pub async fn fetch_price_data(conn: &Connection, stock_id: i64) -> Result<PriceData> {
    conn.call(move |conn| {
        let mut stmt = conn.prepare(
            "SELECT date, open, high, low, close, volume
             FROM prices WHERE stock_id = ?1 ORDER BY date ASC",
        )?;
        let mut closes = Vec::new();
        let mut highs = Vec::new();
        let mut lows = Vec::new();
        let mut volumes = Vec::new();

        let rows = stmt.query_map([stock_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, i64>(5)?,
            ))
        })?;

        for row in rows {
            let (_date, high, low, close, vol) = row?;
            highs.push(decimal_str_to_f64(&high));
            lows.push(decimal_str_to_f64(&low));
            closes.push(decimal_str_to_f64(&close));
            volumes.push(vol as f64);
        }

        Ok::<PriceData, rusqlite::Error>(PriceData {
            closes,
            highs,
            lows,
            volumes,
        })
    })
    .await
    .context("Failed to fetch price data")
}

// -- Watchlist operations --

#[derive(Debug, Clone, serde::Serialize)]
pub struct WatchlistItem {
    pub stock_id: i64,
    pub ticker: String,
    pub name: String,
    pub sector: Option<String>,
    pub notes: Option<String>,
    pub added_at: String,
}

pub async fn watchlist_add(
    conn: &Connection,
    ticker: &str,
    notes: Option<&str>,
) -> Result<()> {
    let ticker = ticker.to_string();
    let ticker_ctx = ticker.clone();
    let notes = notes.map(|s| s.to_string());

    conn.call(move |conn| {
        conn.execute(
            "INSERT OR IGNORE INTO stocks (ticker, name, market) VALUES (?1, ?1, 'jp')",
            rusqlite::params![ticker],
        )?;
        let stock_id: i64 = conn.query_row(
            "SELECT id FROM stocks WHERE ticker = ?1",
            rusqlite::params![ticker],
            |row| row.get(0),
        )?;
        conn.execute(
            "INSERT OR IGNORE INTO watchlist (stock_id, notes) VALUES (?1, ?2)",
            rusqlite::params![stock_id, notes],
        )?;
        Ok::<(), rusqlite::Error>(())
    })
    .await
    .with_context(|| format!("Failed to add {} to watchlist", ticker_ctx))
}

pub async fn watchlist_remove(conn: &Connection, ticker: &str) -> Result<()> {
    let ticker = ticker.to_string();
    let ticker_ctx = ticker.clone();

    conn.call(move |conn| {
        conn.execute(
            "DELETE FROM watchlist WHERE stock_id IN (SELECT id FROM stocks WHERE ticker = ?1)",
            rusqlite::params![ticker],
        )?;
        Ok::<(), rusqlite::Error>(())
    })
    .await
    .with_context(|| format!("Failed to remove {} from watchlist", ticker_ctx))
}

pub async fn watchlist_list(conn: &Connection) -> Result<Vec<WatchlistItem>> {
    conn.call(|conn| {
        let mut stmt = conn.prepare(
            "SELECT s.id, s.ticker, s.name, s.sector, w.notes, w.added_at
             FROM watchlist w
             JOIN stocks s ON s.id = w.stock_id
             ORDER BY w.added_at DESC",
        )?;
        let rows = stmt
            .query_map([], |row| {
                Ok(WatchlistItem {
                    stock_id: row.get(0)?,
                    ticker: row.get(1)?,
                    name: row.get(2)?,
                    sector: row.get(3)?,
                    notes: row.get(4)?,
                    added_at: row.get(5)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok::<Vec<WatchlistItem>, rusqlite::Error>(rows)
    })
    .await
    .context("Failed to list watchlist")
}

// -- Evaluation operations --

#[derive(Debug, Clone, serde::Serialize)]
pub struct Evaluation {
    pub id: i64,
    pub ticker: String,
    pub name: String,
    pub decision: String,
    pub score: i32,
    pub rationale: String,
    pub ta_summary: Option<String>,
    pub evaluated_at: String,
}

pub async fn save_evaluation(
    conn: &Connection,
    stock_id: i64,
    decision: &str,
    score: i32,
    rationale: &str,
    ta_summary: Option<&str>,
    spec_hash: Option<&str>,
    llm_backend: Option<&str>,
) -> Result<i64> {
    let decision = decision.to_string();
    let rationale = rationale.to_string();
    let ta_summary = ta_summary.map(|s| s.to_string());
    let spec_hash = spec_hash.map(|s| s.to_string());
    let llm_backend = llm_backend.map(|s| s.to_string());

    conn.call(move |conn| {
        conn.execute(
            "INSERT INTO evaluations (stock_id, decision, score, rationale, ta_summary, spec_hash, llm_backend)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![stock_id, decision, score, rationale, ta_summary, spec_hash, llm_backend],
        )?;
        Ok::<i64, rusqlite::Error>(conn.last_insert_rowid())
    })
    .await
    .context("Failed to save evaluation")
}

pub async fn list_evaluations(conn: &Connection, limit: i64) -> Result<Vec<Evaluation>> {
    conn.call(move |conn| {
        let mut stmt = conn.prepare(
            "SELECT e.id, s.ticker, s.name, e.decision, e.score, e.rationale, e.ta_summary, e.evaluated_at
             FROM evaluations e
             JOIN stocks s ON s.id = e.stock_id
             ORDER BY e.evaluated_at DESC
             LIMIT ?1",
        )?;
        let rows = stmt
            .query_map([limit], |row| {
                Ok(Evaluation {
                    id: row.get(0)?,
                    ticker: row.get(1)?,
                    name: row.get(2)?,
                    decision: row.get(3)?,
                    score: row.get(4)?,
                    rationale: row.get(5)?,
                    ta_summary: row.get(6)?,
                    evaluated_at: row.get(7)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok::<Vec<Evaluation>, rusqlite::Error>(rows)
    })
    .await
    .context("Failed to list evaluations")
}

// -- Stock lookup --

pub async fn get_stock_id(conn: &Connection, ticker: &str) -> Result<Option<i64>> {
    let ticker = ticker.to_string();
    conn.call(move |conn| {
        let result = conn
            .query_row(
                "SELECT id FROM stocks WHERE ticker = ?1",
                rusqlite::params![ticker],
                |row| row.get::<_, i64>(0),
            )
            .ok();
        Ok::<Option<i64>, rusqlite::Error>(result)
    })
    .await
    .context("Failed to lookup stock")
}

// -- Fetch results operations --

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FetchResult {
    pub id: i64,
    pub stock_id: i64,
    pub ticker: String,
    pub source: String,
    pub category: String,
    pub title: String,
    pub url: Option<String>,
    pub body: Option<String>,
    pub published_at: Option<String>,
    pub fetched_at: String,
}

pub async fn save_fetch_result(
    conn: &Connection,
    stock_id: i64,
    source: &str,
    category: &str,
    title: &str,
    url: Option<&str>,
    body: Option<&str>,
    published_at: Option<&str>,
) -> Result<i64> {
    let source = source.to_string();
    let category = category.to_string();
    let title = title.to_string();
    let url = url.map(|s| s.to_string());
    let body = body.map(|s| s.to_string());
    let published_at = published_at.map(|s| s.to_string());

    conn.call(move |conn| {
        conn.execute(
            "INSERT OR IGNORE INTO fetch_results (stock_id, source, category, title, url, body, published_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![stock_id, source, category, title, url, body, published_at],
        )?;
        Ok::<i64, rusqlite::Error>(conn.last_insert_rowid())
    })
    .await
    .context("Failed to save fetch result")
}

pub async fn get_fetch_results_for_stock(
    conn: &Connection,
    stock_id: i64,
) -> Result<Vec<FetchResult>> {
    conn.call(move |conn| {
        let mut stmt = conn.prepare(
            "SELECT fr.id, fr.stock_id, s.ticker, fr.source, fr.category, fr.title, fr.url, fr.body, fr.published_at, fr.fetched_at
             FROM fetch_results fr
             JOIN stocks s ON s.id = fr.stock_id
             WHERE fr.stock_id = ?1
             ORDER BY fr.fetched_at DESC",
        )?;
        let rows = stmt
            .query_map([stock_id], |row| {
                Ok(FetchResult {
                    id: row.get(0)?,
                    stock_id: row.get(1)?,
                    ticker: row.get(2)?,
                    source: row.get(3)?,
                    category: row.get(4)?,
                    title: row.get(5)?,
                    url: row.get(6)?,
                    body: row.get(7)?,
                    published_at: row.get(8)?,
                    fetched_at: row.get(9)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok::<Vec<FetchResult>, rusqlite::Error>(rows)
    })
    .await
    .context("Failed to get fetch results")
}

// -- Trade cash summary --

pub struct TradeCashSummary {
    pub total_invested: f64,
    pub total_recovered: f64,
}

pub async fn trade_cash_summary(conn: &Connection) -> Result<TradeCashSummary> {
    conn.call(|conn| {
        let total_invested: f64 = conn
            .query_row(
                "SELECT COALESCE(SUM(CAST(price AS REAL) * CAST(quantity AS REAL)), 0)
                 FROM trades WHERE side = 'buy'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0.0);

        let total_recovered: f64 = conn
            .query_row(
                "SELECT COALESCE(SUM(CAST(price AS REAL) * CAST(quantity AS REAL)), 0)
                 FROM trades WHERE side = 'sell'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0.0);

        Ok::<TradeCashSummary, rusqlite::Error>(TradeCashSummary {
            total_invested,
            total_recovered,
        })
    })
    .await
    .context("Failed to get trade cash summary")
}

pub async fn get_latest_evaluations_for_today(conn: &Connection) -> Result<Vec<Evaluation>> {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    conn.call(move |conn| {
        let mut stmt = conn.prepare(
            "SELECT e.id, s.ticker, s.name, e.decision, e.score, e.rationale, e.ta_summary, e.evaluated_at
             FROM evaluations e
             JOIN stocks s ON s.id = e.stock_id
             WHERE e.evaluated_at >= ?1
             ORDER BY e.score DESC",
        )?;
        let rows = stmt
            .query_map([today], |row| {
                Ok(Evaluation {
                    id: row.get(0)?,
                    ticker: row.get(1)?,
                    name: row.get(2)?,
                    decision: row.get(3)?,
                    score: row.get(4)?,
                    rationale: row.get(5)?,
                    ta_summary: row.get(6)?,
                    evaluated_at: row.get(7)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok::<Vec<Evaluation>, rusqlite::Error>(rows)
    })
    .await
    .context("Failed to get today's evaluations")
}
