use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub api: ApiConfig,
    #[serde(default)]
    pub llm: LlmConfig,
    #[serde(default)]
    pub spec: SpecConfig,
    #[serde(default)]
    pub output: OutputConfig,
}

#[derive(Debug, Deserialize, Default)]
pub struct ApiConfig {
    pub jquants_api_key: Option<String>,
    pub anthropic_api_key: Option<String>,
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

impl LlmConfig {
    fn default_fetch() -> String {
        "cli-gemini".to_string()
    }
    fn default_eval() -> String {
        "cli-claude".to_string()
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
        "specs/default.yaml".to_string()
    }
}

#[derive(Debug, Deserialize)]
pub struct OutputConfig {
    #[serde(default = "OutputConfig::default_format")]
    pub default_format: String,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            default_format: "json".to_string(),
        }
    }
}

impl OutputConfig {
    fn default_format() -> String {
        "json".to_string()
    }
}

pub fn config_dir() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    let dir = PathBuf::from(home).join(".config/kktd");
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir)
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let mut config = Self::load_from_file()?;
        config.apply_env_overrides();
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
    }

    fn env_non_empty(key: &str) -> Option<String> {
        std::env::var(key).ok().filter(|v| !v.is_empty())
    }

    pub fn require_key(value: &Option<String>, key_name: &str) -> Result<String> {
        value.clone().ok_or_else(|| {
            anyhow::anyhow!(
                "{} is not set. Set it in ~/.config/kktd/config.toml or as an environment variable.",
                key_name
            )
        })
    }
}
