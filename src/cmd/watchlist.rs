use anyhow::Result;
use tokio_rusqlite::Connection;

use crate::db;

pub async fn add(conn: &Connection, ticker: &str, notes: Option<&str>) -> Result<()> {
    db::watchlist_add(conn, ticker, notes).await?;
    tracing::info!(ticker = ticker, "Added to watchlist");
    Ok(())
}

pub async fn remove(conn: &Connection, ticker: &str) -> Result<()> {
    db::watchlist_remove(conn, ticker).await?;
    tracing::info!(ticker = ticker, "Removed from watchlist");
    Ok(())
}

pub async fn list(conn: &Connection) -> Result<Vec<db::WatchlistItem>> {
    db::watchlist_list(conn).await
}
