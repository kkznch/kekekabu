## Purpose

ポートフォリオ管理。売買記録、加重平均コスト計算、P&L 算出、ポジション追跡。

## Requirements

### Requirement: Record buy transactions
The system SHALL record buy transactions and update portfolio positions with weighted average cost.

#### Scenario: First buy creates new position
- **WHEN** user runs `kktd portfolio buy 7203 --quantity 100 --price 2000`
- **THEN** system creates a new position with quantity=100, avg_cost=2000

#### Scenario: Additional buy updates average cost
- **WHEN** user buys 100 shares at 2000, then buys 100 more at 2200
- **THEN** system updates position to quantity=200, avg_cost=2100 (weighted average)

### Requirement: Record sell transactions
The system SHALL record sell transactions, calculate P&L, and update positions.

#### Scenario: Partial sell
- **WHEN** user sells 50 shares at 2200 from a position of 100 shares at avg_cost 2000
- **THEN** system updates position to quantity=50 and records trade with pnl=(2200-2000)*50=10000

#### Scenario: Full sell closes position
- **WHEN** user sells all shares of a position
- **THEN** system sets is_active=0 on the position (closed)

### Requirement: List active positions
The system SHALL list all active portfolio positions.

#### Scenario: Positions output
- **WHEN** user runs `kktd portfolio positions`
- **THEN** system outputs active positions as JSON (ticker, name, quantity, avg_cost, unrealized_pnl)

### Requirement: Portfolio summary
The system SHALL provide an aggregated portfolio summary.

#### Scenario: Summary calculation
- **WHEN** user runs `kktd portfolio summary`
- **THEN** system outputs position_count, total_invested, total_current_value, total_unrealized_pnl, total_unrealized_pnl_pct

### Requirement: Trade history
The system SHALL provide a list of past trades.

#### Scenario: Trade history with limit
- **WHEN** user runs `kktd portfolio trades --limit 20`
- **THEN** system outputs the 20 most recent trades (ticker, side, date, quantity, price, pnl)
