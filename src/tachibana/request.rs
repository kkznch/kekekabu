use anyhow::{Context, Result};
use std::sync::atomic::{AtomicU64, Ordering};

/// Global monotonically-increasing request counter (p_no).
static P_NO: AtomicU64 = AtomicU64::new(1);

/// Get next p_no value and increment.
pub fn next_p_no() -> String {
    P_NO.fetch_add(1, Ordering::SeqCst).to_string()
}

/// Reset p_no (for testing).
#[cfg(test)]
pub fn reset_p_no() {
    P_NO.store(1, Ordering::SeqCst);
}

/// Generate p_sd_date in Tachibana format: `YYYY.MM.DD-HH:MM:SS.mmm`
pub fn p_sd_date() -> String {
    chrono::Local::now()
        .format("%Y.%m.%d-%H:%M:%S%.3f")
        .to_string()
}

/// Build a REQUEST I/F URL with JSON query parameter.
/// Tachibana API expects: `{base_url}?{url_encoded_json}`
pub fn build_request_url(base_url: &str, json_value: &serde_json::Value) -> Result<String> {
    let json_str = serde_json::to_string(json_value).context("Failed to serialize JSON")?;
    let encoded_json = urlencoding_json(&json_str);
    Ok(format!("{}?{}", base_url, encoded_json))
}

fn urlencoding_json(json: &str) -> String {
    url::form_urlencoded::byte_serialize(json.as_bytes()).collect()
}

/// Decode Shift-JIS response bytes to UTF-8 string.
pub fn decode_shift_jis(bytes: &[u8]) -> String {
    let (cow, _, _) = encoding_rs::SHIFT_JIS.decode(bytes);
    cow.into_owned()
}

/// Parse a Tachibana JSON response, checking for errors.
/// Returns the parsed JSON value or an error with the API error message.
pub fn parse_response(body: &str) -> Result<serde_json::Value> {
    let value: serde_json::Value =
        serde_json::from_str(body).context("Failed to parse Tachibana API response")?;

    // Check p_errno for request-level errors
    if let Some(errno) = value.get("p_errno").and_then(|v| v.as_str())
        && errno != "0"
    {
        let err_text = value
            .get("p_err")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown error");
        anyhow::bail!("Tachibana API error (p_errno={}): {}", errno, err_text);
    }

    // Check sResultCode for business-level errors
    if let Some(result_code) = value.get("sResultCode").and_then(|v| v.as_str())
        && result_code != "0"
    {
        let result_text = value
            .get("sResultText")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown error");
        anyhow::bail!(
            "Tachibana API result error (sResultCode={}): {}",
            result_code,
            result_text
        );
    }

    Ok(value)
}

/// Extract a string field from a JSON value, returning None if missing or null.
pub fn json_str(value: &serde_json::Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_p_no_increments() {
        reset_p_no();
        assert_eq!(next_p_no(), "1");
        assert_eq!(next_p_no(), "2");
        assert_eq!(next_p_no(), "3");
    }

    #[test]
    fn test_p_sd_date_format() {
        let date = p_sd_date();
        // Format: YYYY.MM.DD-HH:MM:SS.mmm
        assert!(date.len() >= 23, "Expected >= 23 chars, got: {}", date);
        assert!(date.contains('.'));
        assert!(date.contains('-'));
        assert!(date.contains(':'));
    }

    #[test]
    fn test_decode_shift_jis_ascii() {
        let input = b"Hello";
        assert_eq!(decode_shift_jis(input), "Hello");
    }

    #[test]
    fn test_decode_shift_jis_japanese() {
        // "テスト" in Shift-JIS
        let bytes: Vec<u8> = vec![0x83, 0x65, 0x83, 0x58, 0x83, 0x67];
        assert_eq!(decode_shift_jis(&bytes), "テスト");
    }

    #[test]
    fn test_build_request_url() {
        let json = serde_json::json!({"sCLMID": "CLMKabuNewOrder", "p_no": "1"});
        let url = build_request_url("https://example.com/api", &json).unwrap();
        assert!(url.starts_with("https://example.com/api?"));
        // Should contain URL-encoded JSON
        assert!(url.contains("sCLMID"));
    }

    #[test]
    fn test_parse_response_success() {
        let body = r#"{"p_errno": "0", "sResultCode": "0", "data": "ok"}"#;
        let result = parse_response(body);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_response_p_errno_error() {
        let body = r#"{"p_errno": "-2", "p_err": "パラメータエラー"}"#;
        let result = parse_response(body);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("p_errno=-2"));
    }

    #[test]
    fn test_parse_response_result_code_error() {
        let body = r#"{"p_errno": "0", "sResultCode": "1", "sResultText": "注文エラー"}"#;
        let result = parse_response(body);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("sResultCode=1"));
    }

    #[test]
    fn test_json_str() {
        let v = serde_json::json!({"sOrderNumber": "12345", "empty": null});
        assert_eq!(json_str(&v, "sOrderNumber"), Some("12345".to_string()));
        assert_eq!(json_str(&v, "empty"), None);
        assert_eq!(json_str(&v, "missing"), None);
    }
}
