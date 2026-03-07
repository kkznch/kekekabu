## Purpose

評価結果からの Markdown レポート生成。Buy/Hold/Avoid カテゴリ別に集約。

## Requirements

### Requirement: Report generates Markdown from evaluations
The system SHALL generate a Markdown report from evaluations, grouped by Buy/Hold/Avoid categories.

#### Scenario: Report to stdout
- **WHEN** user runs `kabu report`
- **THEN** system outputs a Markdown report to stdout with today's evaluations

#### Scenario: Report to file
- **WHEN** user runs `kabu report -o report.md`
- **THEN** system writes the Markdown report to the specified file path

#### Scenario: Report for specific date
- **WHEN** user runs `kabu report --date 2026-03-07`
- **THEN** system generates a report using evaluations from the specified date

### Requirement: Report includes TA details
The system SHALL include technical analysis details (signals, indicator values) in the report for each stock.

#### Scenario: Stock with signals
- **WHEN** a stock has detected trading signals (golden cross, volume spike, etc.)
- **THEN** system includes the signals in the report under the stock's section
