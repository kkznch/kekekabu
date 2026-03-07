use anyhow::Result;
use tokio_rusqlite::Connection;

async fn setup_db() -> Result<Connection> {
    let conn = Connection::open_in_memory().await?;
    kekekabu::db::create_tables(&conn).await?;
    Ok(conn)
}

#[tokio::test]
async fn test_stock_save_and_lookup() -> Result<()> {
    let conn = setup_db().await?;

    let id = kekekabu::db::save_stock(&conn, "7203", "Toyota", Some("Automobile")).await?;
    assert!(id > 0);

    let found = kekekabu::db::get_stock_id(&conn, "7203").await?;
    assert_eq!(found, Some(id));

    let not_found = kekekabu::db::get_stock_id(&conn, "9999").await?;
    assert_eq!(not_found, None);

    Ok(())
}

#[tokio::test]
async fn test_stock_upsert() -> Result<()> {
    let conn = setup_db().await?;

    let id1 = kekekabu::db::save_stock(&conn, "7203", "Toyota", None).await?;
    let id2 = kekekabu::db::save_stock(&conn, "7203", "Toyota Motor", Some("Automobile")).await?;
    assert_eq!(id1, id2);

    Ok(())
}

#[tokio::test]
async fn test_watchlist_crud() -> Result<()> {
    let conn = setup_db().await?;

    kekekabu::db::watchlist_add(&conn, "7203", Some("test")).await?;
    kekekabu::db::watchlist_add(&conn, "6758", None).await?;

    let items = kekekabu::db::watchlist_list(&conn).await?;
    assert_eq!(items.len(), 2);

    kekekabu::db::watchlist_remove(&conn, "7203").await?;

    let items = kekekabu::db::watchlist_list(&conn).await?;
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].ticker, "6758");

    Ok(())
}

#[tokio::test]
async fn test_watchlist_idempotent() -> Result<()> {
    let conn = setup_db().await?;

    kekekabu::db::watchlist_add(&conn, "7203", None).await?;
    kekekabu::db::watchlist_add(&conn, "7203", None).await?;

    let items = kekekabu::db::watchlist_list(&conn).await?;
    assert_eq!(items.len(), 1);

    Ok(())
}

#[tokio::test]
async fn test_save_and_fetch_prices() -> Result<()> {
    let conn = setup_db().await?;

    let stock_id = kekekabu::db::save_stock(&conn, "7203", "Toyota", None).await?;

    let quotes = vec![
        kekekabu::jquants::DailyQuote {
            code: "7203".to_string(),
            date: "2024-01-01".to_string(),
            open: Some(2000.0),
            high: Some(2100.0),
            low: Some(1950.0),
            close: Some(2050.0),
            volume: Some(1000000.0),
            adjustment_close: Some(2050.0),
        },
        kekekabu::jquants::DailyQuote {
            code: "7203".to_string(),
            date: "2024-01-02".to_string(),
            open: Some(2050.0),
            high: Some(2150.0),
            low: Some(2000.0),
            close: Some(2100.0),
            volume: Some(1200000.0),
            adjustment_close: Some(2100.0),
        },
    ];

    kekekabu::db::save_prices(&conn, stock_id, &quotes).await?;

    let price_data = kekekabu::db::fetch_price_data(&conn, stock_id).await?;
    assert_eq!(price_data.closes.len(), 2);
    assert!((price_data.closes[0] - 2050.0).abs() < 0.01);
    assert!((price_data.closes[1] - 2100.0).abs() < 0.01);

    Ok(())
}

#[tokio::test]
async fn test_prices_idempotent() -> Result<()> {
    let conn = setup_db().await?;

    let stock_id = kekekabu::db::save_stock(&conn, "7203", "Toyota", None).await?;

    let quotes = vec![kekekabu::jquants::DailyQuote {
        code: "7203".to_string(),
        date: "2024-01-01".to_string(),
        open: Some(2000.0),
        high: Some(2100.0),
        low: Some(1950.0),
        close: Some(2050.0),
        volume: Some(1000000.0),
        adjustment_close: None,
    }];

    kekekabu::db::save_prices(&conn, stock_id, &quotes).await?;
    kekekabu::db::save_prices(&conn, stock_id, &quotes).await?;

    let price_data = kekekabu::db::fetch_price_data(&conn, stock_id).await?;
    assert_eq!(price_data.closes.len(), 1);

    Ok(())
}

