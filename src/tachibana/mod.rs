pub mod compress;
pub mod event;
pub mod order;
pub mod request;

use anyhow::{Context, Result};
use async_trait::async_trait;

use crate::config::TachibanaConfig;
use request::{decode_shift_jis, json_str};

/// Order side (buy or sell).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Buy,
    Sell,
}

impl Side {
    pub fn as_str(&self) -> &'static str {
        match self {
            Side::Buy => "buy",
            Side::Sell => "sell",
        }
    }
}

impl std::fmt::Display for Side {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Broker API trait for dependency injection.
#[async_trait]
pub trait BrokerClient: Send + Sync {
    async fn ensure_logged_in(&mut self) -> Result<()>;
    async fn place_order(
        &self,
        side: Side,
        ticker: &str,
        price: &str,
        quantity: &str,
        second_password: &str,
    ) -> Result<order::NewOrderResult>;
    async fn query_order(&self, order_number: &str) -> Result<order::OrderDetail>;
    async fn query_balance(&self) -> Result<order::BrokerBalance>;
    async fn query_positions(&self) -> Result<Vec<order::BrokerPosition>>;
    async fn logout(&mut self) -> Result<()>;
}

/// Session URLs obtained after successful authentication.
#[derive(Debug, Clone)]
pub struct SessionUrls {
    pub request_url: String,
    pub event_ws_url: String,
}

/// Tachibana Securities e-Shiten API client.
pub struct TachibanaClient {
    http: reqwest::Client,
    config: TachibanaConfig,
    auth_url: &'static str,
    session: Option<SessionUrls>,
}

impl TachibanaClient {
    /// Create a new client from config. Does not log in automatically.
    pub fn new(config: &TachibanaConfig) -> Self {
        Self {
            http: reqwest::Client::new(),
            auth_url: config.auth_url(),
            config: config.clone(),
            session: None,
        }
    }

    /// Log in to the Tachibana API and obtain session URLs.
    pub async fn login(&mut self) -> Result<&SessionUrls> {
        let user_id = self
            .config
            .user_id
            .as_ref()
            .context("tachibana.user_id is not configured")?;
        let password = self
            .config
            .password
            .as_ref()
            .context("tachibana.password is not configured")?;
        let second_password = self
            .config
            .second_password
            .as_ref()
            .context("tachibana.second_password is not configured")?;

        let auth_json = serde_json::json!({
            "sCLMID": "CLMAuthLoginRequest",
            "p_no": request::next_p_no(),
            "p_sd_date": request::p_sd_date(),
            "sUserId": user_id,
            "sPassword": password,
            "sSecondPassword": second_password,
        });

        let compressed_json = compress::compress(&auth_json);
        let body = request::build_request_body(&compressed_json)?;

        tracing::info!(auth_url = %self.auth_url, "Logging in to Tachibana API");

        let resp = self
            .http
            .post(self.auth_url)
            .header("Content-Type", "application/json; charset=Shift_JIS")
            .body(body)
            .send()
            .await
            .context("Failed to send auth request")?;

        let status = resp.status();
        let bytes = resp.bytes().await.context("Failed to read auth response")?;
        let raw_body = decode_shift_jis(&bytes);
        let raw_value: serde_json::Value =
            serde_json::from_str(&raw_body).context("Failed to parse auth response JSON")?;
        let value = compress::uncompress(&raw_value);
        tracing::debug!(http_status = %status, body_len = bytes.len(), "Auth response: {}", &value);
        request::check_response_errors(&value)?;

        // Check for 金商法交付書面未読
        if let Some(flag) = json_str(&value, "sKinsyouhouMidokuFlg")
            && flag == "1"
        {
            anyhow::bail!(
                "金商法交付書面が未読です。Webブラウザで立花証券にログインして書面を確認してください。"
            );
        }

        let request_url =
            json_str(&value, "sUrlRequest").context("Missing sUrlRequest in auth response")?;
        let event_ws_url = json_str(&value, "sUrlEventWebSocket")
            .context("Missing sUrlEventWebSocket in auth response")?;

        let urls = SessionUrls {
            request_url,
            event_ws_url,
        };

        tracing::info!("Tachibana API login successful");
        self.session = Some(urls);
        Ok(self.session.as_ref().unwrap())
    }

