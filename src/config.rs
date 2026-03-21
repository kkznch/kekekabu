use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::PathBuf;

/// Environment selector for Tachibana Securities API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    #[default]
    Production,
    Demo,
}

#[derive(Debug, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub api: ApiConfig,
    #[serde(default)]
    pub llm: LlmConfig,
    #[serde(default)]
    pub spec: SpecConfig,
    #[serde(default)]
    pub tachibana: Option<TachibanaConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TachibanaConfig {
    pub user_id: Option<String>,
    pub password: Option<String>,
    pub second_password: Option<String>,
    #[serde(default = "TachibanaConfig::default_event_timeout_secs")]
    pub event_timeout_secs: u64,
    #[serde(default)]
    pub environment: Environment,
}

impl TachibanaConfig {
    fn default_event_timeout_secs() -> u64 {
        30
    }

    /// Return the auth URL for the configured environment.
    pub fn auth_url(&self) -> &'static str {
        match self.environment {
            Environment::Production => "https://kabuka.e-shiten.jp/e_api_v4r8/auth/",
            Environment::Demo => "https://demo.e-shiten.jp/e_api_v4r8/auth/",
        }
    }

    /// Validate that all required credentials are present.
    /// Returns a list of missing field names.
    pub fn missing_fields(&self) -> Vec<&'static str> {
        let mut missing = Vec::new();
        if self.user_id.as_ref().is_none_or(|s| s.is_empty()) {
            missing.push("user_id");
        }
        if self.password.as_ref().is_none_or(|s| s.is_empty()) {
            missing.push("password");
        }
        if self.second_password.as_ref().is_none_or(|s| s.is_empty()) {
            missing.push("second_password");
        }
        missing
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct ApiConfig {
    pub jquants_api_key: Option<String>,
    pub anthropic_api_key: Option<String>,
    pub gemini_api_key: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LlmConfig {
    #[serde(default = "LlmConfig::default_fetch")]
    pub fetch: String,
    #[serde(default = "LlmConfig::default_eval")]
    pub eval: String,
    pub eval_model: Option<String>,
    pub fetch_model: Option<String>,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            fetch: "cli-gemini".to_string(),
            eval: "cli-claude".to_string(),
            eval_model: None,
            fetch_model: None,
        }
    }
}

const VALID_BACKENDS: &[&str] = &["cli-gemini", "cli-claude", "api-gemini", "api-anthropic"];

impl LlmConfig {
    fn default_fetch() -> String {
        "cli-gemini".to_string()
    }
    fn default_eval() -> String {
        "cli-claude".to_string()
    }

    pub fn validate(&self) -> Result<()> {
        let mut errors = Vec::new();

        if !VALID_BACKENDS.contains(&self.fetch.as_str()) {
            errors.push(format!(
                "llm.fetch = \"{}\" is invalid. Valid: {}",
                self.fetch,
                VALID_BACKENDS.join(", ")
            ));
        }
        if !VALID_BACKENDS.contains(&self.eval.as_str()) {
            errors.push(format!(
                "llm.eval = \"{}\" is invalid. Valid: {}",
                self.eval,
                VALID_BACKENDS.join(", ")
            ));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            anyhow::bail!("Invalid config:\n  - {}", errors.join("\n  - "));
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SpecConfig {
    #[serde(default = "SpecConfig::default_path")]
    pub path: String,
}

impl Default for SpecConfig {
    fn default() -> Self {
        Self {
            path: Self::default_path(),
        }
    }
}

impl SpecConfig {
    fn default_path() -> String {
        "specs/template.toml".to_string()
    }
}

pub fn config_dir() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    Some(PathBuf::from(home).join(".config/kabu"))
}

/// Ensure config directory exists (call only on write paths).
fn ensure_config_dir() -> Option<PathBuf> {
    let dir = config_dir()?;
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir)
}

const CONFIG_TEMPLATE: &str = r#"# kabu configuration
# See: https://github.com/kkznch/kekekabu

[api]
# J-Quants API key (https://jpx.gitbook.io/j-quants-ja)
# jquants_api_key = ""

# Anthropic API key (for api-anthropic backend)
# anthropic_api_key = ""

# Google Gemini API key (for api-gemini backend)
# gemini_api_key = ""

[llm]
# LLM backend for fetch command (cli-gemini, cli-claude, api-gemini, api-anthropic)
fetch = "cli-gemini"

# LLM backend for eval command
eval = "cli-claude"

# Optional model overrides
# eval_model = ""
# fetch_model = ""

[spec]
# Investment spec file path (relative to config dir or absolute)
path = "specs/template.toml"

# [tachibana]
# Tachibana Securities e-Shiten API (https://www.e-shiten.jp/api/)
# Required for `kabu execute` (non dry-run mode)
# user_id = ""
# password = ""
# second_password = ""
# event_timeout_secs = 30
"#;

const SPEC_TEMPLATE: &str = r#"# Investment Spec: JP Core Value & Quality
# This file defines the investment strategy for kabu eval.

name = "JP Core Value & Quality"
version = "1.0"

[universe]
# Minimum market cap (JPY) — 100 億円
min_market_cap = 10_000_000_000.0
# Minimum average daily trading volume (shares)
min_daily_volume = 100_000.0

[[scoring.factors]]
name = "PBR"
weight = 0.2
description = "Price to Book Ratio. Lower is better (value)."

[[scoring.factors]]
name = "PER"
weight = 0.2
description = "Price to Earnings Ratio. Lower is better (value)."

[[scoring.factors]]
name = "ROE"
weight = 0.25
description = "Return on Equity. Higher is better (quality)."

[[scoring.factors]]
name = "Dividend Yield"
weight = 0.15
description = "Annual dividend yield. Higher is better (income)."

[[scoring.factors]]
name = "Technical Momentum"
weight = 0.2
description = "RSI, MACD, moving average trends."

[execution]
# Max position size as fraction of total portfolio (5%)
max_position_size = 0.05
# Stop loss trigger (negative = loss, -7%)
stop_loss = -0.07
# Trailing stop from high water mark (15%)
trailing_stop = 0.15
"#;

pub fn init_config(force: bool) -> Result<()> {
    let dir = ensure_config_dir().context("Failed to determine config directory")?;
    let config_path = dir.join("config.toml");
    let spec_dir = dir.join("specs");
    let spec_path = spec_dir.join("template.toml");

    if config_path.exists() && !force {
        anyhow::bail!(
            "Config already exists: {}\nUse --force to overwrite.",
            config_path.display()
        );
    }

    // Backup existing config before overwriting
    if config_path.exists() {
        let date = chrono::Local::now().format("%Y%m%d-%H%M%S");
        let backup_name = format!("config.{}.toml", date);
        let backup_path = dir.join(&backup_name);
        std::fs::copy(&config_path, &backup_path)
            .with_context(|| format!("Failed to backup config: {}", config_path.display()))?;
        eprintln!("Config backed up: {}", backup_path.display());
    }

    // Write config
    let overwritten = config_path.exists();
    std::fs::write(&config_path, CONFIG_TEMPLATE)
        .with_context(|| format!("Failed to write config: {}", config_path.display()))?;

    if overwritten {
        eprintln!("Config overwritten: {}", config_path.display());
    } else {
        eprintln!("Config created: {}", config_path.display());
    }

    // Write spec template
    std::fs::create_dir_all(&spec_dir)
        .with_context(|| format!("Failed to create specs dir: {}", spec_dir.display()))?;

    std::fs::write(&spec_path, SPEC_TEMPLATE)
        .with_context(|| format!("Failed to write spec: {}", spec_path.display()))?;
    eprintln!("Spec template written: {}", spec_path.display());

    eprintln!("Edit API keys: {}", config_path.display());

    Ok(())
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let mut config = Self::load_from_file()?;
        config.apply_env_overrides();
        config.llm.validate()?;
        Ok(config)
    }

