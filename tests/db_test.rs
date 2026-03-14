use anyhow::Result;
use kekekabu::db::{DbClient, SqliteClient};

async fn setup_db() -> Result<SqliteClient> {
    SqliteClient::open_in_memory().await
}

#[tokio::test]
async fn test_stock_save_and_lookup() -> Result<()> {
    let db = setup_db().await?;

    let id = db.save_stock("7203", "Toyota", Some("Automobile")).await?;
    assert!(id > 0);

    let found = db.get_stock_id("7203").await?;
    assert_eq!(found, Some(id));

    let not_found = db.get_stock_id("9999").await?;
    assert_eq!(not_found, None);

    Ok(())
}

#[tokio::test]
async fn test_stock_upsert() -> Result<()> {
    let db = setup_db().await?;

    let id1 = db.save_stock("7203", "Toyota", None).await?;
    let id2 = db
        .save_stock("7203", "Toyota Motor", Some("Automobile"))
        .await?;
    assert_eq!(id1, id2);

    Ok(())
}

#[tokio::test]
async fn test_watchlist_crud() -> Result<()> {
    let db = setup_db().await?;

    db.watchlist_add("7203", Some("test")).await?;
    db.watchlist_add("6758", None).await?;

    let items = db.watchlist_list().await?;
    assert_eq!(items.len(), 2);

    db.watchlist_remove("7203").await?;

    let items = db.watchlist_list().await?;
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].ticker, "6758");

    Ok(())
}

#[tokio::test]
async fn test_watchlist_idempotent() -> Result<()> {
    let db = setup_db().await?;

    db.watchlist_add("7203", None).await?;
    db.watchlist_add("7203", None).await?;

    let items = db.watchlist_list().await?;
    assert_eq!(items.len(), 1);

    Ok(())
}

#[tokio::test]
async fn test_save_and_fetch_prices() -> Result<()> {
    let db = setup_db().await?;

    let stock_id = db.save_stock("7203", "Toyota", None).await?;

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

    db.save_prices(stock_id, &quotes).await?;

    let price_data = db.fetch_price_data(stock_id).await?;
    assert_eq!(price_data.closes.len(), 2);
    assert!((price_data.closes[0] - 2050.0).abs() < 0.01);
    assert!((price_data.closes[1] - 2100.0).abs() < 0.01);

    Ok(())
}

#[tokio::test]
async fn test_prices_idempotent() -> Result<()> {
    let db = setup_db().await?;

    let stock_id = db.save_stock("7203", "Toyota", None).await?;

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

    db.save_prices(stock_id, &quotes).await?;
    db.save_prices(stock_id, &quotes).await?;

    let price_data = db.fetch_price_data(stock_id).await?;
    assert_eq!(price_data.closes.len(), 1);

    Ok(())
}

