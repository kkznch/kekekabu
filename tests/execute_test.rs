use anyhow::Result;
use async_trait::async_trait;
use kekekabu::cmd::execute;
use kekekabu::config::AppConfig;
use kekekabu::db::{DbClient, SqliteClient};
use kekekabu::spec::InvestmentSpec;
use kekekabu::tachibana::{BrokerClient, Side, event::FillNotification, order};
use rust_decimal::Decimal;
use std::sync::Mutex;

async fn setup_db() -> Result<SqliteClient> {
    SqliteClient::open_in_memory().await
}

fn make_spec(toml_str: &str) -> InvestmentSpec {
    InvestmentSpec::from_str_for_test(toml_str)
}

// ─── Mock broker ─────────────────────────────────────────────────────

struct MockBrokerClient {
    orders: Mutex<Vec<MockOrder>>,
}

struct MockOrder {
    side: String,
    ticker: String,
    price: String,
    quantity: String,
}

impl MockBrokerClient {
    fn new() -> Self {
        Self {
            orders: Mutex::new(Vec::new()),
        }
    }

    fn placed_orders(&self) -> Vec<String> {
        self.orders
            .lock()
            .unwrap()
            .iter()
            .map(|o| format!("{} {} x{} @{}", o.side, o.ticker, o.quantity, o.price))
            .collect()
    }
}

#[async_trait]
impl BrokerClient for MockBrokerClient {
    async fn ensure_logged_in(&mut self) -> Result<()> {
        Ok(())
    }

    async fn place_order(
        &self,
        side: Side,
        ticker: &str,
        price: &str,
        quantity: &str,
    ) -> Result<order::NewOrderResult> {
        self.orders.lock().unwrap().push(MockOrder {
            side: side.as_str().to_string(),
            ticker: ticker.to_string(),
            price: price.to_string(),
            quantity: quantity.to_string(),
        });
        Ok(order::NewOrderResult {
            order_number: format!("MOCK-{}-{}", ticker, side),
            result_text: "OK".to_string(),
        })
    }

    async fn query_order(&self, _order_number: &str) -> Result<order::OrderDetail> {
        anyhow::bail!("mock: no orders to query")
    }

    async fn wait_for_fills(
        &self,
        _pending_order_numbers: &[String],
    ) -> Result<Vec<FillNotification>> {
        Ok(vec![])
    }

    async fn logout(&mut self) -> Result<()> {
        Ok(())
    }
}

// ─── Execute dry-run tests ───────────────────────────────────────────

