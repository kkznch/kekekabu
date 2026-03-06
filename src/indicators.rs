use std::collections::HashMap;

use anyhow::Result;
use rust_ti::standard_indicators::bulk;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct TechnicalIndicators {
    #[serde(skip)]
    pub sma_5: Vec<f64>,
    #[serde(skip)]
    pub sma_25: Vec<f64>,
    #[serde(skip)]
    pub sma_75: Vec<f64>,
    #[serde(skip)]
    pub ema_12: Vec<f64>,
    #[serde(skip)]
    pub ema_26: Vec<f64>,
    #[serde(skip)]
    pub rsi_14: Vec<f64>,
    #[serde(skip)]
    pub macd: Vec<(f64, f64, f64)>,
    #[serde(skip)]
    pub bollinger: Vec<(f64, f64, f64)>,
    #[serde(skip)]
    pub volume_ma_20: Vec<f64>,
    #[serde(skip)]
    pub atr_14: Vec<f64>,
    pub latest: HashMap<String, f64>,
    pub signals: Vec<String>,
}

impl TechnicalIndicators {
    pub fn latest_values(&self) -> HashMap<String, f64> {
        let mut map = HashMap::new();
        if let Some(&v) = self.rsi_14.last() {
            map.insert("RSI".to_string(), v);
        }
        if let Some(&v) = self.sma_5.last() {
            map.insert("SMA5".to_string(), v);
        }
        if let Some(&v) = self.sma_25.last() {
            map.insert("SMA25".to_string(), v);
        }
        if let Some(&v) = self.sma_75.last() {
            map.insert("SMA75".to_string(), v);
        }
        if let Some(&v) = self.ema_12.last() {
            map.insert("EMA12".to_string(), v);
        }
        if let Some(&v) = self.ema_26.last() {
            map.insert("EMA26".to_string(), v);
        }
        if let Some(&(m, s, h)) = self.macd.last() {
            map.insert("MACD".to_string(), m);
            map.insert("MACD_Signal".to_string(), s);
            map.insert("MACD_Hist".to_string(), h);
        }
        if let Some(&(l, m, u)) = self.bollinger.last() {
            map.insert("BB_Lower".to_string(), l);
            map.insert("BB_Middle".to_string(), m);
            map.insert("BB_Upper".to_string(), u);
        }
        if let Some(&v) = self.volume_ma_20.last() {
            map.insert("Vol_MA20".to_string(), v);
        }
        if let Some(&v) = self.atr_14.last() {
            map.insert("ATR".to_string(), v);
        }
        map
    }

    fn detect_signals(&self, closes: &[f64], volumes: &[f64]) -> Vec<String> {
        let mut signals = Vec::new();

        if is_golden_cross(self) {
            signals.push("GoldenCross(SMA5/25)".to_string());
        }
        if is_dead_cross(self) {
            signals.push("DeadCross(SMA5/25)".to_string());
        }
        if let Some(true) = is_macd_signal_cross(self) {
            signals.push("MACD_BullishCross".to_string());
        }
        if let Some(&price) = closes.last() {
            match bollinger_breakout(self, price) {
                Some("upper_break") => signals.push("BB_UpperBreak".to_string()),
                Some("lower_break") => signals.push("BB_LowerBreak".to_string()),
                _ => {}
            }
        }
        if is_volume_spike(volumes, self, 2.0) {
            signals.push("VolumeSpike(2x)".to_string());
        }
        if let Some(&rsi) = self.rsi_14.last() {
            if rsi < 30.0 {
                signals.push(format!("RSI_Oversold({:.1})", rsi));
            } else if rsi > 70.0 {
                signals.push(format!("RSI_Overbought({:.1})", rsi));
            }
        }

        signals
    }
}

