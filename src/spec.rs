use anyhow::{Context, Result};
use std::path::Path;

pub struct InvestmentSpec {
    pub name: String,
    raw_content: String,
}

pub fn load_spec(path: &str) -> Result<InvestmentSpec> {
    let path = resolve_spec_path(path);
    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read spec file: {}", path.display()))?;

    // Validate as TOML and extract name
    let table: toml::Table = toml::from_str(&content)
        .with_context(|| format!("Invalid TOML in spec file: {}", path.display()))?;

    let name = table
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Spec file must have a 'name' field: {}", path.display()))?
        .to_string();

    Ok(InvestmentSpec {
        name,
        raw_content: content,
    })
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
    pub fn to_prompt_section(&self) -> String {
        format!(
            "## Investment Spec: {}\n\n```toml\n{}\n```",
            self.name, self.raw_content
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_spec_from_string() {
        // Simulate what load_spec does internally
        let toml_str = r#"
name = "Test Strategy"
version = "1.0"

[universe]
min_market_cap = 50000000000.0

[quantitative.value]
max_pbr = 1.2

[execution]
stop_loss = -0.07
"#;
        let table: toml::Table = toml::from_str(toml_str).unwrap();
        let name = table.get("name").unwrap().as_str().unwrap();
        assert_eq!(name, "Test Strategy");
    }

    #[test]
    fn test_missing_name_field() {
        let toml_str = r#"
version = "1.0"
[universe]
min_market_cap = 50000000000.0
"#;
        let table: toml::Table = toml::from_str(toml_str).unwrap();
        assert!(table.get("name").and_then(|v| v.as_str()).is_none());
    }

    #[test]
    fn test_invalid_toml() {
        let text = "this is not valid toml {{{";
        let result: Result<toml::Table, _> = toml::from_str(text);
        assert!(result.is_err());
    }

    #[test]
    fn test_to_prompt_section() {
        let spec = InvestmentSpec {
            name: "Test".to_string(),
            raw_content: "name = \"Test\"\n[execution]\nstop_loss = -0.07\n".to_string(),
        };
        let prompt = spec.to_prompt_section();
        assert!(prompt.contains("## Investment Spec: Test"));
        assert!(prompt.contains("stop_loss = -0.07"));
        assert!(prompt.contains("```toml"));
    }

    #[test]
    fn test_freeform_toml_structure() {
        // Verify that any TOML structure works as long as name exists
        let toml_str = r#"
name = "Custom Strategy"
version = "2.0"

[universe.liquidity]
min_avg_daily_volume_3m = 500_000_000
min_market_cap = 30_000_000_000

[universe.financial]
min_equity_ratio = 40.0

[quantitative.value]
max_pbr = 1.2
max_per = 15.0

[qualitative]
focus_points = """
1. Capital Efficiency
2. Competitive Moat
"""

[execution]
max_position_size = 0.05
"#;
        let table: toml::Table = toml::from_str(toml_str).unwrap();
        let name = table.get("name").unwrap().as_str().unwrap();
        assert_eq!(name, "Custom Strategy");
    }
}
