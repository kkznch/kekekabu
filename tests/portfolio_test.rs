use anyhow::Result;
use kekekabu::db::{DbClient, SqliteClient};
use rust_decimal::Decimal;
use std::str::FromStr;

async fn setup_db() -> Result<SqliteClient> {
    SqliteClient::open_in_memory().await
}

#[tokio::test]
async fn test_buy_creates_position() -> Result<()> {
    let db = setup_db().await?;

    let qty = Decimal::from_str("100").unwrap();
    let price = Decimal::from_str("2000").unwrap();

    db.portfolio_buy("7203", qty, price, Some("test")).await?;

    let positions = db.list_positions().await?;
    assert_eq!(positions.len(), 1);
    assert_eq!(positions[0].ticker, "7203");
    assert_eq!(positions[0].quantity, qty);
    assert_eq!(positions[0].avg_cost, price);

    Ok(())
}

#[tokio::test]
async fn test_buy_additional_updates_avg_cost() -> Result<()> {
    let db = setup_db().await?;

    db.portfolio_buy(
        "7203",
        Decimal::from_str("100").unwrap(),
        Decimal::from_str("2000").unwrap(),
        None,
    )
    .await?;

    db.portfolio_buy(
        "7203",
        Decimal::from_str("100").unwrap(),
        Decimal::from_str("2200").unwrap(),
        None,
    )
    .await?;

    let positions = db.list_positions().await?;
    assert_eq!(positions.len(), 1);
    assert_eq!(positions[0].quantity, Decimal::from_str("200").unwrap());
    assert_eq!(positions[0].avg_cost, Decimal::from_str("2100").unwrap());

    Ok(())
}

#[tokio::test]
async fn test_sell_partial() -> Result<()> {
    let db = setup_db().await?;

    db.portfolio_buy(
        "7203",
        Decimal::from_str("100").unwrap(),
        Decimal::from_str("2000").unwrap(),
        None,
    )
    .await?;

    db.portfolio_sell(
        "7203",
        Decimal::from_str("50").unwrap(),
        Decimal::from_str("2200").unwrap(),
        None,
    )
    .await?;

    let positions = db.list_positions().await?;
    assert_eq!(positions.len(), 1);
    assert_eq!(positions[0].quantity, Decimal::from_str("50").unwrap());

    // Check trade history has PnL
    let trades = db.trade_history(10).await?;
    assert_eq!(trades.len(), 2); // buy + sell
    let sell_trade = trades.iter().find(|t| t.side == "sell").unwrap();
    // PnL = (2200 - 2000) * 50 = 10000
    assert_eq!(sell_trade.pnl, Some(Decimal::from_str("10000").unwrap()));

    Ok(())
}

#[tokio::test]
async fn test_sell_all_closes_position() -> Result<()> {
    let db = setup_db().await?;

    db.portfolio_buy(
        "7203",
        Decimal::from_str("100").unwrap(),
        Decimal::from_str("2000").unwrap(),
        None,
    )
    .await?;

    db.portfolio_sell(
        "7203",
        Decimal::from_str("100").unwrap(),
        Decimal::from_str("2200").unwrap(),
        None,
    )
    .await?;

    let positions = db.list_positions().await?;
    assert_eq!(positions.len(), 0); // is_active = 0

    Ok(())
}

#[tokio::test]
async fn test_sell_all_removes_from_watchlist() -> Result<()> {
    let db = setup_db().await?;

    // Add to watchlist first
    db.watchlist_add("7203", Some("test")).await?;
    let items = db.watchlist_list().await?;
    assert_eq!(items.len(), 1);

    // Buy and then sell all
    db.portfolio_buy(
        "7203",
        Decimal::from_str("100").unwrap(),
        Decimal::from_str("2000").unwrap(),
        None,
    )
    .await?;

    db.portfolio_sell(
        "7203",
        Decimal::from_str("100").unwrap(),
        Decimal::from_str("2200").unwrap(),
        None,
    )
    .await?;

    // Watchlist should be empty
    let items = db.watchlist_list().await?;
    assert_eq!(items.len(), 0);

    // Watchlist event should be recorded
    let events = db.list_watchlist_events(Some("7203")).await?;
    let auto_removed = events.iter().find(|e| e.action == "auto-removed-on-sell");
    assert!(auto_removed.is_some());
    assert_eq!(
        auto_removed.unwrap().reason.as_deref(),
        Some("Position closed")
    );

    Ok(())
}

#[tokio::test]
async fn test_partial_sell_keeps_watchlist() -> Result<()> {
    let db = setup_db().await?;

    // Add to watchlist
    db.watchlist_add("7203", Some("test")).await?;

    // Buy and partial sell
    db.portfolio_buy(
        "7203",
        Decimal::from_str("100").unwrap(),
        Decimal::from_str("2000").unwrap(),
        None,
    )
    .await?;

    db.portfolio_sell(
        "7203",
        Decimal::from_str("50").unwrap(),
        Decimal::from_str("2200").unwrap(),
        None,
    )
    .await?;

    // Watchlist should still have the stock
    let items = db.watchlist_list().await?;
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].ticker, "7203");

    // No auto-removed-on-sell event
    let events = db.list_watchlist_events(Some("7203")).await?;
    assert!(!events.iter().any(|e| e.action == "auto-removed-on-sell"));

    Ok(())
}

#[tokio::test]
async fn test_sell_all_no_watchlist_entry_no_error() -> Result<()> {
    let db = setup_db().await?;

    // Buy without adding to watchlist
    db.portfolio_buy(
        "7203",
        Decimal::from_str("100").unwrap(),
        Decimal::from_str("2000").unwrap(),
        None,
    )
    .await?;

    // Sell all — should not error even though not in watchlist
    db.portfolio_sell(
        "7203",
        Decimal::from_str("100").unwrap(),
        Decimal::from_str("2200").unwrap(),
        None,
    )
    .await?;

    let positions = db.list_positions().await?;
    assert_eq!(positions.len(), 0);

    // Event should still be recorded
    let events = db.list_watchlist_events(Some("7203")).await?;
    assert!(events.iter().any(|e| e.action == "auto-removed-on-sell"));

    Ok(())
}

#[tokio::test]
async fn test_portfolio_summary() -> Result<()> {
    let db = setup_db().await?;

    db.portfolio_buy(
        "7203",
        Decimal::from_str("100").unwrap(),
        Decimal::from_str("2000").unwrap(),
        None,
    )
    .await?;

    db.portfolio_buy(
        "6758",
        Decimal::from_str("50").unwrap(),
        Decimal::from_str("3000").unwrap(),
        None,
    )
    .await?;

    let summary = db.portfolio_summary().await?;
    assert_eq!(summary.position_count, 2);
    // 100*2000 + 50*3000 = 350000
    assert_eq!(summary.total_invested, Decimal::from_str("350000").unwrap());

    Ok(())
}