pub fn calculate_indicators(
    closes: &[f64],
    highs: &[f64],
    lows: &[f64],
    volumes: &[f64],
) -> Result<TechnicalIndicators> {
    let sma_5 = if closes.len() >= 5 {
        bulk::simple_moving_average(closes, 5)
    } else {
        vec![]
    };

    let sma_25 = if closes.len() >= 25 {
        bulk::simple_moving_average(closes, 25)
    } else {
        vec![]
    };

    let sma_75 = if closes.len() >= 75 {
        bulk::simple_moving_average(closes, 75)
    } else {
        vec![]
    };

    let ema_12 = if closes.len() >= 12 {
        bulk::exponential_moving_average(closes, 12)
    } else {
        vec![]
    };

    let ema_26 = if closes.len() >= 26 {
        bulk::exponential_moving_average(closes, 26)
    } else {
        vec![]
    };

    let rsi_14 = if closes.len() >= 14 {
        bulk::rsi(closes)
    } else {
        vec![]
    };

    let macd = if closes.len() >= 34 {
        bulk::macd(closes)
    } else {
        vec![]
    };

    let bollinger = if closes.len() >= 20 {
        bulk::bollinger_bands(closes)
    } else {
        vec![]
    };

    let volume_ma_20 = if volumes.len() >= 20 {
        bulk::simple_moving_average(volumes, 20)
    } else {
        vec![]
    };

    let atr_14 = calculate_atr(highs, lows, closes, 14);

    let mut indicators = TechnicalIndicators {
        sma_5,
        sma_25,
        sma_75,
        ema_12,
        ema_26,
        rsi_14,
        macd,
        bollinger,
        volume_ma_20,
        atr_14,
        latest: HashMap::new(),
        signals: Vec::new(),
    };

    indicators.latest = indicators.latest_values();
    indicators.signals = indicators.detect_signals(closes, volumes);

    Ok(indicators)
}

fn calculate_atr(highs: &[f64], lows: &[f64], closes: &[f64], period: usize) -> Vec<f64> {
    let len = highs.len();
    if len < 2 || len != lows.len() || len != closes.len() {
        return vec![];
    }

    let mut tr = Vec::with_capacity(len - 1);
    for i in 1..len {
        let prev_close = closes[i - 1];
        let hl = highs[i] - lows[i];
        let hc = (highs[i] - prev_close).abs();
        let lc = (lows[i] - prev_close).abs();
        tr.push(hl.max(hc).max(lc));
    }

    if tr.len() >= period {
        bulk::simple_moving_average(&tr, period)
    } else {
        vec![]
    }
}

pub fn is_golden_cross(indicators: &TechnicalIndicators) -> bool {
    let (sma5, sma25) = (&indicators.sma_5, &indicators.sma_25);
    if sma5.len() < 2 || sma25.len() < 2 {
        return false;
    }
    let len5 = sma5.len();
    let len25 = sma25.len();
    sma5[len5 - 1] > sma25[len25 - 1] && sma5[len5 - 2] <= sma25[len25 - 2]
}

pub fn is_dead_cross(indicators: &TechnicalIndicators) -> bool {
    let (sma5, sma25) = (&indicators.sma_5, &indicators.sma_25);
    if sma5.len() < 2 || sma25.len() < 2 {
        return false;
    }
    let len5 = sma5.len();
    let len25 = sma25.len();
    sma5[len5 - 1] < sma25[len25 - 1] && sma5[len5 - 2] >= sma25[len25 - 2]
}

pub fn is_macd_signal_cross(indicators: &TechnicalIndicators) -> Option<bool> {
    if indicators.macd.len() < 2 {
        return None;
    }
    let len = indicators.macd.len();
    let (macd_now, signal_now, _) = indicators.macd[len - 1];
    let (macd_prev, signal_prev, _) = indicators.macd[len - 2];
    Some(macd_now > signal_now && macd_prev <= signal_prev)
}

pub fn bollinger_breakout(
    indicators: &TechnicalIndicators,
    current_price: f64,
) -> Option<&'static str> {
    let (lower, _middle, upper) = *indicators.bollinger.last()?;
    if current_price > upper {
        Some("upper_break")
    } else if current_price < lower {
        Some("lower_break")
    } else {
        None
    }
}

pub fn is_volume_spike(volumes: &[f64], indicators: &TechnicalIndicators, threshold: f64) -> bool {
    match (volumes.last(), indicators.volume_ma_20.last()) {
        (Some(&current_vol), Some(&avg_vol)) => avg_vol > 0.0 && current_vol >= avg_vol * threshold,
        _ => false,
    }
}
