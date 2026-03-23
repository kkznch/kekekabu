use anyhow::{Context, Result};

use super::Side;
use super::request::{self, json_str};

/// Build CLMKabuNewOrder request JSON.
pub fn build_new_order_json(
    side: Side,
    ticker: &str,
    price: &str,
    quantity: &str,
    second_password: &str,
) -> serde_json::Value {
    let baibai_kubun = match side {
        Side::Buy => "3",
        Side::Sell => "1",
    };

    serde_json::json!({
        "sCLMID": "CLMKabuNewOrder",
        "sZyoutoekiKazeiC": "1",                   // 特定口座
        "sIssueCode": ticker,
        "sSizyouC": "00",                           // 東証
        "sBaibaiKubun": baibai_kubun,
        "sCondition": "0",                          // 指定なし
        "sOrderPrice": price,
        "sOrderSuryou": quantity,
        "sGenkinShinyouKubun": "0",                 // 現物
        "sOrderExpireDay": "0",                     // 当日限り
        "sGyakusasiOrderType": "0",                 // 通常（逆指値なし）
        "sGyakusasiZyouken": "0",                   // 指定なし
        "sGyakusasiPrice": "*",                     // 指定なし
        "sTatebiType": "*",                         // 指定なし（現物）
        "sTategyokuZyoutoekiKazeiC": "*",           // 指定なし
        "sSecondPassword": second_password,
        "p_no": request::next_p_no(),
        "p_sd_date": request::p_sd_date(),
    })
}

/// Result of placing a new order.
#[allow(dead_code)]
pub struct NewOrderResult {
    pub order_number: String,
    pub result_text: String,
}

/// Parse CLMKabuNewOrder response from a raw body string.
pub fn parse_new_order_response(body: &str) -> Result<NewOrderResult> {
    let value = request::parse_response(body)?;
    parse_new_order_value(&value)
}

/// Parse CLMKabuNewOrder from an already-parsed (and uncompressed) JSON value.
pub fn parse_new_order_value(value: &serde_json::Value) -> Result<NewOrderResult> {
    request::check_response_errors(value)?;
    let order_number =
        json_str(value, "sOrderNumber").context("Missing sOrderNumber in new order response")?;
    let result_text = json_str(value, "sResultText").unwrap_or_default();

    Ok(NewOrderResult {
        order_number,
        result_text,
    })
}

/// Build CLMOrderListDetail request JSON to query order status.
///
/// - `order_number`: the tachibana order number
/// - `eigyou_day`: business date in YYYYMMDD format (empty string = today)
pub fn build_order_detail_json(order_number: &str, eigyou_day: &str) -> serde_json::Value {
    serde_json::json!({
        "sCLMID": "CLMOrderListDetail",
        "sOrderNumber": order_number,
        "sEigyouDay": eigyou_day,
        "p_no": request::next_p_no(),
        "p_sd_date": request::p_sd_date(),
    })
}

/// Parsed order detail from CLMOrderListDetail.
#[allow(dead_code)]
pub struct OrderDetail {
    pub order_number: String,
    pub issue_code: String,
    pub status_code: String,
    pub baibai_kubun: String,
    pub order_price: String,
    pub order_quantity: String,
    pub filled_price: Option<String>,
    pub filled_quantity: Option<String>,
}

/// Map sOrderStatusCode to our internal order status.
pub fn map_status_code(code: &str) -> &'static str {
    match code {
        "0" | "1" | "13" => "pending", // 受付未済, 未約定, 発注待ち
        "9" => "partial",              // 一部約定
        "10" => "filled",              // 全部約定
        "2" => "rejected",             // 受付エラー
        "7" => "cancelled",            // 取消完了
        "12" | "19" => "expired",      // 全部失効, 繰越失効
        _ => "pending",                // Unknown → keep as pending for re-check
    }
}

/// Parse CLMOrderListDetail response from a raw body string.
pub fn parse_order_detail_response(body: &str) -> Result<OrderDetail> {
    let value = request::parse_response(body)?;
    parse_order_detail_value(&value)
}

