use anyhow::Result;

use crate::db::DbClient;
use crate::output::{self, OutputFormat};

pub async fn watchlist(conn: &dyn DbClient, format: OutputFormat) -> Result<()> {
    let items = conn.watchlist_list().await?;
    output::print_list_output(&items, format);
    Ok(())
}

pub async fn events(conn: &dyn DbClient, ticker: Option<&str>, format: OutputFormat) -> Result<()> {
    let events = conn.list_watchlist_events(ticker).await?;
    output::print_list_output(&events, format);
    Ok(())
}

pub async fn positions(conn: &dyn DbClient, format: OutputFormat) -> Result<()> {
    let positions = conn.list_positions().await?;
    output::print_list_output(&positions, format);
    Ok(())
}

pub async fn evaluations(conn: &dyn DbClient, limit: i64, format: OutputFormat) -> Result<()> {
    let evals = conn.list_evaluations(limit).await?;
    output::print_list_output(&evals, format);
    Ok(())
}

pub async fn stocks(conn: &dyn DbClient, format: OutputFormat) -> Result<()> {
    let stocks = conn.list_stocks().await?;
    output::print_list_output(&stocks, format);
    Ok(())
}

pub async fn tables(conn: &dyn DbClient, format: OutputFormat) -> Result<()> {
    let stats = conn.table_stats().await?;
    output::print_list_output(&stats, format);
    Ok(())
}

pub async fn summary(conn: &dyn DbClient, format: OutputFormat) -> Result<()> {
    let sum = conn.portfolio_summary().await?;
    output::print_output(&sum, format);
    Ok(())
}

pub async fn trades(conn: &dyn DbClient, limit: i64, format: OutputFormat) -> Result<()> {
    let trades = conn.trade_history(limit).await?;
    output::print_list_output(&trades, format);
    Ok(())
}

pub async fn llm_logs(
    conn: &dyn DbClient,
    limit: i64,
    ticker: Option<&str>,
    format: OutputFormat,
) -> Result<()> {
    let logs = conn.list_llm_logs(limit, ticker).await?;
    output::print_list_output(&logs, format);
    Ok(())
}

pub async fn orders(
    conn: &dyn DbClient,
    limit: i64,
    status: Option<&str>,
    format: OutputFormat,
) -> Result<()> {
    let orders = conn.list_orders(limit, status).await?;
    output::print_list_output(&orders, format);
    Ok(())
}