#[tokio::test]
async fn test_evaluation_save_and_list() -> Result<()> {
    let conn = setup_db().await?;

    let stock_id = kekekabu::db::save_stock(&conn, "7203", "Toyota", None).await?;

    let eval_id = kekekabu::db::save_evaluation(
        &conn,
        stock_id,
        "Buy",
        75,
        r#"{"summary":"Good","technical":"Bullish","risks":"None"}"#,
        Some(r#"{"RSI": 45.0}"#),
        None,
        Some("cli-claude"),
    )
    .await?;

    assert!(eval_id > 0);

    let evals = kekekabu::db::list_evaluations(&conn, 10).await?;
    assert_eq!(evals.len(), 1);
    assert_eq!(evals[0].ticker, "7203");
    assert_eq!(evals[0].decision, "Buy");
    assert_eq!(evals[0].score, 75);

    Ok(())
}

#[tokio::test]
async fn test_trade_cash_summary_empty() -> Result<()> {
    let conn = setup_db().await?;
    let summary = kekekabu::db::trade_cash_summary(&conn).await?;
    assert!((summary.total_invested - 0.0).abs() < 0.01);
    assert!((summary.total_recovered - 0.0).abs() < 0.01);
    Ok(())
}

#[tokio::test]
async fn test_trade_cash_summary_buy_only() -> Result<()> {
    let conn = setup_db().await?;
    kekekabu::portfolio::buy(
        &conn, "7203",
        rust_decimal::Decimal::from(100),
        rust_decimal::Decimal::from(2000),
        None,
    ).await?;
    let summary = kekekabu::db::trade_cash_summary(&conn).await?;
    assert!((summary.total_invested - 200000.0).abs() < 0.01);
    assert!((summary.total_recovered - 0.0).abs() < 0.01);
    Ok(())
}

#[tokio::test]
async fn test_trade_cash_summary_buy_and_sell() -> Result<()> {
    let conn = setup_db().await?;
    kekekabu::portfolio::buy(
        &conn, "7203",
        rust_decimal::Decimal::from(100),
        rust_decimal::Decimal::from(2000),
        None,
    ).await?;
    kekekabu::portfolio::sell(
        &conn, "7203",
        rust_decimal::Decimal::from(50),
        rust_decimal::Decimal::from(2200),
        None,
    ).await?;
    let summary = kekekabu::db::trade_cash_summary(&conn).await?;
    assert!((summary.total_invested - 200000.0).abs() < 0.01);
    assert!((summary.total_recovered - 110000.0).abs() < 0.01);
    Ok(())
}

#[tokio::test]
async fn test_save_watchlist_event() -> Result<()> {
    let conn = setup_db().await?;
    kekekabu::db::save_watchlist_event(&conn, "7203", "add", Some("割安銘柄")).await?;
    kekekabu::db::save_watchlist_event(&conn, "6758", "keep", Some("継続監視")).await?;
    kekekabu::db::save_watchlist_event(&conn, "9984", "remove", Some("基準外")).await?;

    // Verify events were saved by querying directly
    let count: i64 = conn
        .call(|conn| {
            let count = conn.query_row(
                "SELECT COUNT(*) FROM watchlist_events",
                [],
                |row| row.get(0),
            )?;
            Ok::<i64, rusqlite::Error>(count)
        })
        .await?;
    assert_eq!(count, 3);
    Ok(())
}

#[tokio::test]
async fn test_save_watchlist_event_without_reason() -> Result<()> {
    let conn = setup_db().await?;
    kekekabu::db::save_watchlist_event(&conn, "7203", "add", None).await?;

    let count: i64 = conn
        .call(|conn| {
            let count = conn.query_row(
                "SELECT COUNT(*) FROM watchlist_events WHERE reason IS NULL",
                [],
                |row| row.get(0),
            )?;
            Ok::<i64, rusqlite::Error>(count)
        })
        .await?;
    assert_eq!(count, 1);
    Ok(())
}

#[tokio::test]
async fn test_list_watchlist_events() -> Result<()> {
    let conn = setup_db().await?;
    kekekabu::db::save_watchlist_event(&conn, "7203", "add", Some("割安")).await?;
    kekekabu::db::save_watchlist_event(&conn, "6758", "add", Some("成長")).await?;
    kekekabu::db::save_watchlist_event(&conn, "7203", "keep", Some("継続")).await?;

    let all = kekekabu::db::list_watchlist_events(&conn, None).await?;
    assert_eq!(all.len(), 3);

    let filtered = kekekabu::db::list_watchlist_events(&conn, Some("7203")).await?;
    assert_eq!(filtered.len(), 2);
    assert!(filtered.iter().all(|e| e.ticker == "7203"));

    Ok(())
}

#[tokio::test]
async fn test_list_stocks() -> Result<()> {
    let conn = setup_db().await?;
    kekekabu::db::save_stock(&conn, "7203", "Toyota", Some("Automobile")).await?;
    kekekabu::db::save_stock(&conn, "6758", "Sony", None).await?;

    let stocks = kekekabu::db::list_stocks(&conn).await?;
    assert_eq!(stocks.len(), 2);
    assert_eq!(stocks[0].ticker, "6758"); // sorted by ticker ASC
    assert_eq!(stocks[1].ticker, "7203");
    Ok(())
}

#[tokio::test]
async fn test_table_stats() -> Result<()> {
    let conn = setup_db().await?;
    kekekabu::db::save_stock(&conn, "7203", "Toyota", None).await?;
    kekekabu::db::watchlist_add(&conn, "7203", None).await?;

    let stats = kekekabu::db::table_stats(&conn).await?;
    assert_eq!(stats.len(), 8);

    let stocks_stat = stats.iter().find(|s| s.table_name == "stocks").unwrap();
    assert_eq!(stocks_stat.row_count, 1);

    let watchlist_stat = stats.iter().find(|s| s.table_name == "watchlist").unwrap();
    assert_eq!(watchlist_stat.row_count, 1);
    Ok(())
}
