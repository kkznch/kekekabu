use anyhow::Result;
use serde::Serialize;
use tracing::info;

use crate::config::AppConfig;
use crate::db::DbClient;
use crate::indicators::{self, TechnicalIndicators};
use crate::jquants::StockApi;

#[derive(Debug, Serialize)]
pub struct ScanResult {
    pub ticker: String,
    pub name: String,
    pub sector: Option<String>,
    pub latest_close: Option<f64>,
    pub data_points: usize,
    pub indicators: Option<TechnicalIndicators>,
}

pub async fn run(
    conn: &dyn DbClient,
    _config: &AppConfig,
    api: &dyn StockApi,
    days: u32,
    refresh_master: bool,
) -> Result<Vec<ScanResult>> {
    // Refresh master data if requested
    if refresh_master {
        info!("Refreshing stock master data from J-Quants API");
        let all_stocks = api.get_all_stock_info().await?;
        let count = conn.save_stocks_bulk(&all_stocks).await?;
        info!(count, "Stock master data refreshed");
    } else if !conn.has_any_stocks().await? {
        anyhow::bail!(
            "stocks テーブルが空です。先に kabu scan --refresh-master を実行してください"
        );
    }

    let watchlist = conn.watchlist_list().await?;
    if watchlist.is_empty() {
        anyhow::bail!("Watchlist is empty. Run kabu discover first.");
    }

    let to_date = chrono::Local::now().format("%Y-%m-%d").to_string();
    let from_date = (chrono::Local::now() - chrono::Duration::days(days as i64))
        .format("%Y-%m-%d")
        .to_string();

    info!(count = watchlist.len(), "Fetching prices for watchlist");

    let mut results = Vec::new();

    for (i, item) in watchlist.iter().enumerate() {
        // Look up stock from DB (populated by --refresh-master)
        let stock_id = match conn.get_stock_id(&item.ticker).await? {
            Some(id) => id,
            None => {
                tracing::warn!(
                    ticker = %item.ticker,
                    "Stock not found in master data, skipping. Run kabu scan --refresh-master to update."
                );
                continue;
            }
        };

        if i > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        }

        info!(ticker = %item.ticker, "Fetching daily quotes");

        match api
            .get_daily_quotes(&item.ticker, &from_date, &to_date)
            .await
        {
            Ok(quotes) => {
                info!(ticker = %item.ticker, count = quotes.len(), "Saved quotes");
                conn.save_prices(stock_id, &quotes).await?;
            }
            Err(e) => {
                tracing::warn!(ticker = %item.ticker, error = %e, "Failed to fetch quotes");
                continue;
            }
        }

        let price_data = conn.fetch_price_data(stock_id).await?;
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

        // Get stock info from DB
        let stock_info = conn.get_stock_info(stock_id).await?;

        results.push(ScanResult {
            ticker: item.ticker.clone(),
            name: stock_info
                .as_ref()
                .map(|s| s.name.clone())
                .unwrap_or_else(|| item.ticker.clone()),
            sector: stock_info.and_then(|s| s.sector),
            latest_close,
            data_points,
            indicators: ta,
        });
    }

    Ok(results)
}