    fn load_from_file() -> Result<Self> {
        let Some(dir) = config_dir() else {
            return Ok(Self::default());
        };

        let path = dir.join("config.toml");
        if !path.exists() {
            return Ok(Self::default());
        }

        let content =
            std::fs::read_to_string(&path).with_context(|| format!("Failed to read {:?}", path))?;

        toml::from_str(&content).with_context(|| format!("Failed to parse {:?}", path))
    }

    fn apply_env_overrides(&mut self) {
        if let Some(v) = Self::env_non_empty("JQUANTS_API_KEY") {
            self.api.jquants_api_key = Some(v);
        }
        if let Some(v) = Self::env_non_empty("ANTHROPIC_API_KEY") {
            self.api.anthropic_api_key = Some(v);
        }
        if let Some(v) = Self::env_non_empty("GEMINI_API_KEY") {
            self.api.gemini_api_key = Some(v);
        }

        // Tachibana env overrides
        let has_tachibana_env = Self::env_non_empty("TACHIBANA_USER_ID").is_some()
            || Self::env_non_empty("TACHIBANA_PASSWORD").is_some()
            || Self::env_non_empty("TACHIBANA_SECOND_PASSWORD").is_some()
            || Self::env_non_empty("TACHIBANA_ENVIRONMENT").is_some();

        if has_tachibana_env {
            let tc = self.tachibana.get_or_insert(TachibanaConfig {
                user_id: None,
                password: None,
                second_password: None,
                event_timeout_secs: TachibanaConfig::default_event_timeout_secs(),
                environment: Environment::default(),
            });
            if let Some(v) = Self::env_non_empty("TACHIBANA_USER_ID") {
                tc.user_id = Some(v);
            }
            if let Some(v) = Self::env_non_empty("TACHIBANA_PASSWORD") {
                tc.password = Some(v);
            }
            if let Some(v) = Self::env_non_empty("TACHIBANA_SECOND_PASSWORD") {
                tc.second_password = Some(v);
            }
            if let Some(v) = Self::env_non_empty("TACHIBANA_ENVIRONMENT") {
                match v.to_lowercase().as_str() {
                    "demo" => tc.environment = Environment::Demo,
                    "production" => tc.environment = Environment::Production,
                    _ => tracing::warn!(
                        value = %v,
                        "Invalid TACHIBANA_ENVIRONMENT value (expected: production, demo)"
                    ),
                }
            }
        }
    }

    fn env_non_empty(key: &str) -> Option<String> {
        std::env::var(key).ok().filter(|v| !v.is_empty())
    }

    pub fn require_key(value: &Option<String>, key_name: &str) -> Result<String> {
        value.clone().ok_or_else(|| {
            anyhow::anyhow!(
                "{} is not set. Set it in ~/.config/kabu/config.toml or as an environment variable.",
                key_name
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_valid_backends() {
        let llm = LlmConfig {
            fetch: "cli-gemini".to_string(),
            eval: "api-anthropic".to_string(),
            eval_model: None,
            fetch_model: None,
        };
        assert!(llm.validate().is_ok());
    }

    #[test]
    fn test_validate_invalid_backends() {
        let llm = LlmConfig {
            fetch: "invalid-backend".to_string(),
            eval: "also-invalid".to_string(),
            eval_model: None,
            fetch_model: None,
        };
        let err = llm.validate().unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("llm.fetch"));
        assert!(msg.contains("llm.eval"));
    }
}
