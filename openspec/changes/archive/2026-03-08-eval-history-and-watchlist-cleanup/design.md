## Context

The pipeline runs fully automated: `discover → scan → fetch → eval → execute`. The eval command is stateless — each run makes independent LLM calls per stock with no memory of previous decisions. This causes potential sell→re-buy oscillation: a stock sold today can be re-purchased tomorrow if the LLM flips its judgment.

Currently, `portfolio buy/sell` CLI commands exist for manual trade recording, but manual trading is not supported in the full-automation model. The `portfolio positions/summary/trades` read-only commands duplicate functionality that belongs in `show`.

### Current code state

- `eval.rs:build_eval_prompt()` constructs prompts per stock with no history context
- `portfolio::sell()` detects position=0 (line 157) and marks position inactive, but does not touch watchlist
- `db::watchlist_remove()` and `db::save_watchlist_event()` exist and work
- `db::list_evaluations()` exists but returns all evaluations, not per-stock
- `show` already has `positions` subcommand; `summary` and `trades` are only under `portfolio`

## Goals / Non-Goals

**Goals:**
- Give eval LLM memory of recent decisions to enable hysteresis-aware judgment (B)
- Auto-remove stocks from watchlist when position is fully closed to prevent re-buy loop (C)
- Remove manual `portfolio buy/sell` CLI commands
- Consolidate all read-only queries under `show`

**Non-Goals:**
- Implementing Tachibana API integration (约定検知 is a separate future change)
- Changing eval's decision logic or scoring thresholds
- Modifying discover's watchlist management behavior
- Adding new DB tables or schema changes

## Decisions

### 1. Eval history query: per-stock, last 3

**Choice**: Add `db::get_recent_evaluations_by_ticker(conn, stock_id, 3)` that returns the most recent 3 evaluations for a specific stock.

**Why not reuse `list_evaluations`**: It returns all stocks mixed together and would require client-side filtering. A dedicated query is simpler and more efficient.

**Prompt format**: Append a `## Past Evaluations` section to the eval prompt with a compact summary:
```
## Past Evaluations (most recent first)
- 2026-03-07: Buy (score: 72) — Strong catalyst, meets spec
- 2026-03-06: Avoid (score: 35) — High risk, poor timing
- 2026-03-05: Buy (score: 68) — Value opportunity
```

This adds ~200-300 characters per stock. Since eval makes individual LLM calls per stock, there is no prompt explosion risk.

**Anti-anchoring instruction**: The prompt must explicitly instruct the LLM that past evaluations are reference context only — the current decision must be based on current data and fundamentals. If the decision differs from recent history, the LLM should explain why.

### 2. Watchlist auto-removal: hook inside `portfolio::sell()`

**Choice**: Add watchlist removal logic inside `portfolio::sell()`, within the existing SQLite transaction, when `new_qty.is_zero()`.

**Why inside the transaction**: Ensures atomicity — position deactivation and watchlist removal either both succeed or both fail.

**Why not in execute.rs**: Execute only generates signals. The actual position update happens in `portfolio::sell()`, which is the single source of truth for position state.

**Event recording**: Call `save_watchlist_event` with action `"auto-removed-on-sell"` after the transaction commits (outside the rusqlite closure, using the async db functions).

**Alternative considered**: Triggering in discover — rejected because it leaves the stock in watchlist (and eval target pool) until the next discover run, wasting one eval cycle.

### 3. Portfolio CLI cleanup

**Choice**: Remove `PortfolioCommand::Buy` and `PortfolioCommand::Sell` variants from the CLI enum. Keep `portfolio::buy()` and `portfolio::sell()` as internal functions in `portfolio.rs` (execute will call them when Tachibana API is integrated).

Remove `PortfolioCommand::Positions`, `PortfolioCommand::Summary`, `PortfolioCommand::Trades` and add `ShowCommand::Summary` and `ShowCommand::Trades` instead. `ShowCommand::Positions` already exists.

If the `PortfolioCommand` enum becomes empty, remove the entire `Portfolio` variant from `Command`.

### 4. Watchlist removal implementation detail

The `portfolio::sell()` function uses `conn.call(move |conn| { ... })` with a synchronous rusqlite closure. Watchlist removal requires the ticker string which is available inside the closure.

**Approach**: Perform watchlist removal inside the same `conn.call` closure, within the same transaction. This avoids needing a second async call and keeps everything atomic.

Steps inside the closure when `new_qty.is_zero()`:
1. Deactivate position (existing code)
2. Delete from watchlist: `DELETE FROM watchlist WHERE stock_id IN (SELECT id FROM stocks WHERE ticker = ?1)`
3. Insert watchlist_event: `INSERT INTO watchlist_events (ticker, action, reason) VALUES (?1, 'auto-removed-on-sell', 'Position closed')`

## Risks / Trade-offs

- **[Risk] History may bias LLM toward consistency** → Mitigation: Only 3 entries, and prompt instructs LLM to change decision when fundamentals change. Monitor for over-consistency in practice.
- **[Risk] Auto-removal prevents manual re-watch** → Mitigation: Users can re-add via next discover cycle. This is the intended behavior.
- **[Risk] Removing portfolio CLI before Tachibana integration means no way to record trades** → Mitigation: Accepted — no manual trading in full-automation mode. Trades will only be recorded when execute integrates with Tachibana API.
- **[Trade-off] Watchlist removal inside sell transaction couples portfolio and watchlist concerns** → Accepted for atomicity. The coupling is intentional and documented.
