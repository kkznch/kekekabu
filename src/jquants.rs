use anyhow::{Context, Result, bail};
use serde::Deserialize;

const BASE_URL: &str = "https://api.jquants.com/v2";

#[derive(Debug, Deserialize)]
struct ListedInfoResponse {
    pub info: Vec<ListedInfo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListedInfo {
    #[serde(rename = "Code")]
    pub code: String,
    #[serde(rename = "CompanyName")]
    pub company_name: String,
    #[serde(rename = "Sector33CodeName")]
    pub sector: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DailyQuotesResponse {
    pub daily_quotes: Vec<DailyQuote>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DailyQuote {
    #[serde(rename = "Code")]
    pub code: String,
    #[serde(rename = "Date")]
    pub date: String,
    #[serde(rename = "Open")]
    pub open: Option<f64>,
    #[serde(rename = "High")]
    pub high: Option<f64>,
    #[serde(rename = "Low")]
    pub low: Option<f64>,
    #[serde(rename = "Close")]
    pub close: Option<f64>,
    #[serde(rename = "Volume")]
    pub volume: Option<f64>,
    #[serde(rename = "AdjustmentClose")]
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

    pub async fn get_listed_info(&self) -> Result<Vec<ListedInfo>> {
        let resp = self
            .http
            .get(format!("{BASE_URL}/listed/info"))
            .header("Authorization", format!("Bearer {}", &self.api_key))
            .send()
            .await
            .context("Failed to request J-Quants listed/info")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            bail!("J-Quants listed/info failed ({}): {}", status, text);
        }

        let data: ListedInfoResponse = resp
            .json()
            .await
            .context("Failed to parse listed info response")?;

        Ok(data.info)
    }

    pub async fn get_stock_info(&self, code: &str) -> Result<Option<ListedInfo>> {
        let resp = self
            .http
            .get(format!("{BASE_URL}/listed/info?code={}", code))
            .header("Authorization", format!("Bearer {}", &self.api_key))
            .send()
            .await
            .context("Failed to request J-Quants listed/info")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            bail!("J-Quants listed/info failed ({}): {}", status, text);
        }

        let data: ListedInfoResponse = resp
            .json()
            .await
            .context("Failed to parse listed info response")?;

        Ok(data.info.into_iter().next())
    }

    pub async fn get_daily_quotes(
        &self,
        code: &str,
        date_from: &str,
        date_to: &str,
    ) -> Result<Vec<DailyQuote>> {
        let url = format!(
            "{BASE_URL}/prices/daily_quotes?code={}&from={}&to={}",
            code, date_from, date_to
        );

        let resp = self
            .http
            .get(&url)
            .header("Authorization", format!("Bearer {}", &self.api_key))
            .send()
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

        let data: DailyQuotesResponse = resp
            .json()
            .await
            .context("Failed to parse daily quotes response")?;

        Ok(data.daily_quotes)
    }
}
