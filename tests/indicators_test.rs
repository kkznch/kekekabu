use keketrade::indicators::*;

fn sample_closes(n: usize) -> Vec<f64> {
    (0..n).map(|i| 100.0 + (i as f64) * 0.5 + (i as f64 * 0.3).sin() * 5.0).collect()
}

#[test]
fn test_calculate_indicators_minimal() {
    let closes = vec![100.0, 101.0, 102.0];
    let highs = vec![101.0, 102.0, 103.0];
    let lows = vec![99.0, 100.0, 101.0];
    let volumes = vec![1000.0, 1100.0, 1200.0];

    let result = calculate_indicators(&closes, &highs, &lows, &volumes).unwrap();
    assert!(result.sma_5.is_empty());
    assert!(result.rsi_14.is_empty());
}

#[test]
fn test_calculate_indicators_full() {
    let n = 100;
    let closes = sample_closes(n);
    let highs: Vec<f64> = closes.iter().map(|c| c + 2.0).collect();
    let lows: Vec<f64> = closes.iter().map(|c| c - 2.0).collect();
    let volumes: Vec<f64> = (0..n).map(|_| 1000000.0).collect();

    let result = calculate_indicators(&closes, &highs, &lows, &volumes).unwrap();

    assert!(!result.sma_5.is_empty());
    assert!(!result.sma_25.is_empty());
    assert!(!result.sma_75.is_empty());
    assert!(!result.ema_12.is_empty());
    assert!(!result.ema_26.is_empty());
    assert!(!result.rsi_14.is_empty());
    assert!(!result.macd.is_empty());
    assert!(!result.bollinger.is_empty());
    assert!(!result.volume_ma_20.is_empty());
    assert!(!result.atr_14.is_empty());

    let latest = result.latest_values();
    assert!(latest.contains_key("RSI"));
    assert!(latest.contains_key("SMA5"));
    assert!(latest.contains_key("MACD"));
    assert!(latest.contains_key("BB_Upper"));
}

#[test]
fn test_golden_cross_detection() {
    let indicators = TechnicalIndicators {
        sma_5: vec![10.0, 12.0],
        sma_25: vec![11.0, 11.0],
        sma_75: vec![],
        ema_12: vec![],
        ema_26: vec![],
        rsi_14: vec![],
        macd: vec![],
        bollinger: vec![],
        volume_ma_20: vec![],
        atr_14: vec![],
        latest: std::collections::HashMap::new(),
        signals: vec![],
    };
    assert!(is_golden_cross(&indicators));
    assert!(!is_dead_cross(&indicators));
}

#[test]
fn test_dead_cross_detection() {
    let indicators = TechnicalIndicators {
        sma_5: vec![12.0, 10.0],
        sma_25: vec![11.0, 11.0],
        sma_75: vec![],
        ema_12: vec![],
        ema_26: vec![],
        rsi_14: vec![],
        macd: vec![],
        bollinger: vec![],
        volume_ma_20: vec![],
        atr_14: vec![],
        latest: std::collections::HashMap::new(),
        signals: vec![],
    };
    assert!(is_dead_cross(&indicators));
    assert!(!is_golden_cross(&indicators));
}

#[test]
fn test_bollinger_breakout() {
    let indicators = TechnicalIndicators {
        sma_5: vec![],
        sma_25: vec![],
        sma_75: vec![],
        ema_12: vec![],
        ema_26: vec![],
        rsi_14: vec![],
        macd: vec![],
        bollinger: vec![(95.0, 100.0, 105.0)],
        volume_ma_20: vec![],
        atr_14: vec![],
        latest: std::collections::HashMap::new(),
        signals: vec![],
    };

    assert_eq!(bollinger_breakout(&indicators, 110.0), Some("upper_break"));
    assert_eq!(bollinger_breakout(&indicators, 90.0), Some("lower_break"));
    assert_eq!(bollinger_breakout(&indicators, 100.0), None);
}

#[test]
fn test_volume_spike() {
    let volumes = vec![2500000.0];
    let indicators = TechnicalIndicators {
        sma_5: vec![],
        sma_25: vec![],
        sma_75: vec![],
        ema_12: vec![],
        ema_26: vec![],
        rsi_14: vec![],
        macd: vec![],
        bollinger: vec![],
        volume_ma_20: vec![1000000.0],
        atr_14: vec![],
        latest: std::collections::HashMap::new(),
        signals: vec![],
    };

    assert!(is_volume_spike(&volumes, &indicators, 2.0));
    assert!(!is_volume_spike(&volumes, &indicators, 3.0));
}