#[tokio::test]
async fn test_evaluation_save_and_list() -> Result<()> {
    let db = setup_db().await?;

    let stock_id = db.save_stock("7203", "Toyota", None).await?;

    let eval_id = db
        .save_evaluation(
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

    let evals = db.list_evaluations(10).await?;
    assert_eq!(evals.len(), 1);
    assert_eq!(evals[0].ticker, "7203");
    assert_eq!(evals[0].decision, "Buy");
    assert_eq!(evals[0].score, 75);

    Ok(())
}

#[tokio::test]
async fn test_trade_cash_summary_empty() -> Result<()> {
    let db = setup_db().await?;
    let summary = db.trade_cash_summary().await?;
    assert!((summary.total_invested - 0.0).abs() < 0.01);
    assert!((summary.total_recovered - 0.0).abs() < 0.01);
    Ok(())
}

#[tokio::test]
async fn test_trade_cash_summary_buy_only() -> Result<()> {
    let db = setup_db().await?;
    db.portfolio_buy(
        "7203",
        rust_decimal::Decimal::from(100),
        rust_decimal::Decimal::from(2000),
        None,
    )
    .await?;
    let summary = db.trade_cash_summary().await?;
    assert!((summary.total_invested - 200000.0).abs() < 0.01);
    assert!((summary.total_recovered - 0.0).abs() < 0.01);
    Ok(())
}

#[tokio::test]
async fn test_trade_cash_summary_buy_and_sell() -> Result<()> {
    let db = setup_db().await?;
    db.portfolio_buy(
        "7203",
        rust_decimal::Decimal::from(100),
        rust_decimal::Decimal::from(2000),
        None,
    )
    .await?;
    db.portfolio_sell(
        "7203",
        rust_decimal::Decimal::from(50),
        rust_decimal::Decimal::from(2200),
        None,
    )
    .await?;
    let summary = db.trade_cash_summary().await?;
    assert!((summary.total_invested - 200000.0).abs() < 0.01);
    assert!((summary.total_recovered - 110000.0).abs() < 0.01);
    Ok(())
}

#[tokio::test]
async fn test_save_watchlist_event() -> Result<()> {
    let db = setup_db().await?;
    db.save_watchlist_event("7203", "add", Some("割安銘柄"))
        .await?;
    db.save_watchlist_event("6758", "keep", Some("継続監視"))
        .await?;
    db.save_watchlist_event("9984", "remove", Some("基準外"))
        .await?;

    // Verify events were saved by querying directly
    let count: i64 = db
        .conn()
        .call(|conn| {
            let count = conn.query_row("SELECT COUNT(*) FROM watchlist_events", [], |row| {
                row.get(0)
            })?;
            Ok::<i64, rusqlite::Error>(count)
        })
        .await?;
    assert_eq!(count, 3);
    Ok(())
}

#[tokio::test]
async fn test_save_watchlist_event_without_reason() -> Result<()> {
    let db = setup_db().await?;
    db.save_watchlist_event("7203", "add", None).await?;

    let count: i64 = db
        .conn()
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
    let db = setup_db().await?;
    db.save_watchlist_event("7203", "add", Some("割安")).await?;
    db.save_watchlist_event("6758", "add", Some("成長")).await?;
    db.save_watchlist_event("7203", "keep", Some("継続"))
        .await?;

    let all = db.list_watchlist_events(None).await?;
    assert_eq!(all.len(), 3);

    let filtered = db.list_watchlist_events(Some("7203")).await?;
    assert_eq!(filtered.len(), 2);
    assert!(filtered.iter().all(|e| e.ticker == "7203"));

    Ok(())
}

#[tokio::test]
async fn test_list_stocks() -> Result<()> {
    let db = setup_db().await?;
    db.save_stock("7203", "Toyota", Some("Automobile")).await?;
    db.save_stock("6758", "Sony", None).await?;

    let stocks = db.list_stocks().await?;
    assert_eq!(stocks.len(), 2);
    assert_eq!(stocks[0].ticker, "6758"); // sorted by ticker ASC
    assert_eq!(stocks[1].ticker, "7203");
    Ok(())
}

#[tokio::test]
async fn test_save_stocks_bulk_empty() -> Result<()> {
    let db = setup_db().await?;
    let count = db.save_stocks_bulk(&[]).await?;
    assert_eq!(count, 0);
    Ok(())
}

#[tokio::test]
async fn test_save_stocks_bulk_multiple() -> Result<()> {
    let db = setup_db().await?;
    let stocks = vec![
        kekekabu::jquants::ListedInfo {
            code: "7203".to_string(),
            company_name: "Toyota Motor".to_string(),
            sector: Some("Automobile".to_string()),
        },
        kekekabu::jquants::ListedInfo {
            code: "6758".to_string(),
            company_name: "Sony Group".to_string(),
            sector: Some("Electronics".to_string()),
        },
    ];
    let count = db.save_stocks_bulk(&stocks).await?;
    assert_eq!(count, 2);

    let all = db.list_stocks().await?;
    assert_eq!(all.len(), 2);
    assert_eq!(all[0].ticker, "6758");
    assert_eq!(all[0].name, "Sony Group");
    Ok(())
}

#[tokio::test]
async fn test_save_stocks_bulk_upsert() -> Result<()> {
    let db = setup_db().await?;
    // Insert initial
    db.save_stock("7203", "Toyota", Some("Auto")).await?;

    // Bulk upsert with updated name
    let stocks = vec![kekekabu::jquants::ListedInfo {
        code: "7203".to_string(),
        company_name: "Toyota Motor Corporation".to_string(),
        sector: Some("Automobile".to_string()),
    }];
    db.save_stocks_bulk(&stocks).await?;

    let all = db.list_stocks().await?;
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].name, "Toyota Motor Corporation");
    assert_eq!(all[0].sector.as_deref(), Some("Automobile"));
    Ok(())
}

#[tokio::test]
async fn test_has_any_stocks_empty() -> Result<()> {
    let db = setup_db().await?;
    assert!(!db.has_any_stocks().await?);
    Ok(())
}

#[tokio::test]
async fn test_has_any_stocks_with_data() -> Result<()> {
    let db = setup_db().await?;
    db.save_stock("7203", "Toyota", None).await?;
    assert!(db.has_any_stocks().await?);
    Ok(())
}

