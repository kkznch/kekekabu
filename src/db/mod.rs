mod embedded {
    refinery::embed_migrations!("migrations");
}

use anyhow::{Context, Result};
use async_trait::async_trait;
use rust_decimal::Decimal;
use std::str::FromStr;
use tokio_rusqlite::Connection;

use crate::portfolio::{self, PortfolioSummary, PositionView, TradeRecord};

// ─── Type definitions ─────────────────────────────────────────────────

pub struct StockRecord {
    pub name: String,
    pub sector: Option<String>,
}

pub struct PriceData {
    pub closes: Vec<f64>,
    pub highs: Vec<f64>,
    pub lows: Vec<f64>,
    pub volumes: Vec<f64>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct WatchlistItem {
    pub stock_id: i64,
    pub ticker: String,
    pub name: String,
    pub sector: Option<String>,
    pub notes: Option<String>,
    pub added_at: String,
}

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

pub struct TradeCashSummary {
    pub total_invested: f64,
    pub total_recovered: f64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct WatchlistEvent {
    pub ticker: String,
    pub action: String,
    pub reason: Option<String>,
    pub discovered_at: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct StockInfo {
    pub ticker: String,
    pub name: String,
    pub sector: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TableStat {
    pub table_name: String,
    pub row_count: i64,
}

#[derive(Debug, serde::Serialize)]
pub struct LlmLog {
    pub id: i64,
    pub command: String,
    pub ticker: Option<String>,
    pub backend: String,
    pub model: Option<String>,
    pub temperature: Option<f64>,
    pub prompt: String,
    pub response: String,
    pub created_at: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Order {
    pub id: i64,
    pub stock_id: i64,
    pub ticker: String,
    pub name: String,
    pub side: String,
    pub order_type: String,
    pub price: String,
    pub quantity: String,
    pub status: String,
    pub tachibana_order_id: Option<String>,
    pub request_id: String,
    pub filled_price: Option<String>,
    pub filled_quantity: Option<String>,
    pub filled_at: Option<String>,
    pub evaluation_id: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}

pub struct FillParams {
    pub order_id: i64,
    pub status: String,
    pub tachibana_order_id: Option<String>,
    pub filled_price: Option<String>,
    pub filled_quantity: Option<String>,
    pub filled_at: Option<String>,
    pub ticker: String,
    pub side: String,
}

// ─── Utility functions ────────────────────────────────────────────────

pub fn db_path() -> std::path::PathBuf {
    if let Some(dir) = crate::config::config_dir() {
        // Ensure directory exists for DB file creation
        let _ = std::fs::create_dir_all(&dir);
        return dir.join("kekekabu.db");
    }
    std::path::PathBuf::from("./kekekabu.db")
}

fn decimal_to_f64(d: Decimal) -> f64 {
    d.to_string().parse::<f64>().unwrap_or(0.0)
}

pub fn decimal_str_to_f64(s: &str) -> f64 {
    match Decimal::from_str(s) {
        Ok(d) => d.to_string().parse::<f64>().unwrap_or(0.0),
        Err(e) => {
            tracing::warn!(value = %s, error = %e, "Failed to parse decimal string");
            0.0
        }
    }
}

fn map_order_row(row: &rusqlite::Row) -> rusqlite::Result<Order> {
    Ok(Order {
        id: row.get(0)?,
        stock_id: row.get(1)?,
        ticker: row.get(2)?,
        name: row.get(3)?,
        side: row.get(4)?,
        order_type: row.get(5)?,
        price: row.get(6)?,
        quantity: row.get(7)?,
        status: row.get(8)?,
        tachibana_order_id: row.get(9)?,
        request_id: row.get(10)?,
        filled_price: row.get(11)?,
        filled_quantity: row.get(12)?,
        filled_at: row.get(13)?,
        evaluation_id: row.get(14)?,
        created_at: row.get(15)?,
        updated_at: row.get(16)?,
    })
}

// ─── DbClient trait ───────────────────────────────────────────────────

#[async_trait]
pub trait DbClient: Send + Sync {
    // Stock operations
    async fn save_stock(&self, ticker: &str, name: &str, sector: Option<&str>) -> Result<i64>;
    async fn save_stocks_bulk(&self, stocks: &[crate::jquants::ListedInfo]) -> Result<usize>;
    async fn has_any_stocks(&self) -> Result<bool>;
    async fn get_stock_info(&self, stock_id: i64) -> Result<Option<StockRecord>>;
    async fn get_stock_id(&self, ticker: &str) -> Result<Option<i64>>;
    async fn list_stocks(&self) -> Result<Vec<StockInfo>>;

    // Price operations
    async fn save_prices(&self, stock_id: i64, quotes: &[crate::jquants::DailyQuote])
    -> Result<()>;
    async fn fetch_price_data(&self, stock_id: i64) -> Result<PriceData>;
    async fn get_latest_close(&self, stock_id: i64) -> Result<Option<f64>>;

    // Watchlist operations
    async fn watchlist_add(&self, ticker: &str, notes: Option<&str>) -> Result<()>;
    async fn watchlist_remove(&self, ticker: &str) -> Result<()>;
    async fn watchlist_list(&self) -> Result<Vec<WatchlistItem>>;
    async fn save_watchlist_event(
        &self,
        ticker: &str,
        action: &str,
        reason: Option<&str>,
    ) -> Result<()>;
    async fn list_watchlist_events(&self, ticker: Option<&str>) -> Result<Vec<WatchlistEvent>>;

    // Evaluation operations
    #[allow(clippy::too_many_arguments)]
    async fn save_evaluation(
        &self,
        stock_id: i64,
        decision: &str,
        score: i32,
        rationale: &str,
        ta_summary: Option<&str>,
        spec_hash: Option<&str>,
        llm_backend: Option<&str>,
    ) -> Result<i64>;
    async fn list_evaluations(&self, limit: i64) -> Result<Vec<Evaluation>>;
    async fn get_recent_evaluations_by_stock(
        &self,
        stock_id: i64,
        limit: i64,
    ) -> Result<Vec<Evaluation>>;
    async fn get_evaluations_for_date(&self, date: &str) -> Result<Vec<Evaluation>>;
    async fn get_latest_evaluations_for_today(&self) -> Result<Vec<Evaluation>>;

    // Fetch results
    #[allow(clippy::too_many_arguments)]
    async fn save_fetch_result(
        &self,
        stock_id: i64,
        source: &str,
        category: &str,
        title: &str,
        url: Option<&str>,
        body: Option<&str>,
        published_at: Option<&str>,
    ) -> Result<i64>;
    async fn get_fetch_results_for_stock(&self, stock_id: i64) -> Result<Vec<FetchResult>>;

    // Trade/Portfolio
    async fn trade_cash_summary(&self) -> Result<TradeCashSummary>;
    async fn list_positions(&self) -> Result<Vec<PositionView>>;
    async fn portfolio_summary(&self) -> Result<PortfolioSummary>;
    async fn trade_history(&self, limit: i64) -> Result<Vec<TradeRecord>>;

    // LLM logs
    #[allow(clippy::too_many_arguments)]
    async fn save_llm_log(
        &self,
        command: &str,
        ticker: Option<&str>,
        backend: &str,
        model: Option<&str>,
        temperature: Option<f32>,
        prompt: &str,
        response: &str,
    ) -> Result<()>;
    async fn list_llm_logs(&self, limit: i64, ticker: Option<&str>) -> Result<Vec<LlmLog>>;

    // Orders
    #[allow(clippy::too_many_arguments)]
    async fn save_order(
        &self,
        stock_id: i64,
        side: &str,
        order_type: &str,
        price: &str,
        quantity: &str,
        request_id: &str,
        evaluation_id: Option<i64>,
    ) -> Result<i64>;
    async fn update_order_status(
        &self,
        order_id: i64,
        status: &str,
        tachibana_order_id: Option<&str>,
        filled_price: Option<&str>,
        filled_quantity: Option<&str>,
        filled_at: Option<&str>,
    ) -> Result<()>;
    async fn list_pending_orders(&self) -> Result<Vec<Order>>;
    async fn list_orders(&self, limit: i64, status: Option<&str>) -> Result<Vec<Order>>;
    async fn order_exists_for_evaluation(&self, evaluation_id: i64, side: &str) -> Result<bool>;
    async fn update_order_and_record_fill(&self, params: FillParams) -> Result<()>;

    // Table stats
    async fn table_stats(&self) -> Result<Vec<TableStat>>;
}

// ─── SqliteClient ─────────────────────────────────────────────────────

pub struct SqliteClient {
    conn: Connection,
}

impl SqliteClient {
    /// Open existing database. Fails if DB file does not exist.
    /// Use `open_or_create()` to create a new database (called by `kabu db migrate`).
    pub async fn open() -> Result<Self> {
        let path = db_path();

        if !path.exists() {
            anyhow::bail!(
                "Database not found at {}\nRun `kabu db migrate` to create and initialize the database.",
                path.display()
            );
        }

        Self::open_and_migrate(path).await
    }

    /// Create database if needed and apply migrations.
    /// This is the only entry point that creates a new DB file.
    pub async fn open_or_create() -> Result<Self> {
        let path = db_path();

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory {:?}", parent))?;
        }

        Self::open_and_migrate(path).await
    }

    async fn open_and_migrate(path: std::path::PathBuf) -> Result<Self> {
        // Run versioned migrations with a raw rusqlite connection.
        // refinery requires &mut Connection, which tokio-rusqlite doesn't expose.
        {
            let path = path.clone();
            tokio::task::spawn_blocking(move || -> Result<()> {
                let mut conn = rusqlite::Connection::open(&path)
                    .with_context(|| format!("Failed to open database at {:?}", path))?;
                conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA busy_timeout=5000;")?;
                embedded::migrations::runner()
                    .run(&mut conn)
                    .map_err(|e| anyhow::anyhow!("Schema migration failed: {}", e))?;
                Ok(())
            })
            .await
            .context("Migration task panicked")??;
        }

        // Open async connection for runtime use
        let conn = Connection::open(&path)
            .await
            .with_context(|| format!("Failed to open database at {:?}", path))?;

        conn.call(|conn| {
            conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA busy_timeout=5000;")?;
            Ok::<(), rusqlite::Error>(())
        })
        .await
        .context("Failed to set database pragmas")?;

        Ok(Self { conn })
    }
}

impl SqliteClient {
    pub async fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory().await?;
        conn.call(|conn| {
            embedded::migrations::runner()
                .run(conn)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
            Ok::<(), rusqlite::Error>(())
        })
        .await
        .context("Failed to run migrations on in-memory DB")?;
        Ok(Self { conn })
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    pub async fn portfolio_buy(
        &self,
        ticker: &str,
        quantity: Decimal,
        price: Decimal,
        strategy: Option<&str>,
    ) -> Result<()> {
        let ticker = ticker.to_string();
        let strategy = strategy.map(|s| s.to_string());
        self.conn
            .call(move |conn| {
                let tx = conn.transaction()?;
                portfolio::buy_sync(&tx, &ticker, quantity, price, strategy.as_deref())?;
                tx.commit()?;
                Ok::<(), rusqlite::Error>(())
            })
            .await
            .context("Failed to execute portfolio buy")
    }

    pub async fn portfolio_sell(
        &self,
        ticker: &str,
        quantity: Decimal,
        price: Decimal,
        strategy: Option<&str>,
    ) -> Result<()> {
        let ticker = ticker.to_string();
        let strategy = strategy.map(|s| s.to_string());
        self.conn
            .call(move |conn| {
                let tx = conn.transaction()?;
                portfolio::sell_sync(&tx, &ticker, quantity, price, strategy.as_deref())?;
                tx.commit()?;
                Ok::<(), rusqlite::Error>(())
            })
            .await
            .context("Failed to execute portfolio sell")
    }
}

// ─── Migration info ───────────────────────────────────────────────────

#[derive(Debug, serde::Serialize)]
pub struct MigrationInfo {
    pub version: i32,
    pub name: String,
    pub applied_on: String,
}

impl SqliteClient {
    pub async fn migration_status(&self) -> Result<Vec<MigrationInfo>> {
        self.conn
            .call(|conn| {
                // refinery_schema_history may not exist if no migrations have been applied
                let has_table: bool = conn
                    .query_row(
                        "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='refinery_schema_history'",
                        [],
                        |row| row.get(0),
                    )
                    .unwrap_or(false);

                if !has_table {
                    return Ok(Vec::new());
                }

                let mut stmt = conn.prepare(
                    "SELECT version, name, applied_on FROM refinery_schema_history ORDER BY version",
                )?;
                let rows = stmt
                    .query_map([], |row| {
                        Ok(MigrationInfo {
                            version: row.get(0)?,
                            name: row.get(1)?,
                            applied_on: row.get(2)?,
                        })
                    })?
                    .collect::<std::result::Result<Vec<_>, _>>()?;
                Ok::<Vec<MigrationInfo>, rusqlite::Error>(rows)
            })
            .await
            .context("Failed to get migration status")
    }
}

// ─── DbClient implementation for SqliteClient ────────────────────────

#[async_trait]
impl DbClient for SqliteClient {
    // -- Stock operations --

    async fn save_stock(&self, ticker: &str, name: &str, sector: Option<&str>) -> Result<i64> {
        let ticker = ticker.to_string();
        let name = name.to_string();
        let sector = sector.map(|s| s.to_string());

        self.conn
            .call(move |conn| {
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

    async fn save_stocks_bulk(&self, stocks: &[crate::jquants::ListedInfo]) -> Result<usize> {
        let stocks: Vec<(String, String, Option<String>)> = stocks
            .iter()
            .map(|s| (s.code.clone(), s.company_name.clone(), s.sector.clone()))
            .collect();
        let count = stocks.len();

        self.conn
            .call(move |conn| {
                let tx = conn.transaction()?;
                for (ticker, name, sector) in &stocks {
                    tx.execute(
                        "INSERT INTO stocks (ticker, name, market, sector)
                         VALUES (?1, ?2, 'jp', ?3)
                         ON CONFLICT(ticker) DO UPDATE SET
                           name = excluded.name,
                           sector = excluded.sector,
                           updated_at = datetime('now')",
                        rusqlite::params![ticker, name, sector],
                    )?;
                }
                tx.commit()?;
                Ok::<(), rusqlite::Error>(())
            })
            .await
            .context("Failed to save stocks in bulk")?;

        Ok(count)
    }

    async fn has_any_stocks(&self) -> Result<bool> {
        self.conn
            .call(|conn| {
                let count: i64 =
                    conn.query_row("SELECT COUNT(*) FROM stocks LIMIT 1", [], |row| row.get(0))?;
                Ok::<bool, rusqlite::Error>(count > 0)
            })
            .await
            .context("Failed to check stocks table")
    }

    async fn get_stock_info(&self, stock_id: i64) -> Result<Option<StockRecord>> {
        self.conn
            .call(move |conn| {
                let result = conn
                    .query_row(
                        "SELECT name, sector FROM stocks WHERE id = ?1",
                        [stock_id],
                        |row| {
                            Ok(StockRecord {
                                name: row.get(0)?,
                                sector: row.get(1)?,
                            })
                        },
                    )
                    .ok();
                Ok::<Option<StockRecord>, rusqlite::Error>(result)
            })
            .await
            .context("Failed to get stock info")
    }

    async fn get_stock_id(&self, ticker: &str) -> Result<Option<i64>> {
        let ticker = ticker.to_string();
        self.conn
            .call(move |conn| {
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

    async fn list_stocks(&self) -> Result<Vec<StockInfo>> {
        self.conn
            .call(|conn| {
                let mut stmt = conn.prepare(
                    "SELECT ticker, name, sector, created_at FROM stocks ORDER BY ticker ASC",
                )?;
                let rows = stmt
                    .query_map([], |row| {
                        Ok(StockInfo {
                            ticker: row.get(0)?,
                            name: row.get(1)?,
                            sector: row.get(2)?,
                            created_at: row.get(3)?,
                        })
                    })?
                    .collect::<std::result::Result<Vec<_>, _>>()?;
                Ok::<Vec<StockInfo>, rusqlite::Error>(rows)
            })
            .await
            .context("Failed to list stocks")
    }

    // -- Price operations --

    async fn save_prices(
        &self,
        stock_id: i64,
        quotes: &[crate::jquants::DailyQuote],
    ) -> Result<()> {
        let quotes: Vec<crate::jquants::DailyQuote> = quotes.to_vec();

        self.conn
            .call(move |conn| {
                let tx = conn.transaction()?;
                for q in &quotes {
                    let open = q.open.and_then(Decimal::from_f64_retain);
                    let high = q.high.and_then(Decimal::from_f64_retain);
                    let low = q.low.and_then(Decimal::from_f64_retain);
                    let close = q.close.and_then(Decimal::from_f64_retain);
                    let volume = q.volume.map(|v| v as i64).unwrap_or(0);
                    let adj_close = q.adjustment_close.and_then(Decimal::from_f64_retain);

                    let (Some(open), Some(high), Some(low), Some(close)) =
                        (open, high, low, close)
                    else {
                        continue;
                    };

                    tx.execute(
                        "INSERT INTO prices (stock_id, date, open, high, low, close, volume, adjusted_close)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                         ON CONFLICT(stock_id, date) DO UPDATE SET
                           open = excluded.open, high = excluded.high, low = excluded.low,
                           close = excluded.close, volume = excluded.volume,
                           adjusted_close = excluded.adjusted_close",
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

    async fn fetch_price_data(&self, stock_id: i64) -> Result<PriceData> {
        self.conn
            .call(move |conn| {
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

    async fn get_latest_close(&self, stock_id: i64) -> Result<Option<f64>> {
        self.conn
            .call(move |conn| {
                let result = conn
                    .query_row(
                        "SELECT close FROM prices WHERE stock_id = ?1 ORDER BY date DESC LIMIT 1",
                        [stock_id],
                        |row| {
                            let close_str: String = row.get(0)?;
                            Ok(decimal_str_to_f64(&close_str))
                        },
                    )
                    .ok();
                Ok::<Option<f64>, rusqlite::Error>(result)
            })
            .await
            .context("Failed to get latest close price")
    }

    // -- Watchlist operations --

    async fn watchlist_add(&self, ticker: &str, notes: Option<&str>) -> Result<()> {
        let ticker = ticker.to_string();
        let ticker_ctx = ticker.clone();
        let notes = notes.map(|s| s.to_string());

        self.conn
            .call(move |conn| {
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

    async fn watchlist_remove(&self, ticker: &str) -> Result<()> {
        let ticker = ticker.to_string();
        let ticker_ctx = ticker.clone();

        self.conn
            .call(move |conn| {
                conn.execute(
                    "DELETE FROM watchlist WHERE stock_id IN (SELECT id FROM stocks WHERE ticker = ?1)",
                    rusqlite::params![ticker],
                )?;
                Ok::<(), rusqlite::Error>(())
            })
            .await
            .with_context(|| format!("Failed to remove {} from watchlist", ticker_ctx))
    }

    async fn watchlist_list(&self) -> Result<Vec<WatchlistItem>> {
        self.conn
            .call(|conn| {
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

    async fn save_watchlist_event(
        &self,
        ticker: &str,
        action: &str,
        reason: Option<&str>,
    ) -> Result<()> {
        let ticker = ticker.to_string();
        let action = action.to_string();
        let reason = reason.map(|s| s.to_string());

        self.conn
            .call(move |conn| {
                conn.execute(
                    "INSERT INTO watchlist_events (ticker, action, reason) VALUES (?1, ?2, ?3)",
                    rusqlite::params![ticker, action, reason],
                )?;
                Ok::<(), rusqlite::Error>(())
            })
            .await
            .context("Failed to save watchlist event")
    }

    async fn list_watchlist_events(&self, ticker: Option<&str>) -> Result<Vec<WatchlistEvent>> {
        let ticker = ticker.map(|s| s.to_string());
        self.conn
            .call(move |conn| {
                let (sql, params): (&str, Vec<Box<dyn rusqlite::types::ToSql>>) =
                    if let Some(ref t) = ticker {
                        (
                            "SELECT ticker, action, reason, discovered_at FROM watchlist_events WHERE ticker = ?1 ORDER BY discovered_at DESC",
                            vec![Box::new(t.clone())],
                        )
                    } else {
                        (
                            "SELECT ticker, action, reason, discovered_at FROM watchlist_events ORDER BY discovered_at DESC",
                            vec![],
                        )
                    };
                let mut stmt = conn.prepare(sql)?;
                let rows = stmt
                    .query_map(rusqlite::params_from_iter(params.iter()), |row| {
                        Ok(WatchlistEvent {
                            ticker: row.get(0)?,
                            action: row.get(1)?,
                            reason: row.get(2)?,
                            discovered_at: row.get(3)?,
                        })
                    })?
                    .collect::<std::result::Result<Vec<_>, _>>()?;
                Ok::<Vec<WatchlistEvent>, rusqlite::Error>(rows)
            })
            .await
            .context("Failed to list watchlist events")
    }

    // -- Evaluation operations --

    async fn save_evaluation(
        &self,
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

        self.conn
            .call(move |conn| {
                conn.execute(
                    "INSERT INTO evaluations (stock_id, decision, score, rationale, ta_summary, spec_hash, llm_backend)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    rusqlite::params![
                        stock_id, decision, score, rationale, ta_summary, spec_hash, llm_backend
                    ],
                )?;
                Ok::<i64, rusqlite::Error>(conn.last_insert_rowid())
            })
            .await
            .context("Failed to save evaluation")
    }

    async fn list_evaluations(&self, limit: i64) -> Result<Vec<Evaluation>> {
        self.conn
            .call(move |conn| {
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

    async fn get_recent_evaluations_by_stock(
        &self,
        stock_id: i64,
        limit: i64,
    ) -> Result<Vec<Evaluation>> {
        self.conn
            .call(move |conn| {
                let mut stmt = conn.prepare(
                    "SELECT e.id, s.ticker, s.name, e.decision, e.score, e.rationale, e.evaluated_at
                     FROM evaluations e
                     JOIN stocks s ON s.id = e.stock_id
                     WHERE e.stock_id = ?1
                     ORDER BY e.id DESC
                     LIMIT ?2",
                )?;
                let rows = stmt
                    .query_map(rusqlite::params![stock_id, limit], |row| {
                        Ok(Evaluation {
                            id: row.get(0)?,
                            ticker: row.get(1)?,
                            name: row.get(2)?,
                            decision: row.get(3)?,
                            score: row.get(4)?,
                            rationale: row.get(5)?,
                            ta_summary: None,
                            evaluated_at: row.get(6)?,
                        })
                    })?
                    .collect::<std::result::Result<Vec<_>, _>>()?;
                Ok::<Vec<Evaluation>, rusqlite::Error>(rows)
            })
            .await
            .context("Failed to get recent evaluations by stock")
    }

    async fn get_evaluations_for_date(&self, date: &str) -> Result<Vec<Evaluation>> {
        let date = date.to_string();
        self.conn
            .call(move |conn| {
                let date_start = format!("{} 00:00:00", date);
                let date_end = format!("{} 23:59:59", date);
                let mut stmt = conn.prepare(
                    "SELECT e.id, s.ticker, s.name, e.decision, e.score, e.rationale, e.ta_summary, e.evaluated_at
                     FROM evaluations e
                     JOIN stocks s ON s.id = e.stock_id
                     WHERE e.evaluated_at BETWEEN ?1 AND ?2
                     ORDER BY e.score DESC",
                )?;
                let rows = stmt
                    .query_map(rusqlite::params![date_start, date_end], |row| {
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
            .context("Failed to get evaluations for date")
    }

    async fn get_latest_evaluations_for_today(&self) -> Result<Vec<Evaluation>> {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        self.conn
            .call(move |conn| {
                let mut stmt = conn.prepare(
                    "SELECT e.id, s.ticker, s.name, e.decision, e.score, e.rationale, e.ta_summary, e.evaluated_at
                     FROM evaluations e
                     JOIN stocks s ON s.id = e.stock_id
                     WHERE e.evaluated_at >= ?1
                       AND e.id = (SELECT MAX(e2.id) FROM evaluations e2 WHERE e2.stock_id = e.stock_id AND e2.evaluated_at >= ?1)
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

    // -- Fetch results --

    async fn save_fetch_result(
        &self,
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

        // Coalesce None URL to empty string for UNIQUE(stock_id, url) to work
        let url = url.or(Some(String::new()));

        self.conn
            .call(move |conn| {
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

    async fn get_fetch_results_for_stock(&self, stock_id: i64) -> Result<Vec<FetchResult>> {
        self.conn
            .call(move |conn| {
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

    // -- Trade/Portfolio --

    async fn trade_cash_summary(&self) -> Result<TradeCashSummary> {
        self.conn
            .call(|conn| {
                let mut stmt = conn.prepare(
                    "SELECT side, price, quantity FROM trades WHERE side IN ('buy', 'sell')",
                )?;
                let mut total_invested = Decimal::ZERO;
                let mut total_recovered = Decimal::ZERO;

                let rows = stmt.query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                })?;

                for row in rows {
                    let (side, price_str, qty_str) = row?;
                    let price = Decimal::from_str(&price_str).unwrap_or_default();
                    let qty = Decimal::from_str(&qty_str).unwrap_or_default();
                    let amount = price * qty;

                    match side.as_str() {
                        "buy" => total_invested += amount,
                        "sell" => total_recovered += amount,
                        _ => {}
                    }
                }

                Ok::<TradeCashSummary, rusqlite::Error>(TradeCashSummary {
                    total_invested: decimal_to_f64(total_invested),
                    total_recovered: decimal_to_f64(total_recovered),
                })
            })
            .await
            .context("Failed to get trade cash summary")
    }

    async fn list_positions(&self) -> Result<Vec<PositionView>> {
        self.conn
            .call(|conn| {
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
                        let current_price =
                            latest_str.as_deref().and_then(|s| Decimal::from_str(s).ok());
                        let current_value = current_price.map(|p| p * quantity);
                        let unrealized_pnl = current_value.map(|cv| cv - total_invested);
                        let unrealized_pnl_pct = unrealized_pnl.and_then(|pnl| {
                            if total_invested.is_zero() {
                                None
                            } else {
                                Some(pnl / total_invested * Decimal::from(100))
                            }
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

    async fn portfolio_summary(&self) -> Result<PortfolioSummary> {
        let positions = self.list_positions().await?;
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

    async fn trade_history(&self, limit: i64) -> Result<Vec<TradeRecord>> {
        self.conn
            .call(move |conn| {
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

    // -- LLM logs --

    async fn save_llm_log(
        &self,
        command: &str,
        ticker: Option<&str>,
        backend: &str,
        model: Option<&str>,
        temperature: Option<f32>,
        prompt: &str,
        response: &str,
    ) -> Result<()> {
        let command = command.to_string();
        let ticker = ticker.map(|s| s.to_string());
        let backend = backend.to_string();
        let model = model.map(|s| s.to_string());
        let prompt = prompt.to_string();
        let response = response.to_string();

        self.conn
            .call(move |conn| {
                conn.execute(
                    "INSERT INTO llm_logs (command, ticker, backend, model, temperature, prompt, response)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    rusqlite::params![
                        command,
                        ticker,
                        backend,
                        model,
                        temperature,
                        prompt,
                        response
                    ],
                )?;
                Ok::<(), rusqlite::Error>(())
            })
            .await
            .context("Failed to save LLM log")
    }

    async fn list_llm_logs(&self, limit: i64, ticker: Option<&str>) -> Result<Vec<LlmLog>> {
        let ticker = ticker.map(|s| s.to_string());

        self.conn
            .call(move |conn| {
                let (sql, params): (String, Vec<Box<dyn rusqlite::types::ToSql>>) = match &ticker {
                    Some(t) => (
                        "SELECT id, command, ticker, backend, model, temperature, prompt, response, created_at
                         FROM llm_logs WHERE ticker = ?1 ORDER BY id DESC LIMIT ?2"
                            .to_string(),
                        vec![
                            Box::new(t.clone()) as Box<dyn rusqlite::types::ToSql>,
                            Box::new(limit),
                        ],
                    ),
                    None => (
                        "SELECT id, command, ticker, backend, model, temperature, prompt, response, created_at
                         FROM llm_logs ORDER BY id DESC LIMIT ?1"
                            .to_string(),
                        vec![Box::new(limit) as Box<dyn rusqlite::types::ToSql>],
                    ),
                };

                let mut stmt = conn.prepare(&sql)?;
                let rows = stmt
                    .query_map(rusqlite::params_from_iter(params.iter()), |row| {
                        Ok(LlmLog {
                            id: row.get(0)?,
                            command: row.get(1)?,
                            ticker: row.get(2)?,
                            backend: row.get(3)?,
                            model: row.get(4)?,
                            temperature: row.get(5)?,
                            prompt: row.get(6)?,
                            response: row.get(7)?,
                            created_at: row.get(8)?,
                        })
                    })?
                    .collect::<std::result::Result<Vec<_>, _>>()?;
                Ok::<Vec<LlmLog>, rusqlite::Error>(rows)
            })
            .await
            .context("Failed to list LLM logs")
    }

    // -- Orders --

    async fn save_order(
        &self,
        stock_id: i64,
        side: &str,
        order_type: &str,
        price: &str,
        quantity: &str,
        request_id: &str,
        evaluation_id: Option<i64>,
    ) -> Result<i64> {
        let side = side.to_string();
        let order_type = order_type.to_string();
        let price = price.to_string();
        let quantity = quantity.to_string();
        let request_id = request_id.to_string();

        self.conn
            .call(move |conn| {
                conn.execute(
                    "INSERT OR IGNORE INTO orders (stock_id, side, order_type, price, quantity, request_id, evaluation_id)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    rusqlite::params![
                        stock_id, side, order_type, price, quantity, request_id, evaluation_id
                    ],
                )?;
                let id: i64 = conn.query_row(
                    "SELECT id FROM orders WHERE request_id = ?1",
                    rusqlite::params![request_id],
                    |row| row.get(0),
                )?;
                Ok::<i64, rusqlite::Error>(id)
            })
            .await
            .context("Failed to save order")
    }

    async fn update_order_status(
        &self,
        order_id: i64,
        status: &str,
        tachibana_order_id: Option<&str>,
        filled_price: Option<&str>,
        filled_quantity: Option<&str>,
        filled_at: Option<&str>,
    ) -> Result<()> {
        let status = status.to_string();
        let tachibana_order_id = tachibana_order_id.map(|s| s.to_string());
        let filled_price = filled_price.map(|s| s.to_string());
        let filled_quantity = filled_quantity.map(|s| s.to_string());
        let filled_at = filled_at.map(|s| s.to_string());

        self.conn
            .call(move |conn| {
                conn.execute(
                    "UPDATE orders SET status = ?1, tachibana_order_id = COALESCE(?2, tachibana_order_id),
                     filled_price = COALESCE(?3, filled_price), filled_quantity = COALESCE(?4, filled_quantity),
                     filled_at = COALESCE(?5, filled_at), updated_at = datetime('now')
                     WHERE id = ?6",
                    rusqlite::params![
                        status,
                        tachibana_order_id,
                        filled_price,
                        filled_quantity,
                        filled_at,
                        order_id
                    ],
                )?;
                Ok::<(), rusqlite::Error>(())
            })
            .await
            .context("Failed to update order status")
    }

    async fn list_pending_orders(&self) -> Result<Vec<Order>> {
        self.conn
            .call(move |conn| {
                let sql = "SELECT o.id, o.stock_id, s.ticker, s.name, o.side, o.order_type, o.price, o.quantity,
                                o.status, o.tachibana_order_id, o.request_id, o.filled_price, o.filled_quantity,
                                o.filled_at, o.evaluation_id, o.created_at, o.updated_at
                         FROM orders o JOIN stocks s ON s.id = o.stock_id
                         WHERE o.status IN ('pending', 'partial')
                         ORDER BY o.created_at DESC";
                let mut stmt = conn.prepare(sql)?;
                let rows = stmt
                    .query_map([], map_order_row)?
                    .collect::<std::result::Result<Vec<_>, _>>()?;
                Ok::<Vec<Order>, rusqlite::Error>(rows)
            })
            .await
            .context("Failed to list pending orders")
    }

    async fn list_orders(&self, limit: i64, status: Option<&str>) -> Result<Vec<Order>> {
        let status = status.map(|s| s.to_string());
        self.conn
            .call(move |conn| {
                let (sql, params): (String, Vec<Box<dyn rusqlite::types::ToSql>>) = match &status {
                    Some(s) => (
                        "SELECT o.id, o.stock_id, s.ticker, s.name, o.side, o.order_type, o.price, o.quantity,
                                o.status, o.tachibana_order_id, o.request_id, o.filled_price, o.filled_quantity,
                                o.filled_at, o.evaluation_id, o.created_at, o.updated_at
                         FROM orders o JOIN stocks s ON s.id = o.stock_id
                         WHERE o.status = ?1
                         ORDER BY o.created_at DESC LIMIT ?2"
                            .to_string(),
                        vec![
                            Box::new(s.clone()) as Box<dyn rusqlite::types::ToSql>,
                            Box::new(limit),
                        ],
                    ),
                    None => (
                        "SELECT o.id, o.stock_id, s.ticker, s.name, o.side, o.order_type, o.price, o.quantity,
                                o.status, o.tachibana_order_id, o.request_id, o.filled_price, o.filled_quantity,
                                o.filled_at, o.evaluation_id, o.created_at, o.updated_at
                         FROM orders o JOIN stocks s ON s.id = o.stock_id
                         ORDER BY o.created_at DESC LIMIT ?1"
                            .to_string(),
                        vec![Box::new(limit) as Box<dyn rusqlite::types::ToSql>],
                    ),
                };
                let mut stmt = conn.prepare(&sql)?;
                let rows = stmt
                    .query_map(rusqlite::params_from_iter(params.iter()), map_order_row)?
                    .collect::<std::result::Result<Vec<_>, _>>()?;
                Ok::<Vec<Order>, rusqlite::Error>(rows)
            })
            .await
            .context("Failed to list orders")
    }

    async fn order_exists_for_evaluation(&self, evaluation_id: i64, side: &str) -> Result<bool> {
        let side = side.to_string();
        self.conn
            .call(move |conn| {
                let count: i64 = conn.query_row(
                    "SELECT COUNT(*) FROM orders WHERE evaluation_id = ?1 AND side = ?2",
                    rusqlite::params![evaluation_id, side],
                    |row| row.get(0),
                )?;
                Ok::<bool, rusqlite::Error>(count > 0)
            })
            .await
            .context("Failed to check order existence")
    }

    async fn update_order_and_record_fill(&self, params: FillParams) -> Result<()> {
        self.conn
            .call(move |conn| {
                let tx = conn.transaction()?;

                // Update order status
                tx.execute(
                    "UPDATE orders SET status = ?1, tachibana_order_id = COALESCE(?2, tachibana_order_id),
                     filled_price = COALESCE(?3, filled_price), filled_quantity = COALESCE(?4, filled_quantity),
                     filled_at = COALESCE(?5, filled_at), updated_at = datetime('now')
                     WHERE id = ?6",
                    rusqlite::params![
                        params.status,
                        params.tachibana_order_id,
                        params.filled_price,
                        params.filled_quantity,
                        params.filled_at,
                        params.order_id
                    ],
                )?;

                // Record fill in portfolio
                if let (Some(ref fp), Some(ref fq)) = (params.filled_price, params.filled_quantity)
                {
                    let price = Decimal::from_str(fp).unwrap_or_default();
                    let qty = Decimal::from_str(fq).unwrap_or_default();

                    match params.side.as_str() {
                        "buy" => portfolio::buy_sync(
                            &tx,
                            &params.ticker,
                            qty,
                            price,
                            Some("tachibana-fill"),
                        )?,
                        "sell" => portfolio::sell_sync(
                            &tx,
                            &params.ticker,
                            qty,
                            price,
                            Some("tachibana-fill"),
                        )?,
                        _ => {}
                    }
                }

                tx.commit()?;
                Ok::<(), rusqlite::Error>(())
            })
            .await
            .context("Failed to update order and record fill")
    }

    // -- Table stats --

    async fn table_stats(&self) -> Result<Vec<TableStat>> {
        self.conn
            .call(|conn| {
                let tables = [
                    "stocks",
                    "prices",
                    "watchlist",
                    "evaluations",
                    "fetch_results",
                    "portfolio_positions",
                    "trades",
                    "watchlist_events",
                    "llm_logs",
                    "orders",
                ];
                let mut stats = Vec::new();
                for table in tables {
                    let count: i64 = conn
                        .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                            row.get(0)
                        })
                        .unwrap_or(0);
                    stats.push(TableStat {
                        table_name: table.to_string(),
                        row_count: count,
                    });
                }
                Ok::<Vec<TableStat>, rusqlite::Error>(stats)
            })
            .await
            .context("Failed to get table stats")
    }
}
