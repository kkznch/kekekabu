# CLAUDE.md

## Project Overview

JP stock investment CLI tool (`kktd`). Rust 2024 edition.

5-command pipeline: `scan → fetch → eval → execute → report`

Currently implementing Phase 1 MVP: `scan`, `eval`, `watchlist`, `history`.

## Architecture

```
src/
  main.rs          CLI entry point (clap derive)
  lib.rs           Library root for integration tests
  config.rs        TOML config (~/.config/kktd/config.toml) + env overrides
  jquants.rs       J-Quants V2 API client
  indicators.rs    TA engine (RSI, MACD, BB, SMA, EMA, ATR, volume MA)
  output.rs        JSON (default) / human output formatting
  db/
    mod.rs         SQLite operations (stocks, prices, watchlist, evaluations)
    schema.rs      Table definitions (4 tables for Phase 1)
  llm/
    mod.rs         LlmBackend trait + factory
    api_anthropic.rs  Anthropic HTTP API
    cli_claude.rs     claude -p
    cli_gemini.rs     gemini -p
  cmd/
    mod.rs
    scan.rs        J-Quants fetch + TA indicators
    eval.rs        LLM investment evaluation (Buy/Hold/Avoid)
    watchlist.rs   Watchlist CRUD
```

## Key Design Decisions

- **JP market only** — no US stock support
- **SQLite** via `tokio-rusqlite` (bundled). DB at `~/.config/kktd/keketrade.db`
- **Money as TEXT** — `rust_decimal::Decimal` for precision
- **Idempotent writes** — `INSERT OR IGNORE` / `ON CONFLICT` everywhere
- **Default JSON output** — `--format human` for table display
- **Logs to stderr** — structured via `tracing`, so stdout is clean JSON

## Commands

```sh
kktd watchlist add 7203              # Add Toyota to watchlist
kktd watchlist list                  # List watchlist
kktd scan --days 60                  # Fetch prices + compute TA
kktd eval                            # Run LLM evaluation on all watchlist
kktd eval 7203 6758                  # Evaluate specific tickers
kktd history --limit 20             # Show past evaluations
```

## Development

```sh
cargo build                          # Build
cargo test                           # Run all tests (19 tests, in-memory SQLite)
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
```

Environment variables override config: `JQUANTS_API_KEY`, `ANTHROPIC_API_KEY`.

## Future Phases

- Phase 2: `fetch` (gemini-cli info gathering) + `report` (Markdown) + YAML Spec
- Phase 3: `execute` (Tachibana Securities API) + portfolio + circuit breaker
