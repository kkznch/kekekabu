use anyhow::Result;
use tokio_rusqlite::Connection;

use crate::db;
use crate::output::{self, OutputFormat};
use crate::portfolio;

pub async fn watchlist(conn: &Connection, format: OutputFormat) -> Result<()> {
    let items = db::watchlist_list(conn).await?;
    output::print_list_output(&items, format);
    Ok(())
}

pub async fn events(conn: &Connection, ticker: Option<&str>, format: OutputFormat) -> Result<()> {
    let events = db::list_watchlist_events(conn, ticker).await?;
    output::print_list_output(&events, format);
    Ok(())
}

pub async fn positions(conn: &Connection, format: OutputFormat) -> Result<()> {
    let positions = portfolio::list_positions(conn).await?;
    output::print_list_output(&positions, format);
    Ok(())
}

pub async fn evaluations(conn: &Connection, limit: i64, format: OutputFormat) -> Result<()> {
    let evals = db::list_evaluations(conn, limit).await?;
    output::print_list_output(&evals, format);
    Ok(())
}

pub async fn stocks(conn: &Connection, format: OutputFormat) -> Result<()> {
    let stocks = db::list_stocks(conn).await?;
    output::print_list_output(&stocks, format);
    Ok(())
}

pub async fn tables(conn: &Connection, format: OutputFormat) -> Result<()> {
    let stats = db::table_stats(conn).await?;
    output::print_list_output(&stats, format);
    Ok(())
}

pub async fn summary(conn: &Connection, format: OutputFormat) -> Result<()> {
    let sum = portfolio::summary(conn).await?;
    output::print_output(&sum, format);
    Ok(())
}

pub async fn trades(conn: &Connection, limit: i64, format: OutputFormat) -> Result<()> {
    let trades = portfolio::trade_history(conn, limit).await?;
    output::print_list_output(&trades, format);
    Ok(())
}

pub async fn llm_logs(
    conn: &Connection,
    limit: i64,
    ticker: Option<&str>,
    format: OutputFormat,
) -> Result<()> {
    let logs = db::list_llm_logs(conn, limit, ticker).await?;
    output::print_list_output(&logs, format);
    Ok(())
}