#[tokio::test]
async fn test_recent_evaluations_by_stock_empty() -> Result<()> {
    let db = setup_db().await?;
    let stock_id = db.save_stock("7203", "Toyota", None).await?;

    let evals = db.get_recent_evaluations_by_stock(stock_id, 3).await?;
    assert!(evals.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_recent_evaluations_by_stock_partial() -> Result<()> {
    let db = setup_db().await?;
    let stock_id = db.save_stock("7203", "Toyota", None).await?;

    db.save_evaluation(stock_id, "Buy", 72, "Good catalyst", None, None, None)
        .await?;
    db.save_evaluation(stock_id, "Avoid", 35, "High risk", None, None, None)
        .await?;

    let evals = db.get_recent_evaluations_by_stock(stock_id, 3).await?;
    assert_eq!(evals.len(), 2);
    // Most recent first
    assert_eq!(evals[0].decision, "Avoid");
    assert_eq!(evals[1].decision, "Buy");
    Ok(())
}

#[tokio::test]
async fn test_recent_evaluations_by_stock_limited() -> Result<()> {
    let db = setup_db().await?;
    let stock_id = db.save_stock("7203", "Toyota", None).await?;

    for (decision, score) in [
        ("Buy", 72),
        ("Avoid", 35),
        ("Buy", 68),
        ("Hold", 55),
        ("Sell", 25),
    ] {
        db.save_evaluation(stock_id, decision, score, "rationale", None, None, None)
            .await?;
    }

    let evals = db.get_recent_evaluations_by_stock(stock_id, 3).await?;
    assert_eq!(evals.len(), 3);
    // Most recent 3 only
    assert_eq!(evals[0].decision, "Sell");
    assert_eq!(evals[1].decision, "Hold");
    assert_eq!(evals[2].decision, "Buy");
    Ok(())
}

#[tokio::test]
async fn test_recent_evaluations_by_stock_filters_by_stock() -> Result<()> {
    let db = setup_db().await?;
    let id1 = db.save_stock("7203", "Toyota", None).await?;
    let id2 = db.save_stock("6758", "Sony", None).await?;

    db.save_evaluation(id1, "Buy", 72, "Toyota eval", None, None, None)
        .await?;
    db.save_evaluation(id2, "Avoid", 30, "Sony eval", None, None, None)
        .await?;

    let evals = db.get_recent_evaluations_by_stock(id1, 3).await?;
    assert_eq!(evals.len(), 1);
    assert_eq!(evals[0].ticker, "7203");
    Ok(())
}

#[tokio::test]
async fn test_table_stats() -> Result<()> {
    let db = setup_db().await?;
    db.save_stock("7203", "Toyota", None).await?;
    db.watchlist_add("7203", None).await?;

    let stats = db.table_stats().await?;
    assert_eq!(stats.len(), 10);

    let stocks_stat = stats.iter().find(|s| s.table_name == "stocks").unwrap();
    assert_eq!(stocks_stat.row_count, 1);

    let watchlist_stat = stats.iter().find(|s| s.table_name == "watchlist").unwrap();
    assert_eq!(watchlist_stat.row_count, 1);
    Ok(())
}

#[tokio::test]
async fn test_save_and_list_llm_logs() -> Result<()> {
    let db = setup_db().await?;

    db.save_llm_log(
        "eval",
        Some("7203"),
        "api-anthropic",
        None,
        Some(0.0),
        "Evaluate this stock",
        r#"{"decision": "Buy"}"#,
    )
    .await?;

    db.save_llm_log(
        "fetch",
        Some("6758"),
        "cli-gemini",
        None,
        None,
        "Fetch news",
        r#"{"items": []}"#,
    )
    .await?;

    let all_logs = db.list_llm_logs(10, None).await?;
    assert_eq!(all_logs.len(), 2);
    // Most recent first
    assert_eq!(all_logs[0].command, "fetch");
    assert_eq!(all_logs[1].command, "eval");

    // Filter by ticker
    let toyota_logs = db.list_llm_logs(10, Some("7203")).await?;
    assert_eq!(toyota_logs.len(), 1);
    assert_eq!(toyota_logs[0].ticker.as_deref(), Some("7203"));
    assert_eq!(toyota_logs[0].temperature, Some(0.0));

    // Limit
    let limited = db.list_llm_logs(1, None).await?;
    assert_eq!(limited.len(), 1);

    Ok(())
}

#[tokio::test]
async fn test_save_llm_log_without_ticker() -> Result<()> {
    let db = setup_db().await?;

    db.save_llm_log(
        "discover",
        None,
        "api-anthropic",
        None,
        None,
        "Discover stocks",
        r#"{"add": [], "remove": [], "keep": []}"#,
    )
    .await?;

    let logs = db.list_llm_logs(10, None).await?;
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].command, "discover");
    assert!(logs[0].ticker.is_none());
    assert!(logs[0].temperature.is_none());
    Ok(())
}

