## Purpose

LLM を使った銘柄情報収集（ニュース、開示、センチメント、競合分析）。

## Requirements

### Requirement: Fetch gathers stock information via LLM
The system SHALL use the configured fetch LLM backend to gather latest information (news, disclosures, sentiment, competitor analysis) for watchlist stocks.

#### Scenario: Successful fetch for all watchlist stocks
- **WHEN** user runs `kabu fetch` with stocks in the watchlist
- **THEN** system sends a structured prompt for each stock to the fetch LLM backend and saves results to `fetch_results` table

#### Scenario: Fetch for specific tickers
- **WHEN** user runs `kabu fetch 7203 6758`
- **THEN** system fetches information only for the specified tickers (if they are in the watchlist)

### Requirement: Fetch results are structured
The system SHALL parse LLM responses as structured JSON containing news, disclosures, sentiment, and competitor information.

#### Scenario: Valid LLM response
- **WHEN** LLM returns a JSON response with expected fields
- **THEN** system parses and saves individual items to the `fetch_results` table with category and content

#### Scenario: Markdown-wrapped JSON response
- **WHEN** LLM returns JSON wrapped in markdown code blocks (```json ... ```)
- **THEN** system extracts the JSON from the code block and parses it correctly

### Requirement: Fetch results are persisted
The system SHALL save fetch results to the database for use by the eval command.

#### Scenario: Results saved with timestamp
- **WHEN** fetch results are saved
- **THEN** each result includes ticker, category, content, source, and fetched_at timestamp
