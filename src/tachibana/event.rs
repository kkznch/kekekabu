use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use std::time::Duration;
use tokio_tungstenite::tungstenite::Message;

use super::request::json_str;

/// A fill notification received from EVENT I/F WebSocket.
#[derive(Debug, Clone)]
pub struct FillNotification {
    pub order_number: String,
    pub issue_code: String,
    pub filled_price: String,
    pub filled_quantity: String,
}

/// Connect to the EVENT I/F WebSocket and wait for fill notifications.
///
/// Returns a list of fill notifications received before the timeout.
/// On connection failure, returns Ok(empty vec) with a warning log.
pub async fn wait_for_fills(
    ws_url: &str,
    timeout_secs: u64,
    pending_order_numbers: &[String],
) -> Result<Vec<FillNotification>> {
    let mut fills = Vec::new();

    if pending_order_numbers.is_empty() {
        return Ok(fills);
    }

    tracing::info!(
        url = ws_url,
        timeout_secs,
        pending_count = pending_order_numbers.len(),
        "Connecting to Tachibana EVENT I/F WebSocket"
    );

    let ws_stream = match tokio_tungstenite::connect_async(ws_url).await {
        Ok((stream, _)) => stream,
        Err(e) => {
            tracing::warn!(
                error = %e,
                "Failed to connect to Tachibana WebSocket, skipping fill wait"
            );
            return Ok(fills);
        }
    };

    let (mut write, mut read) = ws_stream.split();

    // Send subscription message to register for execution notifications
    let subscribe_msg = build_event_subscribe_json().to_string();
    if let Err(e) = write.send(Message::Text(subscribe_msg.into())).await {
        tracing::warn!(error = %e, "Failed to send EVENT subscribe message");
        return Ok(fills);
    }
    tracing::info!("Sent EVENT I/F subscription for execution notifications");
    let timeout = tokio::time::sleep(Duration::from_secs(timeout_secs));
    tokio::pin!(timeout);

    let mut all_filled = false;

    loop {
        tokio::select! {
            _ = &mut timeout => {
                tracing::info!("WebSocket fill wait timed out after {}s", timeout_secs);
                break;
            }
            msg = read.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Some(fill) = parse_fill_notification(&text)
                            && pending_order_numbers.contains(&fill.order_number) {
                                tracing::info!(
                                    order = %fill.order_number,
                                    ticker = %fill.issue_code,
                                    price = %fill.filled_price,
                                    qty = %fill.filled_quantity,
                                    "Fill notification received"
                                );
                                fills.push(fill);

                                // If all pending orders are filled, stop early
                                if fills.len() >= pending_order_numbers.len() {
                                    all_filled = true;
                                    break;
                                }
                            }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        tracing::info!("WebSocket connection closed");
                        break;
                    }
                    Some(Err(e)) => {
                        tracing::warn!(error = %e, "WebSocket read error");
                        break;
                    }
                    _ => {} // Ping/Pong/Binary — ignore
                }
            }
        }
    }

    if all_filled {
        tracing::info!("All pending orders filled");
    }

    Ok(fills)
}

/// Try to parse a WebSocket message as a fill notification (EC event).
fn parse_fill_notification(text: &str) -> Option<FillNotification> {
    let value: serde_json::Value = serde_json::from_str(text).ok()?;

    // Check if this is an execution notification (EC event)
    let evt_cmd = json_str(&value, "p_evt_cmd")?;
    if evt_cmd != "EC" {
        return None;
    }

    // Check if the order is filled or partially filled
    let status_code = json_str(&value, "sOrderStatusCode").unwrap_or_default();
    if status_code != "10" && status_code != "9" {
        // Not a fill event — skip other status changes
        return None;
    }

    let order_number = json_str(&value, "sOrderNumber")?;
    let issue_code = json_str(&value, "sIssueCode").unwrap_or_default();
    let filled_price = json_str(&value, "sYakuzyouPrice").unwrap_or_default();
    let filled_quantity = json_str(&value, "sYakuzyouSuryou").unwrap_or_default();

    Some(FillNotification {
        order_number,
        issue_code,
        filled_price,
        filled_quantity,
    })
}

/// Build the EVENT I/F registration request to subscribe to execution notifications.
fn build_event_subscribe_json() -> serde_json::Value {
    serde_json::json!({
        "p_evt_cmd": "EC",
        "p_no": super::request::next_p_no(),
        "p_sd_date": super::request::p_sd_date(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_fill_notification_ec_filled() {
        let msg = r#"{
            "p_evt_cmd": "EC",
            "sOrderNumber": "ORD001",
            "sIssueCode": "7203",
            "sOrderStatusCode": "10",
            "sYakuzyouPrice": "2500",
            "sYakuzyouSuryou": "100"
        }"#;
        let fill = parse_fill_notification(msg).unwrap();
        assert_eq!(fill.order_number, "ORD001");
        assert_eq!(fill.issue_code, "7203");
        assert_eq!(fill.filled_price, "2500");
        assert_eq!(fill.filled_quantity, "100");
    }

    #[test]
    fn test_parse_fill_notification_not_ec() {
        let msg = r#"{"p_evt_cmd": "KP", "sIssueCode": "7203"}"#;
        assert!(parse_fill_notification(msg).is_none());
    }

    #[test]
    fn test_parse_fill_notification_partial() {
        let msg = r#"{
            "p_evt_cmd": "EC",
            "sOrderNumber": "ORD002",
            "sIssueCode": "6758",
            "sOrderStatusCode": "9",
            "sYakuzyouPrice": "15000",
            "sYakuzyouSuryou": "50"
        }"#;
        let fill = parse_fill_notification(msg).unwrap();
        assert_eq!(fill.order_number, "ORD002");
        assert_eq!(fill.filled_quantity, "50");
    }

    #[test]
    fn test_parse_fill_notification_not_filled() {
        let msg = r#"{
            "p_evt_cmd": "EC",
            "sOrderNumber": "ORD001",
            "sOrderStatusCode": "1"
        }"#;
        assert!(parse_fill_notification(msg).is_none());
    }

    #[test]
    fn test_parse_fill_notification_invalid_json() {
        assert!(parse_fill_notification("not json").is_none());
    }
}
