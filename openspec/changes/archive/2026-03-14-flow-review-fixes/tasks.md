## 1. Database hardening

- [x] 1.1 Enable WAL mode and busy_timeout=5000ms in init_db
- [x] 1.2 Change prices INSERT OR IGNORE to ON CONFLICT DO UPDATE
- [x] 1.3 Add PRAGMA user_version schema version management with migrate() function
- [x] 1.4 Add SCHEMA_VERSION constant to db/schema.rs
- [x] 1.5 Split config_dir() into read-only and ensure_config_dir() for write paths

## 2. Partial fill and order management

- [x] 2.1 Parse status_code "9" as partial fill in tachibana event.rs
- [x] 2.2 Add WebSocket subscription message after EVENT I/F connect
- [x] 2.3 Expand list_pending_orders to include status="partial"
- [x] 2.4 Add partial fill scenario in settle phase (execute.rs)
- [x] 2.5 Add partial status to orders table status CHECK constraint

## 3. Atomic order+portfolio transaction

- [x] 3.1 Extract buy_sync/sell_sync from portfolio.rs as pub synchronous functions
- [x] 3.2 Create FillParams struct and update_order_and_record_fill in execute.rs
- [x] 3.3 Replace separate update_order_status + record_fill calls in settle phase
- [x] 3.4 Replace separate calls in WebSocket fill phase
- [x] 3.5 Remove async buy/sell wrappers from portfolio.rs
- [x] 3.6 Update portfolio_test.rs and db_test.rs to use buy_sync/sell_sync

## 4. Decimal precision and data integrity

- [x] 4.1 Replace SQL CAST in trade_cash_summary with Rust Decimal arithmetic
- [x] 4.2 Add decimal_to_f64 helper in db/mod.rs
- [x] 4.3 Coalesce NULL url to empty string in fetch_results for UNIQUE constraint
- [x] 4.4 Add warning log on decimal_str_to_f64 parse failure
- [x] 4.5 Fix eval score JSON schema minimum from -100 to 0

## 5. Safety and robustness

- [x] 5.1 Add Tachibana API logout on circuit breaker trigger
- [x] 5.2 Safe string slicing in output.rs (get(..N) instead of &s[..N])
- [x] 5.3 Add J-Quants API 30s timeout and 5xx/429 exponential backoff retry
- [x] 5.4 Fix report --date parameter to filter by evaluation date

## 6. Eval and discover improvements

- [x] 6.1 Deduplicate evals per ticker (latest only) to prevent duplicate orders
- [x] 6.2 Change Avoid action_type from "sell_signal" to "review"
- [x] 6.3 Extend ticker validation to accept 4-5 digits (ETF/REIT support)
- [x] 6.4 Add dedup between add/keep lists in discover
- [x] 6.5 Add workflow --skip step validation with warning for unknown values
- [x] 6.6 Add warning log on spec load failure in workflow (was silently ignored)

## 7. OpenSpec sync

- [x] 7.1 Update database/spec.md (WAL, prices ON CONFLICT, schema version)
- [x] 7.2 Update order-management/spec.md (partial status, pending query)
- [x] 7.3 Update trade-execution/spec.md (review action, CB logout, atomic settle)
- [x] 7.4 Update tachibana-api/spec.md (partial fill, WS subscribe)
- [x] 7.5 Update portfolio/spec.md (re-buy, Decimal precision)
- [x] 7.6 Update data-pipeline/spec.md (timeout/retry, ON CONFLICT)
- [x] 7.7 Update stock-discovery/spec.md (4-5 digit ticker, dedup)

## 8. Follow-up (not yet implemented)

- [ ] 8.1 Replace PRAGMA user_version with refinery crate for SQL-file-based migrations