#[tokio::test]
async fn test_execute_dry_run_no_evaluations() -> Result<()> {
    let db = setup_db().await?;
    let config = AppConfig::default();
    let spec = make_spec(
        r#"
name = "Test"
[execution]
stop_loss = -0.07
max_position_size = 0.05
[budget]
initial_cash = 300000
"#,
    );

    let result = execute::run(&db, &config, &spec, None, true).await?;

    assert!(result.actions.is_empty());
    assert!(!result.circuit_breaker_triggered);
    assert!(result.hard_stop_loss_actions.is_empty());
    assert!(result.order_results.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_execute_dry_run_with_buy_signal() -> Result<()> {
    let db = setup_db().await?;
    let config = AppConfig::default();
    let spec = make_spec(
        r#"
name = "Test"
[execution]
stop_loss = -0.07
"#,
    );

    let stock_id = db.save_stock("7203", "Toyota", Some("Auto")).await?;
    db.watchlist_add("7203", Some("test")).await?;
    save_price(&db, stock_id, "2025-01-01", 2500.0).await?;
    db.save_evaluation(stock_id, "Buy", 80, "Strong buy", None, None, None)
        .await?;

    let result = execute::run(&db, &config, &spec, None, true).await?;

    assert_eq!(result.actions.len(), 1);
    assert_eq!(result.actions[0].ticker, "7203");
    assert_eq!(result.actions[0].action, "buy_signal");
    assert!(result.actions[0].detail.contains("DRY RUN"));
    assert!(result.order_results.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_execute_dry_run_with_sell_signal() -> Result<()> {
    let db = setup_db().await?;
    let config = AppConfig::default();
    let spec = make_spec("name = \"Test\"\n");

    let stock_id = db.save_stock("7203", "Toyota", Some("Auto")).await?;
    db.watchlist_add("7203", Some("test")).await?;
    save_price(&db, stock_id, "2025-01-01", 2500.0).await?;
    db.portfolio_buy("7203", Decimal::from(100), Decimal::from(2400), None)
        .await?;
    db.save_evaluation(stock_id, "Sell", 30, "Time to sell", None, None, None)
        .await?;

    let result = execute::run(&db, &config, &spec, None, true).await?;

    assert_eq!(result.actions.len(), 1);
    assert_eq!(result.actions[0].action, "sell_signal");
    assert!(result.actions[0].detail.contains("DRY RUN"));

    Ok(())
}

#[tokio::test]
async fn test_execute_buy_score_too_low() -> Result<()> {
    let db = setup_db().await?;
    let config = AppConfig::default();
    let spec = make_spec("name = \"Test\"\n");

    let stock_id = db.save_stock("7203", "Toyota", Some("Auto")).await?;
    db.watchlist_add("7203", Some("test")).await?;
    save_price(&db, stock_id, "2025-01-01", 2500.0).await?;
    db.save_evaluation(stock_id, "Buy", 50, "Weak buy", None, None, None)
        .await?;

    let result = execute::run(&db, &config, &spec, None, true).await?;

    assert_eq!(result.actions.len(), 1);
    assert_eq!(result.actions[0].action, "hold");
    assert!(result.actions[0].detail.contains("score too low"));

    Ok(())
}

// ─── Hard stop-loss tests ────────────────────────────────────────────

#[tokio::test]
async fn test_hard_stop_loss_triggers_on_threshold() -> Result<()> {
    let db = setup_db().await?;
    let config = AppConfig::default();
    let spec = make_spec(
        r#"
name = "Test"
[execution]
stop_loss = -0.07
"#,
    );

    // Position bought at 2500, current price 2300 = -8% loss
    let stock_id = db.save_stock("7203", "Toyota", Some("Auto")).await?;
    db.watchlist_add("7203", Some("test")).await?;
    save_price(&db, stock_id, "2025-01-01", 2300.0).await?;
    db.portfolio_buy("7203", Decimal::from(100), Decimal::from(2500), None)
        .await?;

    let result = execute::run(&db, &config, &spec, None, true).await?;

    assert_eq!(result.hard_stop_loss_actions.len(), 1);
    assert_eq!(result.hard_stop_loss_actions[0].ticker, "7203");

    Ok(())
}

#[tokio::test]
async fn test_hard_stop_loss_does_not_trigger_within_threshold() -> Result<()> {
    let db = setup_db().await?;
    let config = AppConfig::default();
    let spec = make_spec(
        r#"
name = "Test"
[execution]
stop_loss = -0.07
"#,
    );

    // Position bought at 2500, current price 2450 = -2% loss (within threshold)
    let stock_id = db.save_stock("7203", "Toyota", Some("Auto")).await?;
    db.watchlist_add("7203", Some("test")).await?;
    save_price(&db, stock_id, "2025-01-01", 2450.0).await?;
    db.portfolio_buy("7203", Decimal::from(100), Decimal::from(2500), None)
        .await?;

    let result = execute::run(&db, &config, &spec, None, true).await?;

    assert!(result.hard_stop_loss_actions.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_hard_stop_loss_skipped_when_no_spec() -> Result<()> {
    let db = setup_db().await?;
    let config = AppConfig::default();
    let spec = make_spec("name = \"Test\"\n");

    let stock_id = db.save_stock("7203", "Toyota", Some("Auto")).await?;
    db.watchlist_add("7203", Some("test")).await?;
    save_price(&db, stock_id, "2025-01-01", 2000.0).await?;
    db.portfolio_buy("7203", Decimal::from(100), Decimal::from(2500), None)
        .await?;

    let result = execute::run(&db, &config, &spec, None, true).await?;

    // -20% loss but no stop_loss in spec, so no action
    assert!(result.hard_stop_loss_actions.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_hard_stop_loss_blocks_buy_signal() -> Result<()> {
    let db = setup_db().await?;
    let config = AppConfig::default();
    let spec = make_spec(
        r#"
name = "Test"
[execution]
stop_loss = -0.07
"#,
    );

    // Position with -10% loss + Buy evaluation
    let stock_id = db.save_stock("7203", "Toyota", Some("Auto")).await?;
    db.watchlist_add("7203", Some("test")).await?;
    save_price(&db, stock_id, "2025-01-01", 2250.0).await?;
    db.portfolio_buy("7203", Decimal::from(100), Decimal::from(2500), None)
        .await?;
    db.save_evaluation(stock_id, "Buy", 90, "Strong buy", None, None, None)
        .await?;

    let result = execute::run(&db, &config, &spec, None, true).await?;

    assert_eq!(result.hard_stop_loss_actions.len(), 1);
    let buy_action = result.actions.iter().find(|a| a.ticker == "7203").unwrap();
    assert_eq!(buy_action.action, "blocked_by_stop_loss");

    Ok(())
}

// ─── Sell + stop-loss conflict ────────────────────────────────────────

#[tokio::test]
async fn test_hard_stop_loss_defers_to_eval_sell() -> Result<()> {
    let db = setup_db().await?;
    let config = AppConfig::default();
    let spec = make_spec(
        r#"
name = "Test"
[execution]
stop_loss = -0.07
"#,
    );

    // Position with -10% loss + Sell evaluation
    let stock_id = db.save_stock("7203", "Toyota", Some("Auto")).await?;
    db.watchlist_add("7203", Some("test")).await?;
    save_price(&db, stock_id, "2025-01-01", 2250.0).await?;
    db.portfolio_buy("7203", Decimal::from(100), Decimal::from(2500), None)
        .await?;
    db.save_evaluation(stock_id, "Sell", 30, "Time to sell", None, None, None)
        .await?;

    let result = execute::run(&db, &config, &spec, None, true).await?;

    assert_eq!(result.hard_stop_loss_actions.len(), 1);
    let sell_action = result.actions.iter().find(|a| a.ticker == "7203").unwrap();
    assert_eq!(sell_action.action, "sell_signal");
    assert!(sell_action.detail.contains("DRY RUN"));

    Ok(())
}

// ─── Max position size tests (with mock broker) ──────────────────────

#[tokio::test]
async fn test_max_position_size_rejects_oversized_buy() -> Result<()> {
    let db = setup_db().await?;
    let config = AppConfig::default();
    let spec = make_spec(
        r#"
name = "Test"
[execution]
max_position_size = 0.05
[budget]
initial_cash = 300000
"#,
    );

    // Stock price 5000 × 100 shares = 500,000 > 300,000 × 0.05 = 15,000
    let stock_id = db.save_stock("7203", "Toyota", Some("Auto")).await?;
    db.watchlist_add("7203", Some("test")).await?;
    save_price(&db, stock_id, "2025-01-01", 5000.0).await?;
    db.save_evaluation(stock_id, "Buy", 90, "Strong buy", None, None, None)
        .await?;

    let mut broker = MockBrokerClient::new();
    let result = execute::run(&db, &config, &spec, Some(&mut broker), false).await?;

    // Order should be rejected due to max position size
    assert_eq!(result.order_results.len(), 1);
    assert!(
        result.order_results[0]
            .status
            .contains("exceeds max position size")
    );
    assert!(broker.placed_orders().is_empty()); // No order actually placed

    Ok(())
}

#[tokio::test]
async fn test_max_position_size_allows_small_buy() -> Result<()> {
    let db = setup_db().await?;
    let config = AppConfig::default();
    let spec = make_spec(
        r#"
name = "Test"
[execution]
max_position_size = 0.5
[budget]
initial_cash = 1000000
"#,
    );

    // Stock price 100 × 100 shares = 10,000 < 1,000,000 × 0.5 = 500,000
    let stock_id = db.save_stock("7203", "Toyota", Some("Auto")).await?;
    db.watchlist_add("7203", Some("test")).await?;
    save_price(&db, stock_id, "2025-01-01", 100.0).await?;
    db.save_evaluation(stock_id, "Buy", 90, "Strong buy", None, None, None)
        .await?;

    let mut broker = MockBrokerClient::new();
    let result = execute::run(&db, &config, &spec, Some(&mut broker), false).await?;

    // Order should go through
    assert_eq!(result.order_results.len(), 1);
    assert_eq!(result.order_results[0].status, "pending");
    assert_eq!(broker.placed_orders().len(), 1);

    Ok(())
}

#[tokio::test]
async fn test_max_position_size_skipped_when_not_configured() -> Result<()> {
    let db = setup_db().await?;
    let config = AppConfig::default();
    let spec = make_spec("name = \"Test\"\n");

    let stock_id = db.save_stock("7203", "Toyota", Some("Auto")).await?;
    db.watchlist_add("7203", Some("test")).await?;
    save_price(&db, stock_id, "2025-01-01", 5000.0).await?;
    db.save_evaluation(stock_id, "Buy", 90, "Strong buy", None, None, None)
        .await?;

    let mut broker = MockBrokerClient::new();
    let result = execute::run(&db, &config, &spec, Some(&mut broker), false).await?;

    // No max_position_size → order goes through
    assert_eq!(result.order_results.len(), 1);
    assert_eq!(result.order_results[0].status, "pending");
    assert_eq!(broker.placed_orders().len(), 1);

    Ok(())
}

// ─── Stop-loss with mock broker (non-dry-run) ────────────────────────

#[tokio::test]
async fn test_hard_stop_loss_places_market_order() -> Result<()> {
    let db = setup_db().await?;
    let config = AppConfig::default();
    let spec = make_spec(
        r#"
name = "Test"
[execution]
stop_loss = -0.07
"#,
    );

    // Position bought at 2500, current price 2300 = -8% loss
    let stock_id = db.save_stock("7203", "Toyota", Some("Auto")).await?;
    db.watchlist_add("7203", Some("test")).await?;
    save_price(&db, stock_id, "2025-01-01", 2300.0).await?;
    db.portfolio_buy("7203", Decimal::from(100), Decimal::from(2500), None)
        .await?;

    let mut broker = MockBrokerClient::new();
    let result = execute::run(&db, &config, &spec, Some(&mut broker), false).await?;

    assert_eq!(result.hard_stop_loss_actions.len(), 1);
    // Market order placed (price "0")
    let orders = broker.placed_orders();
    assert_eq!(orders.len(), 1);
    assert!(orders[0].contains("sell"));
    assert!(orders[0].contains("@0")); // market order

    Ok(())
}

// ─── Helpers ─────────────────────────────────────────────────────────

async fn save_price(db: &SqliteClient, stock_id: i64, date: &str, close: f64) -> Result<()> {
    let quotes = vec![kekekabu::jquants::DailyQuote {
        code: String::new(),
        date: date.to_string(),
        open: Some(close),
        high: Some(close),
        low: Some(close),
        close: Some(close),
        volume: Some(1000.0),
        adjustment_close: Some(close),
    }];
    db.save_prices(stock_id, &quotes).await
}