/// Parse CLMOrderListDetail from an already-parsed (and uncompressed) JSON value.
pub fn parse_order_detail_value(value: &serde_json::Value) -> Result<OrderDetail> {
    request::check_response_errors(value)?;

    let order_number =
        json_str(value, "sOrderNumber").context("Missing sOrderNumber in order detail")?;
    let issue_code = json_str(value, "sIssueCode").unwrap_or_default();
    let status_code = json_str(value, "sOrderStatusCode").unwrap_or_default();
    let baibai_kubun = json_str(value, "sBaibaiKubun").unwrap_or_default();
    let order_price = json_str(value, "sOrderPrice").unwrap_or_default();
    let order_quantity = json_str(value, "sOrderSuryou").unwrap_or_default();
    let filled_price = json_str(value, "sYakuzyouPrice");
    let filled_quantity = json_str(value, "sYakuzyouSuryou");

    Ok(OrderDetail {
        order_number,
        issue_code,
        status_code,
        baibai_kubun,
        order_price,
        order_quantity,
        filled_price,
        filled_quantity,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_new_order_buy() {
        let json = build_new_order_json(Side::Buy, "7203", "2500", "100", "pass");
        assert_eq!(json["sCLMID"], "CLMKabuNewOrder");
        assert_eq!(json["sIssueCode"], "7203");
        assert_eq!(json["sBaibaiKubun"], "3"); // buy
        assert_eq!(json["sOrderPrice"], "2500");
        assert_eq!(json["sOrderSuryou"], "100");
        assert_eq!(json["sZyoutoekiKazeiC"], "1"); // 特定口座
        assert_eq!(json["sSecondPassword"], "pass");
        assert_eq!(json["sGenkinShinyouKubun"], "0"); // 現物
        assert_eq!(json["sGyakusasiOrderType"], "0"); // 通常
        assert_eq!(json["sGyakusasiPrice"], "*"); // 指定なし
    }

    #[test]
    fn test_build_new_order_sell() {
        let json = build_new_order_json(Side::Sell, "6758", "15000", "200", "pass");
        assert_eq!(json["sBaibaiKubun"], "1"); // sell
    }

    #[test]
    fn test_map_status_code() {
        assert_eq!(map_status_code("0"), "pending");
        assert_eq!(map_status_code("1"), "pending");
        assert_eq!(map_status_code("9"), "partial");
        assert_eq!(map_status_code("10"), "filled");
        assert_eq!(map_status_code("2"), "rejected");
        assert_eq!(map_status_code("7"), "cancelled");
        assert_eq!(map_status_code("12"), "expired");
        assert_eq!(map_status_code("19"), "expired");
        assert_eq!(map_status_code("99"), "pending"); // unknown
    }

    #[test]
    fn test_parse_new_order_response_success() {
        let body =
            r#"{"p_errno":"0","sResultCode":"0","sOrderNumber":"ORD001","sResultText":"OK"}"#;
        let result = parse_new_order_response(body).unwrap();
        assert_eq!(result.order_number, "ORD001");
        assert_eq!(result.result_text, "OK");
    }

    #[test]
    fn test_parse_new_order_response_error() {
        let body = r#"{"p_errno":"0","sResultCode":"1","sResultText":"余力不足"}"#;
        let result = parse_new_order_response(body);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_order_detail_response() {
        let body = r#"{
            "p_errno":"0","sResultCode":"0",
            "sOrderNumber":"ORD001","sIssueCode":"7203",
            "sOrderStatusCode":"10","sBaibaiKubun":"3",
            "sOrderPrice":"2500","sOrderSuryou":"100",
            "sYakuzyouPrice":"2500","sYakuzyouSuryou":"100"
        }"#;
        let detail = parse_order_detail_response(body).unwrap();
        assert_eq!(detail.order_number, "ORD001");
        assert_eq!(detail.status_code, "10");
        assert_eq!(detail.filled_price.as_deref(), Some("2500"));
        assert_eq!(detail.filled_quantity.as_deref(), Some("100"));
    }

    #[test]
    fn test_build_order_detail_json() {
        let json = build_order_detail_json("ORD001", "20260313");
        assert_eq!(json["sCLMID"], "CLMOrderListDetail");
        assert_eq!(json["sOrderNumber"], "ORD001");
        assert_eq!(json["sEigyouDay"], "20260313");
    }
}
