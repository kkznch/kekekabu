pub mod event;
pub mod order;
pub mod request;

use anyhow::{Context, Result};

use crate::config::TachibanaConfig;
use request::{decode_shift_jis, json_str};

const AUTH_URL: &str = "https://kabuka.e-shiten.jp/e_api_v4r8/auth/";

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
    session: Option<SessionUrls>,
}

impl TachibanaClient {
    /// Create a new client from config. Does not log in automatically.
    pub fn new(config: &TachibanaConfig) -> Self {
        Self {
            http: reqwest::Client::new(),
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
            "p_no": request::next_p_no(),
            "p_sd_date": request::p_sd_date(),
            "sUserId": user_id,
            "sPassword": password,
            "sSecondPassword": second_password,
        });

        let url = request::build_request_url(AUTH_URL, &auth_json)?;

        tracing::info!("Logging in to Tachibana API");

        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .context("Failed to send auth request")?;

        let bytes = resp.bytes().await.context("Failed to read auth response")?;
        let body = decode_shift_jis(&bytes);
        let value = request::parse_response(&body)?;

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

    /// Send a REQUEST I/F command and return the parsed response.
    #[allow(dead_code)]
    pub async fn send_request(&self, json_value: &serde_json::Value) -> Result<serde_json::Value> {
        let session = self
            .session
            .as_ref()
            .context("Not logged in — call login() first")?;

        let url = request::build_request_url(&session.request_url, json_value)?;
        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .context("Failed to send request")?;
        let bytes = resp.bytes().await.context("Failed to read response")?;
        let body = decode_shift_jis(&bytes);

        request::parse_response(&body)
    }

    /// Place a new limit order.
    pub async fn place_order(
        &self,
        side: &str,
        ticker: &str,
        price: &str,
        quantity: &str,
    ) -> Result<order::NewOrderResult> {
        let json = order::build_new_order_json(side, ticker, price, quantity);
        let session = self
            .session
            .as_ref()
            .context("Not logged in — call login() first")?;

        let url = request::build_request_url(&session.request_url, &json)?;
        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .context("Failed to send order")?;
        let bytes = resp
            .bytes()
            .await
            .context("Failed to read order response")?;
        let body = decode_shift_jis(&bytes);

        order::parse_new_order_response(&body)
    }

    /// Query order detail by order number.
    pub async fn query_order(&self, order_number: &str) -> Result<order::OrderDetail> {
        let eigyou_day = ""; // empty = today
        let json = order::build_order_detail_json(order_number, eigyou_day);
        let session = self
            .session
            .as_ref()
            .context("Not logged in — call login() first")?;

        let url = request::build_request_url(&session.request_url, &json)?;
        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .context("Failed to send order query")?;
        let bytes = resp
            .bytes()
            .await
            .context("Failed to read order query response")?;
        let body = decode_shift_jis(&bytes);

        order::parse_order_detail_response(&body)
    }

    /// Wait for fill notifications via WebSocket.
    pub async fn wait_for_fills(
        &self,
        timeout_secs: u64,
        pending_order_numbers: &[String],
    ) -> Result<Vec<event::FillNotification>> {
        let session = self
            .session
            .as_ref()
            .context("Not logged in — call login() first")?;

        event::wait_for_fills(&session.event_ws_url, timeout_secs, pending_order_numbers).await
    }

    /// Log out (invalidate virtual URLs).
    pub async fn logout(&mut self) -> Result<()> {
        if let Some(session) = &self.session {
            let json = serde_json::json!({
                "sCLMID": "CLMLogout",
                "p_no": request::next_p_no(),
                "p_sd_date": request::p_sd_date(),
            });

            let url = request::build_request_url(&session.request_url, &json)?;
            // Best-effort logout — don't fail if it errors
            match self.http.get(&url).send().await {
                Ok(_) => tracing::info!("Logged out from Tachibana API"),
                Err(e) => tracing::warn!(error = %e, "Failed to logout from Tachibana API"),
            }
        }
        self.session = None;
        Ok(())
    }

    /// Get the event timeout from config.
    pub fn event_timeout_secs(&self) -> u64 {
        self.config.event_timeout_secs
    }
}
