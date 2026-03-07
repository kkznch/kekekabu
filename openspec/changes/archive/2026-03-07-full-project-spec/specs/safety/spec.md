## ADDED Requirements

### Requirement: Circuit breaker blocks execute on abnormal individual stock moves
The system SHALL block trade execution when any watchlist stock has moved more than 30% in a single day.

#### Scenario: Individual stock circuit breaker
- **WHEN** a watchlist stock's daily price change exceeds 30%
- **THEN** system triggers circuit breaker, aborts execute, and reports the reason

### Requirement: Circuit breaker blocks execute on market-wide decline
The system SHALL block trade execution when more than 50% of watchlist stocks have declined more than 5%.

#### Scenario: Market-wide circuit breaker
- **WHEN** more than 50% of watchlist stocks show >5% daily decline
- **THEN** system triggers circuit breaker, aborts execute, and reports the reason

### Requirement: Circuit breaker reports reasons
The system SHALL report all circuit breaker trigger reasons in the execute output.

#### Scenario: Multiple triggers
- **WHEN** both individual and market-wide thresholds are exceeded
- **THEN** system includes all trigger reasons in `circuit_breaker_reasons` array

### Requirement: Execute defaults to dry run
The system SHALL default `--dry-run` to `true` to prevent accidental order placement.

#### Scenario: Default dry run
- **WHEN** user runs `kabu execute` without explicit `--dry-run` flag
- **THEN** system operates in dry-run mode (no actual orders placed)
