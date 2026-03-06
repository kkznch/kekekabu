use anyhow::Result;
use serde::Serialize;
use tokio_rusqlite::Connection;
use tracing::info;

use crate::config::AppConfig;
use crate::db;
use crate::indicators::{self, TechnicalIndicators};
use crate::jquants::JQuantsClient;

#[derive(Debug, Serialize)]
pub struct ScanResult {
    pub ticker: String,
    pub name: String,
    pub sector: Option<String>,
    pub latest_close: Option<f64>,
    pub data_points: usize,
    pub indicators: Option<TechnicalIndicators>,
}

pub async fn run(conn: &Connection, config: &AppConfig, days: u32) -> Result<Vec<ScanResult>> {
    let api_key = AppConfig::require_key(&config.api.jquants_api_key, "JQUANTS_API_KEY")?;
    let client = JQuantsClient::new(api_key);

    let watchlist = db::watchlist_list(conn).await?;
    if watchlist.is_empty() {
        anyhow::bail!("Watchlist is empty. Add stocks with: kktd watchlist add <ticker>");
    }

    let to_date = chrono::Local::now().format("%Y-%m-%d").to_string();
    let from_date = (chrono::Local::now() - chrono::Duration::days(days as i64))
        .format("%Y-%m-%d")
        .to_string();

    info!(count = watchlist.len(), "Fetching prices for watchlist");

    let mut results = Vec::new();

    for (i, item) in watchlist.iter().enumerate() {
        if i > 0 {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }

        info!(ticker = %item.ticker, "Fetching data");

        let stock_info = client.get_stock_info(&item.ticker).await?;
        let name = stock_info
            .as_ref()
            .map(|i| i.company_name.as_str())
            .unwrap_or(&item.ticker);
        let sector = stock_info.as_ref().and_then(|i| i.sector.as_deref());

        let stock_id = db::save_stock(conn, &item.ticker, name, sector).await?;

        match client
            .get_daily_quotes(&item.ticker, &from_date, &to_date)
            .await
        {
            Ok(quotes) => {
                info!(ticker = %item.ticker, count = quotes.len(), "Saved quotes");
                db::save_prices(conn, stock_id, &quotes).await?;
            }
            Err(e) => {
                tracing::warn!(ticker = %item.ticker, error = %e, "Failed to fetch quotes");
                continue;
            }
        }

        let price_data = db::fetch_price_data(conn, stock_id).await?;
        let data_points = price_data.closes.len();
        let latest_close = price_data.closes.last().copied();

        let ta = if data_points >= 5 {
            Some(indicators::calculate_indicators(
                &price_data.closes,
                &price_data.highs,
                &price_data.lows,
                &price_data.volumes,
            )?)
        } else {
            None
        };

        results.push(ScanResult {
            ticker: item.ticker.clone(),
            name: name.to_string(),
            sector: sector.map(|s| s.to_string()),
            latest_close,
            data_points,
            indicators: ta,
        });
    }

    Ok(results)
}
