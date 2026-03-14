use anyhow::Result;
use async_trait::async_trait;

/// Notification backend trait. Implement this for each platform (LINE, Slack, ntfy, etc.).
/// TODO: Add concrete backend implementation — https://github.com/kkznch/kekekabu/issues/30
#[async_trait]
pub trait Notifier: Send + Sync {
    async fn send(&self, message: &str) -> Result<()>;
}

/// No-op notifier — silently discards all messages.
pub struct NullNotifier;

#[async_trait]
impl Notifier for NullNotifier {
    async fn send(&self, _message: &str) -> Result<()> {
        Ok(())
    }
}

pub fn format_execute_summary(result: &crate::cmd::execute::ExecuteResult) -> Option<String> {
    let mut lines = Vec::new();

    // Circuit breaker
    if result.circuit_breaker_triggered {
        lines.push("[Circuit Breaker Triggered]".to_string());
        for reason in &result.circuit_breaker_reasons {
            lines.push(format!("  - {}", reason));
        }
    }

    // Hard stop-loss actions
    for sl in &result.hard_stop_loss_actions {
        lines.push(format!(
            "[STOP-LOSS] {} ({}) loss: {}% threshold: {}%",
            sl.ticker, sl.name, sl.loss_pct, sl.threshold
        ));
    }

    // Orders placed
    for order in &result.order_results {
        lines.push(format!(
            "[{}] {} {} x{} @{} [{}]",
            order.side.to_uppercase(),
            order.ticker,
            order.tachibana_order_id.as_deref().unwrap_or("no-id"),
            order.quantity,
            order.price,
            order.status,
        ));
    }

    // Settle results
    for settle in &result.settle_results {
        lines.push(format!(
            "[SETTLED] {} {} -> {}",
            settle.ticker, settle.old_status, settle.new_status
        ));
    }

    if lines.is_empty() {
        None
    } else {
        lines.insert(0, "[kabu execute]".to_string());
        Some(lines.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::execute::{ExecuteResult, HardStopLossAction, OrderResult};

    #[test]
    fn test_format_execute_summary_empty() {
        let result = ExecuteResult {
            actions: vec![],
            circuit_breaker_triggered: false,
            circuit_breaker_reasons: vec![],
            hard_stop_loss_actions: vec![],
            settle_results: vec![],
            order_results: vec![],
        };
        assert!(format_execute_summary(&result).is_none());
    }

    #[test]
    fn test_format_execute_summary_with_order() {
        let result = ExecuteResult {
            actions: vec![],
            circuit_breaker_triggered: false,
            circuit_breaker_reasons: vec![],
            hard_stop_loss_actions: vec![],
            settle_results: vec![],
            order_results: vec![OrderResult {
                ticker: "7203".to_string(),
                side: "buy".to_string(),
                price: "2500".to_string(),
                quantity: "100".to_string(),
                tachibana_order_id: Some("ORD001".to_string()),
                status: "pending".to_string(),
            }],
        };
        let msg = format_execute_summary(&result).unwrap();
        assert!(msg.contains("[kabu execute]"));
        assert!(msg.contains("[BUY]"));
        assert!(msg.contains("7203"));
    }

    #[test]
    fn test_format_execute_summary_with_stop_loss() {
        let result = ExecuteResult {
            actions: vec![],
            circuit_breaker_triggered: false,
            circuit_breaker_reasons: vec![],
            hard_stop_loss_actions: vec![HardStopLossAction {
                ticker: "9984".to_string(),
                name: "SoftBank".to_string(),
                avg_cost: "5000".to_string(),
                current_price: "4500".to_string(),
                loss_pct: "-10.0".to_string(),
                threshold: "-7.0".to_string(),
                action: "Force-sell".to_string(),
            }],
            settle_results: vec![],
            order_results: vec![],
        };
        let msg = format_execute_summary(&result).unwrap();
        assert!(msg.contains("STOP-LOSS"));
        assert!(msg.contains("9984"));
    }

    #[test]
    fn test_format_execute_summary_circuit_breaker() {
        let result = ExecuteResult {
            actions: vec![],
            circuit_breaker_triggered: true,
            circuit_breaker_reasons: vec!["Market crash detected".to_string()],
            hard_stop_loss_actions: vec![],
            settle_results: vec![],
            order_results: vec![],
        };
        let msg = format_execute_summary(&result).unwrap();
        assert!(msg.contains("Circuit Breaker"));
        assert!(msg.contains("Market crash detected"));
    }

    #[test]
    fn test_null_notifier() {
        let notifier = NullNotifier;
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            assert!(notifier.send("test").await.is_ok());
        });
    }
}
