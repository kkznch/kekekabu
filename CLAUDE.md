# CLAUDE.md

## Project Overview

JP stock investment CLI tool (`kabu`). Rust 2024 edition.

5-command pipeline: `scan → fetch → eval → execute → report`

All phases implemented. Tachibana Securities API integration is stubbed (pending API access).

## Architecture

```
src/
  main.rs            CLI entry point (clap derive)
  lib.rs             Library root for integration tests
  config.rs          TOML config (~/.config/kabu/config.toml) + env overrides
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
- **SQLite** via `tokio-rusqlite` (bundled). DB at `~/.config/kabu/kekekabu.db`
- **Money as TEXT** — `rust_decimal::Decimal` for precision
- **Idempotent writes** — `INSERT OR IGNORE` / `ON CONFLICT` everywhere
- **Default JSON output** — `--format human` for table display
- **Logs to stderr** — structured via `tracing`, so stdout is clean JSON
- **Circuit breaker** — blocks execute on >30% individual stock moves or >50% market decline

## Commands

```sh
# Pipeline
kabu scan --days 60                  # Fetch prices + compute TA
kabu fetch                           # Gather info via Gemini
kabu eval                            # LLM evaluation (Buy/Hold/Avoid)
kabu execute --dry-run               # Execute trades (dry run)
kabu report -o report.md             # Generate Markdown report

# Management
kabu watchlist add 7203              # Add to watchlist
kabu watchlist list                  # List watchlist
kabu portfolio buy 7203 --quantity 100 --price 2000
kabu portfolio sell 7203 --quantity 50 --price 2200
kabu portfolio positions             # Active positions
kabu portfolio summary               # Portfolio summary
kabu portfolio trades                # Trade history
kabu history --limit 20              # Past evaluations
```

## Automation (cron/launchd)

```sh
# Morning: scan → fetch → eval
kabu scan --days 60 && kabu fetch && kabu eval

# Market open: execute
kabu execute

# Evening: report
kabu report -o ~/reports/$(date +%Y-%m-%d).md
```

## Development

```sh
aqua install                         # Install tools (just, etc.)
just build                           # Build
just test                            # Run all tests (32 tests, in-memory SQLite)
just lint                            # Clippy lints
just ci                              # fmt-check + lint + test
just --list                          # Show all available tasks
```

## Config

File: `~/.config/kabu/config.toml`

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
