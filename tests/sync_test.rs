use anyhow::Result;
use async_trait::async_trait;
use kekekabu::cmd::sync::{self, MismatchKind};
use kekekabu::db::{DbClient, SqliteClient};
use kekekabu::tachibana::{BrokerClient, Side, order};
use rust_decimal::Decimal;
use std::sync::Mutex;

async fn setup_db() -> Result<SqliteClient> {
    SqliteClient::open_in_memory().await
}

// ─── Mock broker for sync ────────────────────────────────────────────

struct MockSyncBroker {
    cash_available: String,
    positions: Mutex<Vec<order::BrokerPosition>>,
}

impl MockSyncBroker {
    fn new(cash: &str, positions: Vec<order::BrokerPosition>) -> Self {
        Self {
            cash_available: cash.to_string(),
            positions: Mutex::new(positions),
        }
    }
}

#[async_trait]
impl BrokerClient for MockSyncBroker {
    async fn ensure_logged_in(&mut self) -> Result<()> {
        Ok(())
    }

    async fn place_order(
        &self,
        _side: Side,
        _ticker: &str,
        _price: &str,
        _quantity: &str,
        _second_password: &str,
    ) -> Result<order::NewOrderResult> {
        anyhow::bail!("mock sync broker does not support orders")
    }

    async fn query_order(&self, _order_number: &str) -> Result<order::OrderDetail> {
        anyhow::bail!("mock sync broker does not support order detail")
    }

    async fn query_balance(&self) -> Result<order::BrokerBalance> {
        Ok(order::BrokerBalance {
            cash_available: self.cash_available.clone(),
        })
    }

    async fn query_positions(&self) -> Result<Vec<order::BrokerPosition>> {
        Ok(self.positions.lock().unwrap().clone())
    }

    async fn logout(&mut self) -> Result<()> {
        Ok(())
    }
}

// ─── Tests ───────────────────────────────────────────────────────────

#[tokio::test]
async fn test_sync_no_mismatches() -> Result<()> {
    let db = setup_db().await?;

    // Set up DB position
    db.portfolio_buy(
        "7203",
        Decimal::from(100),
        Decimal::from(2500),
        Some("test"),
    )
    .await?;

    // Broker reports same position
    let mut broker = MockSyncBroker::new(
        "500000",
        vec![order::BrokerPosition {
            ticker: "7203".to_string(),
            quantity: 100,
            avg_cost: "2500".to_string(),
        }],
    );

    let result = sync::run(&db, &mut broker, false).await?;

    assert_eq!(result.cash_available, "500000");
    assert_eq!(result.broker_position_count, 1);
    assert_eq!(result.db_position_count, 1);
    assert!(result.mismatches.is_empty());
    assert!(!result.fixed);

    // Balance was saved
    let saved = db.get_latest_balance().await?.unwrap();
    assert_eq!(saved.cash_available, "500000");

    Ok(())
}

#[tokio::test]
async fn test_sync_db_only_no_fix() -> Result<()> {
    let db = setup_db().await?;
    db.portfolio_buy(
        "7203",
        Decimal::from(100),
        Decimal::from(2500),
        Some("test"),
    )
    .await?;

    // Broker has no positions
    let mut broker = MockSyncBroker::new("500000", vec![]);

    let result = sync::run(&db, &mut broker, false).await?;

    assert_eq!(result.mismatches.len(), 1);
    assert_eq!(result.mismatches[0].ticker, "7203");
    assert_eq!(result.mismatches[0].kind, MismatchKind::DbOnly);
    assert!(!result.fixed);

    // DB position should still exist (no --fix)
    let positions = db.list_positions().await?;
    assert_eq!(positions.len(), 1);

    Ok(())
}

#[tokio::test]
async fn test_sync_db_only_with_fix() -> Result<()> {
    let db = setup_db().await?;
    db.portfolio_buy(
        "7203",
        Decimal::from(100),
        Decimal::from(2500),
        Some("test"),
    )
    .await?;

    let mut broker = MockSyncBroker::new("500000", vec![]);

    let result = sync::run(&db, &mut broker, true).await?;

    assert!(result.fixed);

    // DB position should be deleted
    let positions = db.list_positions().await?;
    assert!(positions.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_sync_broker_only_with_fix() -> Result<()> {
    let db = setup_db().await?;
    // No DB positions

    // Broker reports an unknown position
    let mut broker = MockSyncBroker::new(
        "500000",
        vec![order::BrokerPosition {
            ticker: "6184".to_string(),
            quantity: 200,
            avg_cost: "466".to_string(),
        }],
    );

    let result = sync::run(&db, &mut broker, true).await?;

    assert_eq!(result.mismatches.len(), 1);
    assert_eq!(result.mismatches[0].kind, MismatchKind::BrokerOnly);
    assert!(result.fixed);

    let positions = db.list_positions().await?;
    assert_eq!(positions.len(), 1);
    assert_eq!(positions[0].ticker, "6184");
    assert_eq!(positions[0].quantity, Decimal::from(200));

    Ok(())
}

#[tokio::test]
async fn test_sync_quantity_diff_with_fix() -> Result<()> {
    let db = setup_db().await?;
    db.portfolio_buy(
        "7203",
        Decimal::from(100),
        Decimal::from(2500),
        Some("test"),
    )
    .await?;

    // Broker reports different quantity
    let mut broker = MockSyncBroker::new(
        "500000",
        vec![order::BrokerPosition {
            ticker: "7203".to_string(),
            quantity: 300,
            avg_cost: "2500".to_string(),
        }],
    );

    let result = sync::run(&db, &mut broker, true).await?;

    assert_eq!(result.mismatches.len(), 1);
    assert_eq!(result.mismatches[0].kind, MismatchKind::QuantityDiff);
    assert!(result.fixed);

    let positions = db.list_positions().await?;
    assert_eq!(positions.len(), 1);
    assert_eq!(positions[0].quantity, Decimal::from(300));

    Ok(())
}

#[tokio::test]
async fn test_sync_no_fix_does_not_modify_db() -> Result<()> {
    let db = setup_db().await?;
    db.portfolio_buy(
        "7203",
        Decimal::from(100),
        Decimal::from(2500),
        Some("test"),
    )
    .await?;

    let mut broker = MockSyncBroker::new(
        "500000",
        vec![order::BrokerPosition {
            ticker: "6184".to_string(),
            quantity: 200,
            avg_cost: "466".to_string(),
        }],
    );

    let result = sync::run(&db, &mut broker, false).await?;

    // Both DbOnly and BrokerOnly mismatches
    assert_eq!(result.mismatches.len(), 2);
    assert!(!result.fixed);

    // DB unchanged
    let positions = db.list_positions().await?;
    assert_eq!(positions.len(), 1);
    assert_eq!(positions[0].ticker, "7203");

    Ok(())
}

#[tokio::test]
async fn test_sync_balance_history() -> Result<()> {
    let db = setup_db().await?;
    let mut broker = MockSyncBroker::new("100000", vec![]);

    // First sync
    sync::run(&db, &mut broker, false).await?;

    // Change balance and sync again
    let mut broker2 = MockSyncBroker::new("200000", vec![]);
    sync::run(&db, &mut broker2, false).await?;

    // Latest balance should be the most recent
    let latest = db.get_latest_balance().await?.unwrap();
    assert_eq!(latest.cash_available, "200000");

    Ok(())
}
