use anyhow::{Context, Result, bail};
use reqwest::StatusCode;
use serde::Deserialize;
use tracing::warn;

const BASE_URL: &str = "https://api.jquants.com/v2";
const MAX_RETRIES: u32 = 3;

#[derive(Debug, Deserialize)]
struct EquitiesMasterResponse {
    data: Vec<ListedInfo>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct ListedInfo {
    #[serde(rename = "Code")]
    pub code: String,
    #[serde(rename = "CoName")]
    pub company_name: String,
    #[serde(rename = "S33Nm")]
    pub sector: Option<String>,
}

#[derive(Debug, Deserialize)]
struct EquitiesBarsResponse {
    data: Vec<DailyQuote>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct DailyQuote {
    #[serde(rename = "Code")]
    pub code: String,
    #[serde(rename = "Date")]
    pub date: String,
    #[serde(rename = "O")]
    pub open: Option<f64>,
    #[serde(rename = "H")]
    pub high: Option<f64>,
    #[serde(rename = "L")]
    pub low: Option<f64>,
    #[serde(rename = "C")]
    pub close: Option<f64>,
    #[serde(rename = "Vo")]
    pub volume: Option<f64>,
    #[serde(rename = "AdjC")]
    pub adjustment_close: Option<f64>,
}

pub struct JQuantsClient {
    http: reqwest::Client,
    api_key: String,
}

impl JQuantsClient {
    pub fn new(api_key: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            api_key,
        }
    }

    async fn get_with_retry(&self, url: &str) -> Result<reqwest::Response> {
        for attempt in 0..MAX_RETRIES {
            let resp = self
                .http
                .get(url)
                .header("x-api-key", &self.api_key)
                .send()
                .await?;

            if resp.status() == StatusCode::TOO_MANY_REQUESTS {
                let wait = 2u64.pow(attempt + 1);
                warn!(attempt = attempt + 1, wait_secs = wait, "Rate limited, retrying");
                tokio::time::sleep(std::time::Duration::from_secs(wait)).await;
                continue;
            }

            return Ok(resp);
        }

        bail!("Rate limited after {} retries: {}", MAX_RETRIES, url)
    }

    pub async fn get_stock_info(&self, code: &str) -> Result<Option<ListedInfo>> {
        let url = format!("{BASE_URL}/equities/master?code={}", code);
        let resp = self
            .get_with_retry(&url)
            .await
            .context("Failed to request J-Quants equities/master")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            bail!("J-Quants equities/master failed ({}): {}", status, text);
        }

        let data: EquitiesMasterResponse = resp
            .json()
            .await
            .context("Failed to parse equities master response")?;

        Ok(data.data.into_iter().next())
    }

    pub async fn get_daily_quotes(
        &self,
        code: &str,
        date_from: &str,
        date_to: &str,
    ) -> Result<Vec<DailyQuote>> {
        let url = format!(
            "{BASE_URL}/equities/bars/daily?code={}&from={}&to={}",
            code, date_from, date_to
        );

        let resp = self
            .get_with_retry(&url)
            .await
            .with_context(|| format!("Failed to request daily quotes for {}", code))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            bail!(
                "J-Quants daily quotes failed for {} ({}): {}",
                code,
                status,
                text
            );
        }

        let data: EquitiesBarsResponse = resp
            .json()
            .await
            .context("Failed to parse daily quotes response")?;

        Ok(data.data)
    }
}
