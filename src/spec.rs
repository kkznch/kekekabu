use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvestmentSpec {
    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub universe: UniverseFilter,
    #[serde(default)]
    pub scoring: ScoringConfig,
    #[serde(default)]
    pub execution: ExecutionConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniverseFilter {
    #[serde(default = "default_min_market_cap")]
    pub min_market_cap: f64,
    #[serde(default = "default_min_daily_volume")]
    pub min_daily_volume: f64,
}

impl Default for UniverseFilter {
    fn default() -> Self {
        Self {
            min_market_cap: default_min_market_cap(),
            min_daily_volume: default_min_daily_volume(),
        }
    }
}

fn default_min_market_cap() -> f64 {
    10_000_000_000.0
}
fn default_min_daily_volume() -> f64 {
    100_000.0
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScoringConfig {
    #[serde(default)]
    pub factors: Vec<ScoringFactor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringFactor {
    pub name: String,
    pub weight: f64,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    #[serde(default = "default_max_position_size")]
    pub max_position_size: f64,
    #[serde(default = "default_stop_loss")]
    pub stop_loss: f64,
    #[serde(default = "default_trailing_stop")]
    pub trailing_stop: f64,
}

fn default_max_position_size() -> f64 {
    0.05
}
fn default_stop_loss() -> f64 {
    -0.07
}
fn default_trailing_stop() -> f64 {
    0.15
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            max_position_size: default_max_position_size(),
            stop_loss: default_stop_loss(),
            trailing_stop: default_trailing_stop(),
        }
    }
}

pub fn load_spec(path: &str) -> Result<InvestmentSpec> {
    let path = resolve_spec_path(path);
    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read spec file: {}", path.display()))?;
    let spec: InvestmentSpec = toml::from_str(&content)
        .with_context(|| format!("Failed to parse spec TOML: {}", path.display()))?;
    spec.validate()?;
    Ok(spec)
}

pub fn spec_hash(path: &str) -> Result<String> {
    use sha2::{Digest, Sha256};
    let path = resolve_spec_path(path);
    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read spec file: {}", path.display()))?;
    let hash = Sha256::digest(content.as_bytes());
    Ok(format!("{:x}", hash))
}

fn resolve_spec_path(path: &str) -> std::path::PathBuf {
    let p = Path::new(path);
    if p.is_absolute() {
        p.to_path_buf()
    } else if let Some(dir) = crate::config::config_dir() {
        let config_relative = dir.join(path);
        if config_relative.exists() {
            return config_relative;
        }
        p.to_path_buf()
    } else {
        p.to_path_buf()
    }
}

impl InvestmentSpec {
    pub fn validate(&self) -> Result<()> {
        let mut errors = Vec::new();

        if self.name.is_empty() {
            errors.push("name must not be empty".to_string());
        }

        // Universe
        if self.universe.min_market_cap <= 0.0 {
            errors.push(format!(
                "universe.min_market_cap must be positive, got {}",
                self.universe.min_market_cap
            ));
        }
        if self.universe.min_daily_volume <= 0.0 {
            errors.push(format!(
                "universe.min_daily_volume must be positive, got {}",
                self.universe.min_daily_volume
            ));
        }

        // Scoring factors
        for (i, f) in self.scoring.factors.iter().enumerate() {
            if f.weight <= 0.0 || f.weight > 1.0 {
                errors.push(format!(
                    "scoring.factors[{}].weight must be in (0.0, 1.0], got {}",
                    i, f.weight
                ));
            }
        }

        // Execution
        if self.execution.max_position_size <= 0.0 || self.execution.max_position_size > 1.0 {
            errors.push(format!(
                "execution.max_position_size must be in (0.0, 1.0], got {}",
                self.execution.max_position_size
            ));
        }
        if self.execution.stop_loss >= 0.0 {
            errors.push(format!(
                "execution.stop_loss must be negative, got {}",
                self.execution.stop_loss
            ));
        }
        if self.execution.trailing_stop <= 0.0 || self.execution.trailing_stop > 1.0 {
            errors.push(format!(
                "execution.trailing_stop must be in (0.0, 1.0], got {}",
                self.execution.trailing_stop
            ));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            anyhow::bail!("Invalid investment spec:\n  - {}", errors.join("\n  - "));
        }
    }

    pub fn to_prompt_section(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!("## Investment Spec: {} (v{})\n\n", self.name, self.version));

        s.push_str("### Universe Filter\n");
        s.push_str(&format!("- Min market cap: {:.0}\n", self.universe.min_market_cap));
        s.push_str(&format!("- Min daily volume: {:.0}\n\n", self.universe.min_daily_volume));

        if !self.scoring.factors.is_empty() {
            s.push_str("### Scoring Factors\n");
            for f in &self.scoring.factors {
                s.push_str(&format!("- {} (weight: {:.1}): {}\n", f.name, f.weight, f.description));
            }
            s.push('\n');
        }

        s.push_str("### Execution Rules\n");
        s.push_str(&format!("- Max position size: {:.0}% of portfolio\n", self.execution.max_position_size * 100.0));
        s.push_str(&format!("- Stop loss: {:.0}%\n", self.execution.stop_loss * 100.0));
        s.push_str(&format!("- Trailing stop: {:.0}%\n", self.execution.trailing_stop * 100.0));

        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_spec_toml() {
        let toml_str = r#"
name = "Test Strategy"
version = "1.0"

[universe]
min_market_cap = 50000000000.0
min_daily_volume = 200000.0

[[scoring.factors]]
name = "PBR"
weight = 0.3
description = "Price to Book Ratio"

[[scoring.factors]]
name = "ROE"
weight = 0.3
description = "Return on Equity"

[execution]
max_position_size = 0.05
stop_loss = -0.07
trailing_stop = 0.15
"#;
        let spec: InvestmentSpec = toml::from_str(toml_str).unwrap();
        assert_eq!(spec.name, "Test Strategy");
        assert_eq!(spec.scoring.factors.len(), 2);
        assert!((spec.execution.stop_loss - (-0.07)).abs() < 0.001);
    }

    #[test]
    fn test_validate_valid_spec() {
        let spec = InvestmentSpec {
            name: "Test".to_string(),
            version: "1.0".to_string(),
            universe: UniverseFilter::default(),
            scoring: ScoringConfig {
                factors: vec![ScoringFactor {
                    name: "PBR".to_string(),
                    weight: 0.5,
                    description: String::new(),
                }],
            },
            execution: ExecutionConfig::default(),
        };
        assert!(spec.validate().is_ok());
    }

    #[test]
    fn test_validate_invalid_spec() {
        let spec = InvestmentSpec {
            name: String::new(),
            version: String::new(),
            universe: UniverseFilter {
                min_market_cap: -1.0,
                min_daily_volume: 0.0,
            },
            scoring: ScoringConfig {
                factors: vec![ScoringFactor {
                    name: "Bad".to_string(),
                    weight: 1.5,
                    description: String::new(),
                }],
            },
            execution: ExecutionConfig {
                max_position_size: 2.0,
                stop_loss: 0.1,
                trailing_stop: -0.5,
            },
        };
        let err = spec.validate().unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("name must not be empty"));
        assert!(msg.contains("min_market_cap must be positive"));
        assert!(msg.contains("min_daily_volume must be positive"));
        assert!(msg.contains("weight must be in"));
        assert!(msg.contains("max_position_size must be in"));
        assert!(msg.contains("stop_loss must be negative"));
        assert!(msg.contains("trailing_stop must be in"));
    }

    #[test]
    fn test_spec_to_prompt() {
        let spec = InvestmentSpec {
            name: "Test".to_string(),
            version: "1.0".to_string(),
            universe: UniverseFilter::default(),
            scoring: ScoringConfig { factors: vec![] },
            execution: ExecutionConfig::default(),
        };
        let prompt = spec.to_prompt_section();
        assert!(prompt.contains("Test"));
        assert!(prompt.contains("Stop loss: -7%"));
    }
}
