## ADDED Requirements

### Requirement: Eval generates investment judgments via LLM
The system SHALL build a comprehensive prompt with TA indicators, fetch results, and investment Spec, then send it to the eval LLM backend for investment judgment.

#### Scenario: Successful evaluation
- **WHEN** user runs `kabu eval` with stocks in the watchlist that have scan and fetch data
- **THEN** system generates a Buy/Hold/Avoid decision with score (0-100) and rationale for each stock

#### Scenario: Eval for specific tickers
- **WHEN** user runs `kabu eval 7203`
- **THEN** system evaluates only the specified ticker

### Requirement: Eval response format
The system SHALL parse the LLM response as JSON containing `decision` (Buy/Hold/Avoid), `score` (0-100), and `rationale` (summary, technical, risks).

#### Scenario: Valid eval response
- **WHEN** LLM returns a properly formatted JSON response
- **THEN** system extracts decision, score, and rationale fields

#### Scenario: Markdown-wrapped eval response
- **WHEN** LLM returns JSON wrapped in markdown code blocks
- **THEN** system extracts JSON from the code block before parsing

### Requirement: Eval includes investment Spec in prompt
The system SHALL include the investment Spec (universe filters, scoring factors, execution parameters) in the eval prompt.

#### Scenario: Spec included in prompt
- **WHEN** eval command runs with a configured Spec file
- **THEN** system loads the Spec YAML, converts it to a prompt section, and includes it in the LLM prompt

### Requirement: Eval results are persisted with Spec hash
The system SHALL save evaluation results to the database with the SHA256 hash of the Spec used.

#### Scenario: Evaluation saved with spec_hash
- **WHEN** an evaluation is completed
- **THEN** system saves ticker, name, decision, score, rationale, spec_hash, and evaluated_at to the `evaluations` table
