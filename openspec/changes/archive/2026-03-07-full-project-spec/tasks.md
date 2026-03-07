## 1. Config & Init

- [x] 1.1 AppConfig struct with [api], [llm], [spec], [output] sections
- [x] 1.2 TOML config loading from ~/.config/kabu/config.toml
- [x] 1.3 Environment variable overrides (JQUANTS_API_KEY, ANTHROPIC_API_KEY, GEMINI_API_KEY)
- [x] 1.4 `kabu init` command with config.toml + specs/template.yaml generation
- [x] 1.5 `--force` flag for config overwrite, template always regenerated

## 2. Database

- [x] 2.1 SQLite setup with tokio-rusqlite (bundled)
- [x] 2.2 Schema: stocks, prices, watchlist, evaluations, fetch_results, portfolio_positions, trades
- [x] 2.3 Idempotent writes (INSERT OR IGNORE / ON CONFLICT)
- [x] 2.4 Money as TEXT with rust_decimal::Decimal
- [x] 2.5 CRUD operations for all 7 tables
- [x] 2.6 DB integration tests (7 tests)

## 3. Data Pipeline (scan)

- [x] 3.1 J-Quants V2 API client (Bearer token auth)
- [x] 3.2 Daily OHLCV price data fetching
- [x] 3.3 Rate limiting (1 second between API calls)
- [x] 3.4 Technical indicators: SMA(5/25/75), EMA(12/26), RSI(14), MACD, BB(20,2), ATR(14), Volume MA(20)
- [x] 3.5 Signal detection: golden/dead cross, MACD cross, BB breakout, volume spike, RSI oversold/overbought
- [x] 3.6 Indicator tests (6 tests)

## 4. LLM Integration

- [x] 4.1 LlmBackend trait with send_message(prompt, max_tokens)
- [x] 4.2 Factory function create_backend() with ApiConfig parameter
- [x] 4.3 api-anthropic backend (Anthropic Messages API)
- [x] 4.4 api-gemini backend (Gemini generateContent API)
- [x] 4.5 cli-claude backend (claude -p)
- [x] 4.6 cli-gemini backend (gemini -p)
- [x] 4.7 Model override support (eval_model, fetch_model)

## 5. Info Gathering (fetch)

- [x] 5.1 Structured prompt for news/disclosure/sentiment/competitor info
- [x] 5.2 JSON response parsing with markdown code block extraction
- [x] 5.3 Persist fetch_results to database
- [x] 5.4 Fetch tests (2 tests)

## 6. Investment Evaluation (eval)

- [x] 6.1 Comprehensive prompt with TA indicators + fetch results + Spec
- [x] 6.2 JSON response parsing (decision, score, rationale)
- [x] 6.3 Investment Spec YAML loader with SHA256 hashing
- [x] 6.4 Spec to_prompt_section() for embedding in eval prompt
- [x] 6.5 Persist evaluations with spec_hash
- [x] 6.6 Eval tests (3 tests) + Spec tests (2 tests)

## 7. Trade Execution (execute)

- [x] 7.1 Circuit breaker check before processing
- [x] 7.2 Buy/sell signal generation based on decision + score thresholds
- [x] 7.3 Dry-run mode (default: true)
- [x] 7.4 Tachibana API integration stub (pending API access)

## 8. Reporting

- [x] 8.1 Markdown report generation from evaluations
- [x] 8.2 Group by Buy/Hold/Avoid categories
- [x] 8.3 Include TA details per stock
- [x] 8.4 Output to stdout or file (-o flag)
- [x] 8.5 Date filter (--date flag)

## 9. Watchlist

- [x] 9.1 watchlist add (with optional --notes)
- [x] 9.2 watchlist remove
- [x] 9.3 watchlist list
- [x] 9.4 Idempotent add (INSERT OR IGNORE)

## 10. Portfolio

- [x] 10.1 Buy with weighted average cost calculation
- [x] 10.2 Sell with P&L calculation
- [x] 10.3 Position tracking (is_active flag)
- [x] 10.4 Portfolio summary (position_count, total_invested, total_value, pnl)
- [x] 10.5 Trade history with limit
- [x] 10.6 Portfolio tests (5 tests)

## 11. Safety

- [x] 11.1 Individual stock circuit breaker (>30% daily move)
- [x] 11.2 Market-wide circuit breaker (>50% of watchlist down >5%)
- [x] 11.3 Circuit breaker reason reporting

## 12. Output & CLI

- [x] 12.1 OutputFormat enum (Json/Human) with clap derive
- [x] 12.2 HumanDisplay trait implementations for all output types
- [x] 12.3 JSON output to stdout, logs to stderr (tracing)
- [x] 12.4 Global --format flag

## 13. Tooling

- [x] 13.1 aqua.yaml with casey/just
- [x] 13.2 justfile with build/test/lint/ci tasks
- [x] 13.3 README.md with full documentation
- [x] 13.4 CLAUDE.md with architecture overview
