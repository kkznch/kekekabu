use super::request::json_str;

/// A fill notification received from EVENT I/F WebSocket.
#[derive(Debug, Clone)]
pub struct FillNotification {
    pub order_number: String,
    pub issue_code: String,
    pub filled_price: String,
    pub filled_quantity: String,
    /// true if partial fill (sOrderStatusCode="9"), false if full fill ("10")
    pub is_partial: bool,
}

/// Try to parse a WebSocket message as a fill notification (EC event).
/// Expects the message to be already uncompressed (caller is responsible for compress::uncompress).
pub fn parse_fill_notification(text: &str) -> Option<FillNotification> {
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
        is_partial: status_code == "9",
    })
}

/// Build the EVENT I/F registration request to subscribe to execution notifications.
/// NOTE: Caller is responsible for applying compress::compress() before sending.
pub fn build_event_subscribe_json() -> serde_json::Value {
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
        assert!(!fill.is_partial);
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
        assert!(fill.is_partial);
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
