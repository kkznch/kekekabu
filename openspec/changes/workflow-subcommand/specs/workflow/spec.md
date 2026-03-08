## ADDED Requirements

### Requirement: Pipeline orchestration
The system SHALL provide a `kabu workflow run` subcommand that executes the discover → scan → fetch → eval pipeline in a single process with a single DB connection.

#### Scenario: Full pipeline run
- **WHEN** user runs `kabu workflow run`
- **THEN** the system SHALL execute discover, scan, fetch, and eval in sequence, sharing DB connection and config

#### Scenario: Skip steps
- **WHEN** user runs `kabu workflow run --skip discover`
- **THEN** the system SHALL skip the discover step and start from scan

### Requirement: Per-stock error isolation
The system SHALL isolate per-stock errors so that a failure in one stock does not prevent other stocks from being processed.

#### Scenario: Scan failure for one stock
- **WHEN** the J-Quants API returns an error for stock A during scan
- **THEN** stock A SHALL be marked as failed for scan, and stocks B and C SHALL continue through scan, fetch, and eval normally

#### Scenario: Fetch failure for one stock
- **WHEN** the LLM returns an error for stock B during fetch
- **THEN** stock B SHALL be marked as failed for fetch and skipped for eval, but stock A and C SHALL continue through eval normally

#### Scenario: Failed stock excluded from subsequent steps
- **WHEN** a stock fails in scan
- **THEN** that stock SHALL be excluded from fetch and eval steps (marked as Skipped)

### Requirement: Workflow report
The system SHALL output a `WorkflowReport` containing per-stock status for each pipeline step and a list of errors.

#### Scenario: Report with mixed results
- **WHEN** the pipeline completes with 3 stocks: A succeeded all steps, B failed scan, C failed fetch
- **THEN** the report SHALL show A as Success/Success/Success, B as Failed/Skipped/Skipped, C as Success/Success/Failed with error details

#### Scenario: All stocks succeed
- **WHEN** the pipeline completes with no errors
- **THEN** the report SHALL show all stocks as Success for all steps and an empty error list
