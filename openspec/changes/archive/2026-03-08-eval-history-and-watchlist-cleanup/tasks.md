## 1. B: Eval History Injection

- [x] 1.1 Add `get_recent_evaluations_by_stock(conn, stock_id, limit)` to `src/db/mod.rs` — returns Vec<Evaluation> ordered by evaluated_at DESC
- [x] 1.2 Add `history_section` parameter to `build_eval_prompt()` in `src/cmd/eval.rs` and format past evaluations as `## Past Evaluations` section
- [x] 1.3 In `eval::run()`, call `get_recent_evaluations_by_stock()` for each target and pass to `build_eval_prompt()`
- [x] 1.4 Add unit test for `get_recent_evaluations_by_stock` (empty, partial, full 3 results)

## 2. C: Watchlist Auto-Removal on Sell

- [x] 2.1 Modify `portfolio::sell()` to delete from watchlist and insert watchlist_event when `new_qty.is_zero()`, within the same transaction
- [x] 2.2 Add integration test: sell all shares → verify watchlist removal and event recording
- [x] 2.3 Add integration test: partial sell → verify watchlist NOT removed

## 3. Portfolio CLI Cleanup

- [x] 3.1 Remove `PortfolioCommand::Buy` and `PortfolioCommand::Sell` variants from main.rs
- [x] 3.2 Add `ShowCommand::Summary` and `ShowCommand::Trades { limit }` to main.rs
- [x] 3.3 Add `show::summary()` and `show::trades()` handlers in `src/cmd/show.rs`
- [x] 3.4 Remove `PortfolioCommand::Positions`, `PortfolioCommand::Summary`, `PortfolioCommand::Trades` (already covered by show)
- [x] 3.5 Remove entire `Portfolio` variant from `Command` enum if empty, clean up match arms in main()

## 4. Documentation

- [x] 4.1 Update CLAUDE.md commands section (remove portfolio buy/sell, add show summary/trades)
- [x] 4.2 Update README.md usage section and data flow diagram
- [x] 4.3 Update README.md dependency matrix (portfolio → show for R/W split)

## 5. Verification

- [x] 5.1 Run `just ci` — all tests pass, no clippy warnings, format check passes
