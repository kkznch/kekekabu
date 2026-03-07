## Purpose

監視銘柄の管理（追加・削除・一覧）。パイプラインの対象銘柄を定義。

## Requirements

### Requirement: Add stock to watchlist
The system SHALL allow adding a stock to the watchlist by ticker code.

#### Scenario: Add new stock
- **WHEN** user runs `kabu watchlist add 7203`
- **THEN** system adds ticker 7203 to the watchlist and logs confirmation to stderr

#### Scenario: Add with notes
- **WHEN** user runs `kabu watchlist add 7203 --notes "Toyota Motor"`
- **THEN** system adds the stock with the provided notes

#### Scenario: Idempotent add
- **WHEN** user adds a stock that is already in the watchlist
- **THEN** system does not create a duplicate entry (INSERT OR IGNORE)

### Requirement: Remove stock from watchlist
The system SHALL allow removing a stock from the watchlist by ticker code.

#### Scenario: Remove existing stock
- **WHEN** user runs `kabu watchlist remove 7203`
- **THEN** system removes ticker 7203 from the watchlist

### Requirement: List watchlist stocks
The system SHALL list all stocks in the watchlist.

#### Scenario: List with JSON output
- **WHEN** user runs `kabu watchlist list`
- **THEN** system outputs watchlist items as JSON array to stdout (ticker, name, sector, notes)

#### Scenario: List with human output
- **WHEN** user runs `kabu watchlist list --format human`
- **THEN** system outputs watchlist items in a formatted table
