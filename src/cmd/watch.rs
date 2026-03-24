use anyhow::{Context, Result};
use futures_util::StreamExt;
use std::time::Duration;
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, info, trace, warn};

use crate::config::AppConfig;
use crate::db::{DbClient, FillParams};
use crate::tachibana::TachibanaClient;
use crate::tachibana::event::{EventMessage, build_event_ws_url, parse_event_message};

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

        // Build WebSocket URL with EC subscription query params
        let ws_url = build_event_ws_url(&session.event_ws_url);
        info!(ws_url = %ws_url, "Connecting to EVENT I/F WebSocket");

        let ws_stream = match tokio_tungstenite::connect_async(&ws_url).await {
            Ok((stream, _)) => stream,
            Err(e) => {
                warn!(error = %e, backoff_secs, "Failed to connect WebSocket, retrying after backoff");
                let _ = client.logout().await;
                tokio::time::sleep(Duration::from_secs(backoff_secs)).await;
                backoff_secs = (backoff_secs * 2).min(MAX_BACKOFF_SECS);
                continue;
            }
        };

        // No subscription message needed — EVENT I/F uses URL query params
        let (_write, mut read) = ws_stream.split();

        info!("WebSocket connected, listening for fill notifications (Ctrl-C to stop)");
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
                            debug!(raw_len = text.len(), "Received EVENT message");
                            let event = parse_event_message(&text);
                            match event {
                                EventMessage::EC(ec) => {
                                    info!(
                                        order_number = %ec.order_number,
                                        notification_type = %ec.notification_type,
                                        execution_status = %ec.execution_status,
                                        issue_code = %ec.issue_code,
                                        "EC event received"
                                    );
                                    // p_NT=12 means execution filled
                                    if ec.notification_type == "12" {
                                        let is_partial = ec.execution_status == "1";
                                        if let Err(e) = process_fill(conn, &ec, is_partial).await {
                                            warn!(
                                                order_number = %ec.order_number,
                                                error = %e,
                                                "Failed to process fill notification"
                                            );
                                        }
                                    }
                                }
                                EventMessage::KP => {
                                    trace!("Keep-alive received");
                                }
                                EventMessage::ST(st) => {
                                    warn!(
                                        errno = %st.errno,
                                        err = %st.err,
                                        "EVENT I/F error status received (server will disconnect)"
                                    );
                                }
                                EventMessage::Other(cmd) => {
                                    debug!(cmd = %cmd, "Unhandled EVENT type");
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

/// Process a single fill notification (p_NT=12): look up order in DB and update.
async fn process_fill(
    conn: &dyn DbClient,
    ec: &crate::tachibana::event::ExecutionEvent,
    is_partial: bool,
) -> Result<()> {
    let pending_orders = conn.list_pending_orders().await?;

    let order = pending_orders
        .iter()
        .find(|o| o.tachibana_order_id.as_deref() == Some(&ec.order_number));

    let Some(order) = order else {
        info!(
            order_number = %ec.order_number,
            "No matching pending order found in DB, skipping"
        );
        return Ok(());
    };

    let filled_at = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    conn.update_order_and_record_fill(FillParams {
        order_id: order.id,
        status: if is_partial { "partial" } else { "filled" }.to_string(),
        tachibana_order_id: None,
        filled_price: Some(ec.execution_price.clone()),
        filled_quantity: Some(ec.execution_quantity.clone()),
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
        price = %ec.execution_price,
        quantity = %ec.execution_quantity,
        partial = is_partial,
        "Fill recorded in DB"
    );

    Ok(())
}
