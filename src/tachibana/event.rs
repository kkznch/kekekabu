use std::collections::HashMap;

/// EVENT I/F field separator (^A = 0x01)
const FIELD_SEP: char = '\x01';
/// EVENT I/F key/value separator (^B = 0x02)
const KV_SEP: char = '\x02';

/// Parse EVENT I/F proprietary format into key-value pairs.
/// Format: `key1^Bvalue1^Akey2^Bvalue2^A...`
pub fn parse_event_fields(text: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for field in text.split(FIELD_SEP) {
        if field.is_empty() {
            continue;
        }
        if let Some((key, value)) = field.split_once(KV_SEP) {
            map.insert(key.to_string(), value.to_string());
        }
    }
    map
}

/// Parsed EVENT I/F message.
#[derive(Debug, Clone)]
pub enum EventMessage {
    /// EC: Order/execution notification
    EC(ExecutionEvent),
    /// KP: Keep-alive (sent every 5s if no other notifications)
    KP,
    /// ST: Error status (server will disconnect after this)
    ST(ErrorStatus),
    /// Other event types (FD, NS, SS, US, etc.)
    Other(String),
}

/// EC (execution) event fields.
#[derive(Debug, Clone)]
pub struct ExecutionEvent {
    /// p_ON: Order number
    pub order_number: String,
    /// p_NT: Notification type (1=accepted, 12=filled, 13=expired, etc.)
    pub notification_type: String,
    /// p_EXST: Execution status (0=unfilled, 1=partial, 2=filled, 3=filling)
    pub execution_status: String,
    /// p_EXPR: Execution price (from exchange)
    pub execution_price: String,
    /// p_EXSR: Execution quantity (from exchange)
    pub execution_quantity: String,
    /// p_IC: Issue code
    pub issue_code: String,
    /// p_BBKB: Buy/sell (1=sell, 3=buy)
    pub buy_sell: String,
    /// p_ENO: Event number (unique per business day)
    pub event_number: String,
}

/// ST (error status) event fields.
#[derive(Debug, Clone)]
pub struct ErrorStatus {
    pub errno: String,
    pub err: String,
}

/// Parse a raw EVENT I/F text message into an EventMessage.
pub fn parse_event_message(text: &str) -> EventMessage {
    let fields = parse_event_fields(text);
    let cmd = fields.get("p_cmd").map(|s| s.as_str()).unwrap_or("");

    match cmd {
        "EC" => EventMessage::EC(ExecutionEvent {
            order_number: fields.get("p_ON").cloned().unwrap_or_default(),
            notification_type: fields.get("p_NT").cloned().unwrap_or_default(),
            execution_status: fields.get("p_EXST").cloned().unwrap_or_default(),
            execution_price: fields.get("p_EXPR").cloned().unwrap_or_default(),
            execution_quantity: fields.get("p_EXSR").cloned().unwrap_or_default(),
            issue_code: fields.get("p_IC").cloned().unwrap_or_default(),
            buy_sell: fields.get("p_BBKB").cloned().unwrap_or_default(),
            event_number: fields.get("p_ENO").cloned().unwrap_or_default(),
        }),
        "KP" => EventMessage::KP,
        "ST" => EventMessage::ST(ErrorStatus {
            errno: fields.get("p_errno").cloned().unwrap_or_default(),
            err: fields.get("p_err").cloned().unwrap_or_default(),
        }),
        other => EventMessage::Other(other.to_string()),
    }
}

/// Build the EVENT I/F WebSocket URL with subscription query parameters.
/// Subscribes to EC (execution) events only.
pub fn build_event_ws_url(base_url: &str) -> String {
    format!("{base_url}?p_rid=0&p_board_no=1000&p_eno=0&p_evt_cmd=EC")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_event_fields_normal() {
        let msg = "p_no\x021\x01p_date\x022026.03.24\x01p_cmd\x02EC\x01";
        let fields = parse_event_fields(msg);
        assert_eq!(fields.get("p_no").unwrap(), "1");
        assert_eq!(fields.get("p_date").unwrap(), "2026.03.24");
        assert_eq!(fields.get("p_cmd").unwrap(), "EC");
    }

    #[test]
    fn test_parse_event_fields_empty() {
        let fields = parse_event_fields("");
        assert!(fields.is_empty());
    }

    #[test]
    fn test_parse_event_fields_no_separator() {
        let fields = parse_event_fields("no_separator_here");
        assert!(fields.is_empty());
    }

    #[test]
    fn test_parse_event_message_ec() {
        let msg = "p_no\x021\x01p_cmd\x02EC\x01p_ON\x023000945\x01p_NT\x0212\x01p_EXST\x022\x01p_EXPR\x02850.0\x01p_EXSR\x02100\x01p_IC\x022468\x01p_BBKB\x023\x01p_ENO\x0210507\x01";
        match parse_event_message(msg) {
            EventMessage::EC(ec) => {
                assert_eq!(ec.order_number, "3000945");
                assert_eq!(ec.notification_type, "12");
                assert_eq!(ec.execution_status, "2");
                assert_eq!(ec.execution_price, "850.0");
                assert_eq!(ec.execution_quantity, "100");
                assert_eq!(ec.issue_code, "2468");
                assert_eq!(ec.buy_sell, "3");
                assert_eq!(ec.event_number, "10507");
            }
            other => panic!("Expected EC, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_event_message_ec_partial() {
        let msg = "p_cmd\x02EC\x01p_ON\x02123\x01p_NT\x0212\x01p_EXST\x021\x01p_EXPR\x02500.0\x01p_EXSR\x0250\x01";
        match parse_event_message(msg) {
            EventMessage::EC(ec) => {
                assert_eq!(ec.execution_status, "1"); // partial
                assert_eq!(ec.execution_quantity, "50");
            }
            other => panic!("Expected EC, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_event_message_kp() {
        let msg = "p_no\x0220\x01p_date\x022018.12.03-11:34:59.138\x01p_cmd\x02KP\x01";
        assert!(matches!(parse_event_message(msg), EventMessage::KP));
    }

    #[test]
    fn test_parse_event_message_st() {
        let msg = "p_no\x021\x01p_date\x022026.03.24\x01p_errno\x02-1\x01p_err\x02parameter error.\x01p_cmd\x02ST\x01";
        match parse_event_message(msg) {
            EventMessage::ST(st) => {
                assert_eq!(st.errno, "-1");
                assert_eq!(st.err, "parameter error.");
            }
            other => panic!("Expected ST, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_event_message_other() {
        let msg = "p_cmd\x02FD\x01p_no\x021\x01";
        assert!(matches!(parse_event_message(msg), EventMessage::Other(s) if s == "FD"));
    }

    #[test]
    fn test_parse_event_message_no_cmd() {
        let msg = "p_no\x021\x01";
        assert!(matches!(parse_event_message(msg), EventMessage::Other(s) if s.is_empty()));
    }

    #[test]
    fn test_build_event_ws_url() {
        let url = build_event_ws_url("wss://demo-kabuka.e-shiten.jp/e_api_v4r8/event_ws/TOKEN123/");
        assert_eq!(
            url,
            "wss://demo-kabuka.e-shiten.jp/e_api_v4r8/event_ws/TOKEN123/?p_rid=0&p_board_no=1000&p_eno=0&p_evt_cmd=EC"
        );
    }
}
