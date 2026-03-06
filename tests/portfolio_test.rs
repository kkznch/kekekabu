use anyhow::Result;
use rust_decimal::Decimal;
use std::str::FromStr;
use tokio_rusqlite::Connection;

async fn setup_db() -> Result<Connection> {
    let conn = Connection::open_in_memory().await?;
    keketrade::db::create_tables(&conn).await?;
    Ok(conn)
}

#[tokio::test]
async fn test_buy_creates_position() -> Result<()> {
    let conn = setup_db().await?;

    let qty = Decimal::from_str("100").unwrap();
    let price = Decimal::from_str("2000").unwrap();

    keketrade::portfolio::buy(&conn, "7203", qty, price, Some("test")).await?;

    let positions = keketrade::portfolio::list_positions(&conn).await?;
    assert_eq!(positions.len(), 1);
    assert_eq!(positions[0].ticker, "7203");
    assert_eq!(positions[0].quantity, qty);
    assert_eq!(positions[0].avg_cost, price);

    Ok(())
}

#[tokio::test]
async fn test_buy_additional_updates_avg_cost() -> Result<()> {
    let conn = setup_db().await?;

    keketrade::portfolio::buy(
        &conn,
        "7203",
        Decimal::from_str("100").unwrap(),
        Decimal::from_str("2000").unwrap(),
        None,
    )
    .await?;

    keketrade::portfolio::buy(
        &conn,
        "7203",
        Decimal::from_str("100").unwrap(),
        Decimal::from_str("2200").unwrap(),
        None,
    )
    .await?;

    let positions = keketrade::portfolio::list_positions(&conn).await?;
    assert_eq!(positions.len(), 1);
    assert_eq!(positions[0].quantity, Decimal::from_str("200").unwrap());
    assert_eq!(positions[0].avg_cost, Decimal::from_str("2100").unwrap());

    Ok(())
}

#[tokio::test]
async fn test_sell_partial() -> Result<()> {
    let conn = setup_db().await?;

    keketrade::portfolio::buy(
        &conn,
        "7203",
        Decimal::from_str("100").unwrap(),
        Decimal::from_str("2000").unwrap(),
        None,
    )
    .await?;

    keketrade::portfolio::sell(
        &conn,
        "7203",
        Decimal::from_str("50").unwrap(),
        Decimal::from_str("2200").unwrap(),
        None,
    )
    .await?;

    let positions = keketrade::portfolio::list_positions(&conn).await?;
    assert_eq!(positions.len(), 1);
    assert_eq!(positions[0].quantity, Decimal::from_str("50").unwrap());

    // Check trade history has PnL
    let trades = keketrade::portfolio::trade_history(&conn, 10).await?;
    assert_eq!(trades.len(), 2); // buy + sell
    let sell_trade = trades.iter().find(|t| t.side == "sell").unwrap();
    // PnL = (2200 - 2000) * 50 = 10000
    assert_eq!(sell_trade.pnl, Some(Decimal::from_str("10000").unwrap()));

    Ok(())
}

#[tokio::test]
async fn test_sell_all_closes_position() -> Result<()> {
    let conn = setup_db().await?;

    keketrade::portfolio::buy(
        &conn,
        "7203",
        Decimal::from_str("100").unwrap(),
        Decimal::from_str("2000").unwrap(),
        None,
    )
    .await?;

    keketrade::portfolio::sell(
        &conn,
        "7203",
        Decimal::from_str("100").unwrap(),
        Decimal::from_str("2200").unwrap(),
        None,
    )
    .await?;

    let positions = keketrade::portfolio::list_positions(&conn).await?;
    assert_eq!(positions.len(), 0); // is_active = 0

    Ok(())
}

#[tokio::test]
async fn test_portfolio_summary() -> Result<()> {
    let conn = setup_db().await?;

    keketrade::portfolio::buy(
        &conn,
        "7203",
        Decimal::from_str("100").unwrap(),
        Decimal::from_str("2000").unwrap(),
        None,
    )
    .await?;

    keketrade::portfolio::buy(
        &conn,
        "6758",
        Decimal::from_str("50").unwrap(),
        Decimal::from_str("3000").unwrap(),
        None,
    )
    .await?;

    let summary = keketrade::portfolio::summary(&conn).await?;
    assert_eq!(summary.position_count, 2);
    // 100*2000 + 50*3000 = 350000
    assert_eq!(
        summary.total_invested,
        Decimal::from_str("350000").unwrap()
    );

    Ok(())
}
