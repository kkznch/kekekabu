## Purpose

SQLite による永続化層。7 テーブル、冪等書き込み、rust_decimal による金額精度保証。

## Requirements

### Requirement: SQLite database with 7 tables
The system SHALL use SQLite (tokio-rusqlite, bundled) with 7 tables: stocks, prices, watchlist, evaluations, fetch_results, portfolio_positions, trades.

#### Scenario: Database initialization
- **WHEN** application starts (any command except init)
- **THEN** system creates the database file at ~/.config/kabu/kekekabu.db and ensures all 7 tables exist

### Requirement: Stocks table
The system SHALL store stock master data (ticker, name, sector) with ticker as unique key.

#### Scenario: Stock upsert
- **WHEN** stock data is saved with same ticker but updated name/sector
- **THEN** system updates the existing record (ON CONFLICT UPDATE)

### Requirement: Prices table
The system SHALL store daily OHLCV price data with (ticker, date) as unique key.

#### Scenario: Idempotent price insert
- **WHEN** same price data is inserted for same ticker and date
- **THEN** system ignores the duplicate (INSERT OR IGNORE)

### Requirement: Money stored as TEXT
The system SHALL store all monetary values as TEXT in SQLite, using rust_decimal::Decimal for precision.

#### Scenario: Decimal preservation
- **WHEN** a price of 2345.50 is saved and read back
- **THEN** system preserves the exact decimal value without floating-point rounding errors

### Requirement: Evaluations table with spec_hash
The system SHALL store evaluation results with SHA256 hash of the investment Spec used.

#### Scenario: Evaluation with spec tracking
- **WHEN** an evaluation is saved
- **THEN** system includes spec_hash field linking to the Spec version used for the evaluation

### Requirement: Fetch results table
The system SHALL store LLM-gathered information (category, content, source) per ticker.

#### Scenario: Multiple categories per ticker
- **WHEN** fetch gathers news, disclosure, and sentiment for a stock
- **THEN** system stores each as a separate row with appropriate category

### Requirement: Portfolio tables
The system SHALL use portfolio_positions (active positions with avg_cost) and trades (history with P&L) tables.

#### Scenario: Position lifecycle
- **WHEN** buy → partial sell → full sell sequence occurs
- **THEN** portfolio_positions tracks quantity/avg_cost and trades records each transaction with P&L on sells
