# CLAUDE.md

## Project Overview

JP stock investment CLI tool (`kabu`). Rust 2024 edition.

6-command pipeline: `discover → scan → fetch → eval → execute → report`

All phases implemented. Tachibana Securities API integration is stubbed (pending API access).

## Architecture

```
src/
  main.rs            CLI entry point (clap derive)
  lib.rs             Library root for integration tests
  config.rs          TOML config (~/.config/kabu/config.toml) + env overrides + validation
  jquants.rs         J-Quants V2 API client
  indicators.rs      TA engine (RSI, MACD, BB, SMA, EMA, ATR, volume MA)
  output.rs          JSON (default) / human output formatting
  portfolio.rs       Portfolio management (buy/sell, weighted avg cost, P&L)
  circuit_breaker.rs Safety checks (abnormal price moves, market-wide crash)
  spec.rs            Investment Spec TOML loader + SHA256 hashing
  db/
    mod.rs           SQLite operations (8 tables)
    schema.rs        Table definitions
  llm/
    mod.rs           LlmBackend trait + factory
    api_anthropic.rs Anthropic HTTP API
    cli_claude.rs    claude -p
    cli_gemini.rs    gemini -p
  cmd/
    discover.rs      LLM stock discovery + watchlist management
    scan.rs          J-Quants fetch + TA indicators
    fetch.rs         Gemini info gathering (news, disclosure, sentiment)
    eval.rs          LLM investment evaluation (Hunting: Buy/Avoid, Farming: Hold/Sell) + history injection
    execute.rs       Trade execution (circuit breaker + Buy/Sell signals)
    report.rs        Markdown report generation
    show.rs          DB viewer (watchlist, events, positions, evaluations, stocks, tables, summary, trades)
    config.rs        Config init + validate handlers
```

## Key Design Decisions

- **JP market only** — no US stock support
- **SQLite** via `tokio-rusqlite` (bundled). DB at `~/.config/kabu/kekekabu.db`
- **Money as TEXT** — `rust_decimal::Decimal` for precision
- **Idempotent writes** — `INSERT OR IGNORE` / `ON CONFLICT` everywhere
- **Default JSON output** — `--format human` for table display
- **Logs to stderr** — structured via `tracing`, so stdout is clean JSON
- **Circuit breaker** — blocks execute on >30% individual stock moves or >50% market decline
- **Eval history** — injects last 3 evaluations per stock into LLM prompt to prevent flip-flopping
- **Watchlist auto-cleanup** — auto-removes stock from watchlist when position is fully sold

## Commands

```sh
# Pipeline
kabu discover                        # LLM stock discovery → watchlist
kabu scan --days 60                  # Fetch prices + compute TA
kabu fetch                           # Gather info via Gemini
kabu eval                            # LLM evaluation (Hunting + Farming)
kabu execute --dry-run               # Execute trades (dry run)
kabu report -o report.md             # Generate Markdown report

# Config
kabu config init                     # Initialize config + spec template
kabu config init --force             # Overwrite existing config
kabu config validate                 # Validate config + spec

# DB viewer (no config required)
kabu show watchlist                  # Current watchlist
kabu show events                     # Watchlist change history
kabu show events --ticker 7203       # Filter by ticker
kabu show positions                  # Active positions
kabu show evaluations                # Past evaluations
kabu show stocks                     # Registered stocks
kabu show tables                     # Table row counts
kabu show summary                    # Portfolio summary
kabu show trades                     # Trade history
```

## Automation (cron/launchd)

```sh
# Morning: discover → scan → fetch → eval
kabu discover && kabu scan --days 60 && kabu fetch && kabu eval

# Market open: execute
kabu execute

# Evening: report
kabu report -o ~/reports/$(date +%Y-%m-%d).md
```

## Development

```sh
aqua install                         # Install tools (just, etc.)
just build                           # Build
just test                            # Run all tests (40 tests, in-memory SQLite)
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
path = "specs/jp-core-value-quality-v1.toml"
```

Environment variables override config: `JQUANTS_API_KEY`, `ANTHROPIC_API_KEY`.

## DB Tables (8)

1. `stocks` — ticker master
2. `prices` — daily OHLCV
3. `watchlist` — monitored stocks (managed by discover)
4. `watchlist_events` — watchlist change log (add/remove/keep with reasons)
5. `evaluations` — LLM judgments (with spec_hash)
6. `fetch_results` — gathered information
7. `portfolio_positions` — active positions (weighted avg cost)
8. `trades` — trade history (with P&L)