// ─── Orders tests ────────────────────────────────────────────────────

#[tokio::test]
async fn test_order_save_and_list() -> Result<()> {
    let db = setup_db().await?;
    let stock_id = db.save_stock("7203", "Toyota", None).await?;

    let eval_id = db
        .save_evaluation(stock_id, "Buy", 75, "Good", None, None, None)
        .await?;

    let order_id = db
        .save_order(
            stock_id,
            "buy",
            "limit",
            "2500",
            "100",
            "2026-03-13-7203-buy-1",
            Some(eval_id),
        )
        .await?;
    assert!(order_id > 0);

    let orders = db.list_orders(10, None).await?;
    assert_eq!(orders.len(), 1);
    assert_eq!(orders[0].ticker, "7203");
    assert_eq!(orders[0].side, "buy");
    assert_eq!(orders[0].price, "2500");
    assert_eq!(orders[0].quantity, "100");
    assert_eq!(orders[0].status, "pending");
    assert_eq!(orders[0].evaluation_id, Some(eval_id));

    Ok(())
}

#[tokio::test]
async fn test_order_update_status() -> Result<()> {
    let db = setup_db().await?;
    let stock_id = db.save_stock("7203", "Toyota", None).await?;

    let order_id = db
        .save_order(stock_id, "buy", "limit", "2500", "100", "req-1", None)
        .await?;

    db.update_order_status(
        order_id,
        "filled",
        Some("T12345"),
        Some("2500"),
        Some("100"),
        Some("2026-03-13 10:00:00"),
    )
    .await?;

    let orders = db.list_orders(10, Some("filled")).await?;
    assert_eq!(orders.len(), 1);
    assert_eq!(orders[0].status, "filled");
    assert_eq!(orders[0].tachibana_order_id.as_deref(), Some("T12345"));
    assert_eq!(orders[0].filled_price.as_deref(), Some("2500"));
    assert_eq!(orders[0].filled_quantity.as_deref(), Some("100"));

    Ok(())
}

#[tokio::test]
async fn test_order_list_pending() -> Result<()> {
    let db = setup_db().await?;
    let stock_id = db.save_stock("7203", "Toyota", None).await?;

    let id1 = db
        .save_order(stock_id, "buy", "limit", "2500", "100", "req-1", None)
        .await?;
    db.save_order(stock_id, "sell", "limit", "2600", "50", "req-2", None)
        .await?;

    // Fill one
    db.update_order_status(id1, "filled", None, None, None, None)
        .await?;

    let pending = db.list_pending_orders().await?;
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].side, "sell");
    assert_eq!(pending[0].request_id, "req-2");

    Ok(())
}

#[tokio::test]
async fn test_order_request_id_idempotent() -> Result<()> {
    let db = setup_db().await?;
    let stock_id = db.save_stock("7203", "Toyota", None).await?;

    let id1 = db
        .save_order(stock_id, "buy", "limit", "2500", "100", "req-dup", None)
        .await?;
    let id2 = db
        .save_order(stock_id, "buy", "limit", "2500", "100", "req-dup", None)
        .await?;

    assert_eq!(id1, id2, "Duplicate request_id should return same order id");

    let orders = db.list_orders(10, None).await?;
    assert_eq!(orders.len(), 1);

    Ok(())
}

#[tokio::test]
async fn test_order_exists_for_evaluation() -> Result<()> {
    let db = setup_db().await?;
    let stock_id = db.save_stock("7203", "Toyota", None).await?;
    let eval_id = db
        .save_evaluation(stock_id, "Buy", 75, "Good", None, None, None)
        .await?;

    assert!(!db.order_exists_for_evaluation(eval_id, "buy").await?);

    db.save_order(
        stock_id,
        "buy",
        "limit",
        "2500",
        "100",
        "req-eval",
        Some(eval_id),
    )
    .await?;

    assert!(db.order_exists_for_evaluation(eval_id, "buy").await?);
    assert!(!db.order_exists_for_evaluation(eval_id, "sell").await?);

    Ok(())
}
