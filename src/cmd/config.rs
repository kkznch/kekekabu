use anyhow::Result;

use crate::config;
use crate::spec;

pub fn init(force: bool) -> Result<()> {
    config::init_config(force)
}

pub fn validate() -> Result<()> {
    let use_color = std::io::IsTerminal::is_terminal(&std::io::stderr());
    let mut passed = 0;
    let total = 3;

    let config = config::AppConfig::load()?;
    eprintln!("{} Config", ok_mark(use_color));
    passed += 1;

    match spec::load_spec(&config.spec.path) {
        Ok(s) => {
            eprintln!("{} Spec \u{2014} {}", ok_mark(use_color), s.name);
            passed += 1;
        }
        Err(e) => {
            eprintln!("{} Spec \u{2014} {}", fail_mark(use_color), e);
            eprintln!("\n{}/{} checks passed.", passed, total);
            return Err(e);
        }
    }

    // Tachibana config check (optional — only required for execute non-dry-run)
    match &config.tachibana {
        Some(tc) => {
            let missing = tc.missing_fields();
            if missing.is_empty() {
                eprintln!("{} Tachibana", ok_mark(use_color));
                passed += 1;
            } else {
                eprintln!(
                    "{} Tachibana \u{2014} missing: {}",
                    warn_mark(use_color),
                    missing.join(", ")
                );
                eprintln!("  (required for `kabu execute` without --dry-run)");
                passed += 1; // warn, not fail — it's optional
            }
        }
        None => {
            eprintln!(
                "{} Tachibana \u{2014} not configured (optional, needed for `kabu execute`)",
                warn_mark(use_color),
            );
            passed += 1;
        }
    }

    eprintln!("\n{}/{} checks passed.", passed, total);
    Ok(())
}

fn ok_mark(color: bool) -> &'static str {
    if color {
        "\x1b[32m\u{2713}\x1b[0m"
    } else {
        "\u{2713}"
    }
}

fn warn_mark(color: bool) -> &'static str {
    if color {
        "\x1b[33m\u{26a0}\x1b[0m"
    } else {
        "\u{26a0}"
    }
}

fn fail_mark(color: bool) -> &'static str {
    if color {
        "\x1b[31m\u{2717}\x1b[0m"
    } else {
        "\u{2717}"
    }
}
