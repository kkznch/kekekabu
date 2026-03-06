use anyhow::Result;
use tokio_rusqlite::Connection;

async fn setup_db() -> Result<Connection> {
    let conn = Connection::open_in_memory().await?;
    keketrade::db::create_tables(&conn).await?;
    Ok(conn)
}

#[tokio::test]
async fn test_stock_save_and_lookup() -> Result<()> {
    let conn = setup_db().await?;

    let id = keketrade::db::save_stock(&conn, "7203", "Toyota", Some("Automobile")).await?;
    assert!(id > 0);

    let found = keketrade::db::get_stock_id(&conn, "7203").await?;
    assert_eq!(found, Some(id));

    let not_found = keketrade::db::get_stock_id(&conn, "9999").await?;
    assert_eq!(not_found, None);

    Ok(())
}

#[tokio::test]
async fn test_stock_upsert() -> Result<()> {
    let conn = setup_db().await?;

    let id1 = keketrade::db::save_stock(&conn, "7203", "Toyota", None).await?;
    let id2 = keketrade::db::save_stock(&conn, "7203", "Toyota Motor", Some("Automobile")).await?;
    assert_eq!(id1, id2);

    Ok(())
}

#[tokio::test]
async fn test_watchlist_crud() -> Result<()> {
    let conn = setup_db().await?;

    keketrade::db::watchlist_add(&conn, "7203", Some("test")).await?;
    keketrade::db::watchlist_add(&conn, "6758", None).await?;

    let items = keketrade::db::watchlist_list(&conn).await?;
    assert_eq!(items.len(), 2);

    keketrade::db::watchlist_remove(&conn, "7203").await?;

    let items = keketrade::db::watchlist_list(&conn).await?;
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].ticker, "6758");

    Ok(())
}

#[tokio::test]
async fn test_watchlist_idempotent() -> Result<()> {
    let conn = setup_db().await?;

    keketrade::db::watchlist_add(&conn, "7203", None).await?;
    keketrade::db::watchlist_add(&conn, "7203", None).await?;

    let items = keketrade::db::watchlist_list(&conn).await?;
    assert_eq!(items.len(), 1);

    Ok(())
}

#[tokio::test]
async fn test_save_and_fetch_prices() -> Result<()> {
    let conn = setup_db().await?;

    let stock_id = keketrade::db::save_stock(&conn, "7203", "Toyota", None).await?;

    let quotes = vec![
        keketrade::jquants::DailyQuote {
            code: "7203".to_string(),
            date: "2024-01-01".to_string(),
            open: Some(2000.0),
            high: Some(2100.0),
            low: Some(1950.0),
            close: Some(2050.0),
            volume: Some(1000000.0),
            adjustment_close: Some(2050.0),
        },
        keketrade::jquants::DailyQuote {
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

    keketrade::db::save_prices(&conn, stock_id, &quotes).await?;

    let price_data = keketrade::db::fetch_price_data(&conn, stock_id).await?;
    assert_eq!(price_data.closes.len(), 2);
    assert!((price_data.closes[0] - 2050.0).abs() < 0.01);
    assert!((price_data.closes[1] - 2100.0).abs() < 0.01);

    Ok(())
}

#[tokio::test]
async fn test_prices_idempotent() -> Result<()> {
    let conn = setup_db().await?;

    let stock_id = keketrade::db::save_stock(&conn, "7203", "Toyota", None).await?;

    let quotes = vec![keketrade::jquants::DailyQuote {
        code: "7203".to_string(),
        date: "2024-01-01".to_string(),
        open: Some(2000.0),
        high: Some(2100.0),
        low: Some(1950.0),
        close: Some(2050.0),
        volume: Some(1000000.0),
        adjustment_close: None,
    }];

    keketrade::db::save_prices(&conn, stock_id, &quotes).await?;
    keketrade::db::save_prices(&conn, stock_id, &quotes).await?;

    let price_data = keketrade::db::fetch_price_data(&conn, stock_id).await?;
    assert_eq!(price_data.closes.len(), 1);

    Ok(())
}

#[tokio::test]
async fn test_evaluation_save_and_list() -> Result<()> {
    let conn = setup_db().await?;

    let stock_id = keketrade::db::save_stock(&conn, "7203", "Toyota", None).await?;

    let eval_id = keketrade::db::save_evaluation(
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

    let evals = keketrade::db::list_evaluations(&conn, 10).await?;
    assert_eq!(evals.len(), 1);
    assert_eq!(evals[0].ticker, "7203");
    assert_eq!(evals[0].decision, "Buy");
    assert_eq!(evals[0].score, 75);

    Ok(())
}
