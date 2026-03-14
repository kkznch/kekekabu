## Why

End-to-end flow verification revealed 25 issues (3 HIGH, 10 MEDIUM, 12 LOW) across the entire pipeline. These range from data-loss bugs (partial fills silently dropped, portfolio re-buy UNIQUE violations) to operational risks (no WAL mode, no schema migration, non-atomic order+portfolio writes) and precision loss (SQL CAST on Decimal columns). Fixing these hardens the system for unattended production use via launchd.

## What Changes

- Partial fill (status "9") handling in EVENT I/F and settle phase
- Atomic order status + portfolio update in a single SQLite transaction
- `trade_cash_summary` uses Rust-side Decimal arithmetic instead of SQL CAST
- Schema migration via `PRAGMA user_version` (to be replaced with `refinery` crate)
- WAL mode + `busy_timeout` on DB init
- `prices` INSERT OR IGNORE → ON CONFLICT DO UPDATE
- Circuit breaker logout on trigger
- WebSocket EVENT I/F subscription message
- Eval dedup per ticker, fetch_results NULL url coalesce
- Workflow `--skip` validation, spec load warning, safe string slicing
- Ticker validation extended to 4-5 digits, discover list dedup
- Avoid action_type changed from `sell_signal` to `review`
- `config_dir()` split into read-only and write paths

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `database`: WAL mode, busy_timeout, prices ON CONFLICT UPDATE, schema version management via `PRAGMA user_version` + migration function
- `order-management`: partial status added, pending orders query includes partial
- `trade-execution`: Avoid → review action, CB logout, partial fill in settle, atomic order+portfolio transaction
- `tachibana-api`: partial fill query (status "9"), WebSocket subscribe message, partial fill notification
- `portfolio`: re-buy after sell reactivation, Decimal precision in trade_cash_summary
- `data-pipeline`: API timeout/retry (30s + exponential backoff), prices ON CONFLICT UPDATE
- `stock-discovery`: 4-5 digit ticker validation, add/keep dedup

## Impact

- **Code**: 11 source files modified (execute.rs, portfolio.rs, db/mod.rs, db/schema.rs, event.rs, jquants.rs, output.rs, config.rs, workflow.rs, discover.rs, report.rs)
- **Tests**: 2 integration test files updated (portfolio_test.rs, db_test.rs), all 109 tests passing
- **DB**: Existing databases gain WAL mode + busy_timeout on next startup; `user_version` set to 1
- **API**: `portfolio::buy`/`sell` async wrappers removed; `buy_sync`/`sell_sync` are now the public API
- **Dependencies**: No new crate dependencies (refinery planned as follow-up)
