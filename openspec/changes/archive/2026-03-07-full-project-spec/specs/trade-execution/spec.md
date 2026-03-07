## ADDED Requirements

### Requirement: Execute processes today's evaluations
The system SHALL process today's evaluations and generate buy/sell signals based on decision and score.

#### Scenario: Buy signal for high-score Buy
- **WHEN** evaluation has decision="Buy" and score >= 70
- **THEN** system generates a buy signal action

#### Scenario: Buy signal skipped for low-score Buy
- **WHEN** evaluation has decision="Buy" and score < 70
- **THEN** system skips the buy signal with explanation "score too low"

#### Scenario: Sell signal for strong Avoid
- **WHEN** evaluation has decision="Avoid" and score <= 30
- **THEN** system generates a sell signal action to review existing positions

#### Scenario: Hold action
- **WHEN** evaluation has decision="Hold" or does not meet buy/sell thresholds
- **THEN** system generates a hold action

### Requirement: Execute supports dry run
The system SHALL default to dry-run mode, prefixing actions with "[DRY RUN]".

#### Scenario: Dry run mode
- **WHEN** user runs `kabu execute --dry-run true`
- **THEN** system outputs actions prefixed with "[DRY RUN]" without placing actual orders

### Requirement: Execute checks circuit breaker before processing
The system SHALL check the circuit breaker before processing any evaluations.

#### Scenario: Circuit breaker triggered
- **WHEN** circuit breaker detects unsafe market conditions
- **THEN** system aborts execute with `circuit_breaker_triggered: true` and lists reasons

#### Scenario: No evaluations for today
- **WHEN** no evaluations exist for today
- **THEN** system returns empty actions with informational log message
