## Why

The fully automated pipeline (discover → scan → fetch → eval → execute) has a critical sell→re-buy loop problem. Because eval is stateless (no memory of previous decisions), it can flip-flop between Buy and Sell for the same stock across consecutive runs. Additionally, once a stock is sold, it remains in the watchlist indefinitely, wasting eval cycles and increasing the risk of re-purchase. Separately, the `portfolio buy/sell` CLI commands exist for manual trading, which contradicts the full-automation design — manual trading is not supported.

## What Changes

- **B: Eval judgment history injection** — Pass the most recent 3 evaluation results (decision, score, rationale) for each stock into the eval LLM prompt, giving it memory of past decisions to prevent flip-flopping
- **C: Watchlist auto-removal on sell** — When a stock's portfolio position reaches zero after a sell, automatically remove it from the watchlist and record a `auto-removed-on-sell` event
- **BREAKING**: Remove `portfolio buy` and `portfolio sell` CLI subcommands (internal functions retained for future Tachibana API integration)
- Move `portfolio summary` and `portfolio trades` to `show summary` and `show trades` subcommands

## Capabilities

### New Capabilities
- `eval-history`: Inject recent evaluation history into eval LLM prompts for all target stocks (up to 3 most recent per stock)
- `watchlist-auto-cleanup`: Automatically remove stocks from watchlist when portfolio position reaches zero after a sell

### Modified Capabilities
- `investment-evaluation`: Eval prompt now includes past judgment history to enable hysteresis-aware decisions
- `watchlist`: Watchlist entries can be auto-removed on sell (new removal trigger besides discover)
- `portfolio`: Remove manual buy/sell CLI commands; retain internal functions only
- `trade-execution`: Document that execute will call portfolio functions internally when Tachibana API is integrated

## Impact

- **Code**: `src/cmd/eval.rs` (prompt modification), `src/db/mod.rs` (new query), `src/portfolio.rs` (sell hook + watchlist removal), `src/main.rs` (CLI restructuring), `src/cmd/show.rs` (new subcommands)
- **DB**: No schema changes — uses existing `evaluations` and `watchlist` tables
- **CLI**: `portfolio buy/sell` removed, `show summary/trades` added
- **Breaking**: Users relying on `kabu portfolio buy/sell` commands must stop using them
