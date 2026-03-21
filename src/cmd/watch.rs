use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use std::time::Duration;
use tokio_tungstenite::tungstenite::Message;
use tracing::{info, warn};

use crate::config::AppConfig;
use crate::db::{DbClient, FillParams};
use crate::tachibana::TachibanaClient;
use crate::tachibana::event::{build_event_subscribe_json, parse_fill_notification};

/// Run the watch command: connect to Tachibana EVENT I/F WebSocket and
/// listen for fill notifications, updating the DB in real time.
pub async fn run(conn: &dyn DbClient, config: &AppConfig) -> Result<()> {
    let tc_config = config.tachibana.as_ref().ok_or_else(|| {
        anyhow::anyhow!(
            "[tachibana] config is required for watch command. \
             Set it in ~/.config/kabu/config.toml or use TACHIBANA_* env vars."
        )
    })?;

    let mut backoff_secs: u64 = 1;
    const MAX_BACKOFF_SECS: u64 = 60;

    loop {
        // Login to Tachibana API
        let mut client = TachibanaClient::new(tc_config);
        let session = match client.login().await {
            Ok(s) => s.clone(),
            Err(e) => {
                warn!(error = %e, backoff_secs, "Failed to login, retrying after backoff");
                tokio::time::sleep(Duration::from_secs(backoff_secs)).await;
                backoff_secs = (backoff_secs * 2).min(MAX_BACKOFF_SECS);
                continue;
            }
        };

        info!(ws_url = %session.event_ws_url, "Connecting to EVENT I/F WebSocket");

        let ws_stream = match tokio_tungstenite::connect_async(&session.event_ws_url).await {
            Ok((stream, _)) => stream,
            Err(e) => {
                warn!(error = %e, backoff_secs, "Failed to connect WebSocket, retrying after backoff");
                let _ = client.logout().await;
                tokio::time::sleep(Duration::from_secs(backoff_secs)).await;
                backoff_secs = (backoff_secs * 2).min(MAX_BACKOFF_SECS);
                continue;
            }
        };

        let (mut write, mut read) = ws_stream.split();

        // Send EC subscription
        let subscribe_msg = build_event_subscribe_json().to_string();
        if let Err(e) = write.send(Message::Text(subscribe_msg.into())).await {
            warn!(error = %e, "Failed to send EVENT subscribe message");
            let _ = client.logout().await;
            tokio::time::sleep(Duration::from_secs(backoff_secs)).await;
            backoff_secs = (backoff_secs * 2).min(MAX_BACKOFF_SECS);
            continue;
        }

        info!("WebSocket connected, listening for fill notifications (Ctrl-C to stop)");
        // Reset backoff on successful connection
        backoff_secs = 1;

        let mut shutdown = false;

        loop {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {
                    info!("Received SIGINT, shutting down gracefully");
                    shutdown = true;
                    break;
                }
                msg = read.next() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            if let Some(fill) = parse_fill_notification(&text) {
                                info!(
                                    order_number = %fill.order_number,
                                    ticker = %fill.issue_code,
                                    price = %fill.filled_price,
                                    qty = %fill.filled_quantity,
                                    "Fill notification received"
                                );
                                if let Err(e) = process_fill(conn, &fill).await {
                                    warn!(
                                        order_number = %fill.order_number,
                                        error = %e,
                                        "Failed to process fill notification"
                                    );
                                }
                            }
                        }
                        Some(Ok(Message::Close(_))) | None => {
                            warn!("WebSocket connection closed by server");
                            break;
                        }
                        Some(Err(e)) => {
                            warn!(error = %e, "WebSocket read error");
                            break;
                        }
                        _ => {} // Ping/Pong/Binary — ignore
                    }
                }
            }
        }

        // Logout before reconnect or exit
        info!("Logging out from Tachibana API");
        let _ = client.logout().await;

        if shutdown {
            info!("Watch command stopped");
            return Ok(());
        }

        // Reconnect with backoff
        warn!(
            backoff_secs,
            "WebSocket disconnected, reconnecting after backoff"
        );
        tokio::time::sleep(Duration::from_secs(backoff_secs)).await;
        backoff_secs = (backoff_secs * 2).min(MAX_BACKOFF_SECS);
    }
}

/// Process a single fill notification: look up order in DB and update accordingly.
async fn process_fill(
    conn: &dyn DbClient,
    fill: &crate::tachibana::event::FillNotification,
) -> Result<()> {
    let pending_orders = conn.list_pending_orders().await?;

    let order = pending_orders
        .iter()
        .find(|o| o.tachibana_order_id.as_deref() == Some(&fill.order_number));

    let Some(order) = order else {
        info!(
            order_number = %fill.order_number,
            "No matching pending order found in DB, skipping"
        );
        return Ok(());
    };

    let filled_at = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    conn.update_order_and_record_fill(FillParams {
        order_id: order.id,
        status: "filled".to_string(),
        tachibana_order_id: None,
        filled_price: Some(fill.filled_price.clone()),
        filled_quantity: Some(fill.filled_quantity.clone()),
        filled_at: Some(filled_at),
        ticker: order.ticker.clone(),
        side: order.side.clone(),
    })
    .await
    .context("Failed to update order and record fill")?;

    info!(
        ticker = %order.ticker,
        order_id = order.id,
        side = %order.side,
        price = %fill.filled_price,
        quantity = %fill.filled_quantity,
        "Fill recorded in DB"
    );

    Ok(())
}
