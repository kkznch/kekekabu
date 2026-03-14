use anyhow::Result;
use async_trait::async_trait;
use kekekabu::config::AppConfig;
use kekekabu::db::{DbClient, SqliteClient};
use kekekabu::jquants::{DailyQuote, ListedInfo, StockApi};

struct MockStockApi {
    stocks: Vec<ListedInfo>,
    quotes: Vec<DailyQuote>,
}

#[async_trait]
impl StockApi for MockStockApi {
    async fn get_all_stock_info(&self) -> Result<Vec<ListedInfo>> {
        Ok(self.stocks.clone())
    }

    async fn get_daily_quotes(
        &self,
        _code: &str,
        _date_from: &str,
        _date_to: &str,
    ) -> Result<Vec<DailyQuote>> {
        Ok(self.quotes.clone())
    }
}

async fn setup_db() -> Result<SqliteClient> {
    SqliteClient::open_in_memory().await
}

fn make_quotes(code: &str, count: usize) -> Vec<DailyQuote> {
    (0..count)
        .map(|i| DailyQuote {
            code: code.to_string(),
            date: format!("2025-01-{:02}", i + 1),
            open: Some(100.0 + i as f64),
            high: Some(105.0 + i as f64),
            low: Some(95.0 + i as f64),
            close: Some(102.0 + i as f64),
            volume: Some(1000.0),
            adjustment_close: Some(102.0 + i as f64),
        })
        .collect()
}

#[tokio::test]
async fn test_scan_with_mock_api() -> Result<()> {
    let db = setup_db().await?;

    // Pre-populate stock master and watchlist
    db.save_stock("7203", "Toyota", Some("Automobile")).await?;
    db.watchlist_add("7203", Some("test")).await?;

    let api = MockStockApi {
        stocks: vec![],
        quotes: make_quotes("7203", 30),
    };

    let config = AppConfig::default();
    let results = kekekabu::cmd::scan::run(&db, &config, &api, 60, false).await?;

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].ticker, "7203");
    assert_eq!(results[0].name, "Toyota");
    assert_eq!(results[0].data_points, 30);
    assert!(results[0].indicators.is_some());

    Ok(())
}

#[tokio::test]
async fn test_scan_with_refresh_master() -> Result<()> {
    let db = setup_db().await?;

    // Add to watchlist but no stock master — refresh_master will populate it
    db.watchlist_add("6758", Some("test")).await?;

    let api = MockStockApi {
        stocks: vec![ListedInfo {
            code: "6758".to_string(),
            company_name: "Sony".to_string(),
            sector: Some("Electric".to_string()),
        }],
        quotes: make_quotes("6758", 10),
    };

    let config = AppConfig::default();
    let results = kekekabu::cmd::scan::run(&db, &config, &api, 60, true).await?;

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].ticker, "6758");
    assert_eq!(results[0].name, "Sony");
    assert_eq!(results[0].data_points, 10);

    Ok(())
}

#[tokio::test]
async fn test_scan_empty_watchlist_errors() -> Result<()> {
    let db = setup_db().await?;

    // Stock master exists but watchlist is empty
    db.save_stock("7203", "Toyota", None).await?;

    let api = MockStockApi {
        stocks: vec![],
        quotes: vec![],
    };

    let config = AppConfig::default();
    let result = kekekabu::cmd::scan::run(&db, &config, &api, 60, false).await;

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Watchlist is empty")
    );

    Ok(())
}
