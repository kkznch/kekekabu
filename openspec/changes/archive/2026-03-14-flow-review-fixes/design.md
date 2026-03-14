## Context

The kekekabu CLI runs a 6-command pipeline (discover → scan → fetch → eval → execute → report) for JP stock investment automation. End-to-end flow verification uncovered 25 issues spanning data integrity, atomicity, precision, and operational robustness. The system targets unattended production use via launchd, making these fixes critical for reliability.

Current state: all fixes are implemented and passing CI (109 tests). This design documents the architectural decisions made.

## Goals / Non-Goals

**Goals:**
- Eliminate data-loss scenarios (partial fills, re-buy UNIQUE violations)
- Ensure atomicity for order status + portfolio updates
- Maintain Decimal precision throughout the data path
- Establish schema migration infrastructure for future changes
- Harden API interactions with timeouts and retries

**Non-Goals:**
- Full DB abstraction layer (Repository pattern) for future DB migration
- Tick size / lot size table for order price validation (Issue #17, deferred)
- Replacing SQLite with another database

## Decisions

### 1. Atomic order+portfolio via extracted sync functions

**Decision**: Extract `buy_sync`/`sell_sync` from portfolio.rs as synchronous functions taking `&rusqlite::Connection`, then combine with order status update in a single `conn.call()` transaction in execute.rs.

**Alternatives considered**:
- Nested `conn.call()` — not possible with tokio-rusqlite (each `conn.call` is a separate operation)
- DB-level triggers — too implicit, hard to test
- Saga/compensation pattern — overkill for single-process SQLite

**Rationale**: Extracting sync inner functions lets both standalone use (tests) and combined-transaction use (execute) share the same logic. The async wrappers were removed to avoid dead-code warnings in the binary target.

### 2. `PRAGMA user_version` for schema migration (interim)

**Decision**: Use SQLite's built-in `PRAGMA user_version` to track schema version. A `migrate()` function runs after `create_tables()` and applies version-gated ALTER TABLE statements.

**Follow-up**: Replace with `refinery` crate for SQL-file-based migrations. This provides better tooling (migration files, rollback, dry-run) without changing the fundamental approach.

### 3. Rust-side Decimal arithmetic for trade_cash_summary

**Decision**: Replace `CAST(price AS REAL) * CAST(quantity AS REAL)` SQL with Rust-side `Decimal` multiplication. Query raw TEXT columns and aggregate in Rust.

**Rationale**: Consistent with the "Money as TEXT + rust_decimal" design principle. SQL CAST to REAL loses precision for large values.

### 4. prices ON CONFLICT DO UPDATE (not INSERT OR IGNORE)

**Decision**: Changed prices table insert to `ON CONFLICT(stock_id, date) DO UPDATE` so J-Quants corrected data overwrites stale records.

**Rationale**: J-Quants may retroactively correct OHLCV data. INSERT OR IGNORE would silently discard corrections.

### 5. FillParams struct to reduce argument count

**Decision**: Introduced `FillParams` struct in execute.rs to bundle the 9 parameters of `update_order_and_record_fill` into 2 (conn + params), satisfying clippy's `too_many_arguments` lint.

## Risks / Trade-offs

- **[Risk] `buy_sync`/`sell_sync` are public API** → Integration tests need direct access. Acceptable since they're the canonical implementation. The async wrappers were purely convenience.
- **[Risk] `PRAGMA user_version` is SQLite-specific** → Migrating to another DB would require replacing the migration mechanism entirely. Acceptable given SQLite is the only target. Planned refinery adoption mitigates tooling concerns.
- **[Risk] Rust-side aggregation for trade_cash_summary loads all trades into memory** → For a personal investment tool with hundreds of trades, this is negligible. Would need pagination if trade volume reached millions.
- **[Trade-off] WAL mode increases disk usage** → WAL creates `-wal` and `-shm` files. Acceptable for better concurrent read performance and crash recovery.