    /// Get current session URLs, or None if not logged in.
    #[allow(dead_code)]
    pub fn session(&self) -> Option<&SessionUrls> {
        self.session.as_ref()
    }

    /// Ensure we are logged in, returning session URLs.
    pub async fn ensure_logged_in(&mut self) -> Result<&SessionUrls> {
        if self.session.is_some() {
            return Ok(self.session.as_ref().unwrap());
        }
        self.login().await
    }

    /// Send a REQUEST I/F command via POST, compress outgoing keys, uncompress response keys.
    async fn send_request_raw(&self, json_value: &serde_json::Value) -> Result<serde_json::Value> {
        let session = self
            .session
            .as_ref()
            .context("Not logged in — call login() first")?;

        let compressed = compress::compress(json_value);
        let body = request::build_request_body(&compressed)?;
        let resp = self
            .http
            .post(&session.request_url)
            .header("Content-Type", "application/json; charset=Shift_JIS")
            .body(body)
            .send()
            .await
            .context("Failed to send request")?;
        let bytes = resp.bytes().await.context("Failed to read response")?;
        let raw_body = decode_shift_jis(&bytes);
        let raw_value: serde_json::Value =
            serde_json::from_str(&raw_body).context("Failed to parse response JSON")?;
        Ok(compress::uncompress(&raw_value))
    }

    /// Place a new order.
    pub async fn place_order(
        &self,
        side: Side,
        ticker: &str,
        price: &str,
        quantity: &str,
        second_password: &str,
    ) -> Result<order::NewOrderResult> {
        let json = order::build_new_order_json(side, ticker, price, quantity, second_password);
        let value = self.send_request_raw(&json).await?;
        order::parse_new_order_value(&value)
    }

    /// Query order detail by order number.
    pub async fn query_order(&self, order_number: &str) -> Result<order::OrderDetail> {
        let json = order::build_order_detail_json(order_number, "");
        let value = self.send_request_raw(&json).await?;
        order::parse_order_detail_value(&value)
    }

    /// Query buying power (cash available for new orders).
    pub async fn query_balance(&self) -> Result<order::BrokerBalance> {
        let json = order::build_balance_query_json();
        let value = self.send_request_raw(&json).await?;
        order::parse_balance_value(&value)
    }

    /// Query current spot stock holdings.
    pub async fn query_positions(&self) -> Result<Vec<order::BrokerPosition>> {
        let json = order::build_positions_query_json();
        let value = self.send_request_raw(&json).await?;
        order::parse_positions_value(&value)
    }

    /// Log out (invalidate virtual URLs).
    pub async fn logout(&mut self) -> Result<()> {
        if let Some(session) = &self.session {
            let json = serde_json::json!({
                "sCLMID": "CLMAuthLogoutRequest",
                "p_no": request::next_p_no(),
                "p_sd_date": request::p_sd_date(),
            });

            let compressed = compress::compress(&json);
            let body = request::build_request_body(&compressed)?;
            // Best-effort logout — don't fail if it errors, response not uncompressed intentionally
            match self
                .http
                .post(&session.request_url)
                .header("Content-Type", "application/json; charset=Shift_JIS")
                .body(body)
                .send()
                .await
            {
                Ok(_) => tracing::info!("Logged out from Tachibana API"),
                Err(e) => tracing::warn!(error = %e, "Failed to logout from Tachibana API"),
            }
        }
        self.session = None;
        Ok(())
    }
}

#[async_trait]
impl BrokerClient for TachibanaClient {
    async fn ensure_logged_in(&mut self) -> Result<()> {
        TachibanaClient::ensure_logged_in(self).await?;
        Ok(())
    }

    async fn place_order(
        &self,
        side: Side,
        ticker: &str,
        price: &str,
        quantity: &str,
        second_password: &str,
    ) -> Result<order::NewOrderResult> {
        TachibanaClient::place_order(self, side, ticker, price, quantity, second_password).await
    }

    async fn query_order(&self, order_number: &str) -> Result<order::OrderDetail> {
        TachibanaClient::query_order(self, order_number).await
    }

    async fn query_balance(&self) -> Result<order::BrokerBalance> {
        TachibanaClient::query_balance(self).await
    }

    async fn query_positions(&self) -> Result<Vec<order::BrokerPosition>> {
        TachibanaClient::query_positions(self).await
    }

    async fn logout(&mut self) -> Result<()> {
        TachibanaClient::logout(self).await
    }
}
