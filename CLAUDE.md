# CLAUDE.md

## Project Overview

JP stock investment CLI tool (`kktd`). Rust 2024 edition.

5-command pipeline: `scan → fetch → eval → execute → report`

All phases implemented. Tachibana Securities API integration is stubbed (pending API access).

## Architecture

```
src/
  main.rs            CLI entry point (clap derive)
  lib.rs             Library root for integration tests
  config.rs          TOML config (~/.config/kktd/config.toml) + env overrides
  jquants.rs         J-Quants V2 API client
  indicators.rs      TA engine (RSI, MACD, BB, SMA, EMA, ATR, volume MA)
  output.rs          JSON (default) / human output formatting
  portfolio.rs       Portfolio management (buy/sell, weighted avg cost, P&L)
  circuit_breaker.rs Safety checks (abnormal price moves, market-wide crash)
  spec.rs            Investment Spec YAML loader + SHA256 hashing
  db/
    mod.rs           SQLite operations (7 tables)
    schema.rs        Table definitions
  llm/
    mod.rs           LlmBackend trait + factory
    api_anthropic.rs Anthropic HTTP API
    cli_claude.rs    claude -p
    cli_gemini.rs    gemini -p
  cmd/
    scan.rs          J-Quants fetch + TA indicators
    fetch.rs         Gemini info gathering (news, disclosure, sentiment)
    eval.rs          LLM investment evaluation (Buy/Hold/Avoid)
    execute.rs       Trade execution (circuit breaker + order logic)
    report.rs        Markdown report generation
    watchlist.rs     Watchlist CRUD
```

## Key Design Decisions

- **JP market only** — no US stock support
- **SQLite** via `tokio-rusqlite` (bundled). DB at `~/.config/kktd/keketrade.db`
- **Money as TEXT** — `rust_decimal::Decimal` for precision
- **Idempotent writes** — `INSERT OR IGNORE` / `ON CONFLICT` everywhere
- **Default JSON output** — `--format human` for table display
- **Logs to stderr** — structured via `tracing`, so stdout is clean JSON
- **Circuit breaker** — blocks execute on >30% individual stock moves or >50% market decline

## Commands

```sh
# Pipeline
kktd scan --days 60                  # Fetch prices + compute TA
kktd fetch                           # Gather info via Gemini
kktd eval                            # LLM evaluation (Buy/Hold/Avoid)
kktd execute --dry-run               # Execute trades (dry run)
kktd report -o report.md             # Generate Markdown report

# Management
kktd watchlist add 7203              # Add to watchlist
kktd watchlist list                  # List watchlist
kktd portfolio buy 7203 --quantity 100 --price 2000
kktd portfolio sell 7203 --quantity 50 --price 2200
kktd portfolio positions             # Active positions
kktd portfolio summary               # Portfolio summary
kktd portfolio trades                # Trade history
kktd history --limit 20              # Past evaluations
```

## Automation (cron/launchd)

```sh
# Morning: scan → fetch → eval
kktd scan --days 60 && kktd fetch && kktd eval

# Market open: execute
kktd execute

# Evening: report
kktd report -o ~/reports/$(date +%Y-%m-%d).md
```

## Development

```sh
cargo build                          # Build
cargo test                           # Run all tests (32 tests, in-memory SQLite)
RUST_LOG=debug cargo run -- scan     # Run with debug logging
```

## Config

File: `~/.config/kktd/config.toml`

```toml
[api]
jquants_api_key = "..."
anthropic_api_key = "..."

[llm]
fetch = "cli-gemini"
eval = "cli-claude"

[spec]
path = "specs/jp-core-value-quality-v1.yaml"
```

Environment variables override config: `JQUANTS_API_KEY`, `ANTHROPIC_API_KEY`.

## DB Tables (7)

1. `stocks` — ticker master
2. `prices` — daily OHLCV
3. `watchlist` — monitored stocks
4. `evaluations` — LLM judgments (with spec_hash)
5. `fetch_results` — gathered information
6. `portfolio_positions` — active positions (weighted avg cost)
7. `trades` — trade history (with P&L)
