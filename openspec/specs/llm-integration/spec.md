## Purpose

LLM バックエンド抽象化。4 バックエンド（api-anthropic, api-gemini, cli-claude, cli-gemini）の統一インターフェース。

## Requirements

### Requirement: LLM backend abstraction
The system SHALL provide a `LlmBackend` trait with a `send_message(prompt, max_tokens)` method that all backends implement.

#### Scenario: Backend selection via config
- **WHEN** config specifies `llm.fetch = "cli-gemini"` or `llm.eval = "cli-claude"`
- **THEN** system creates the corresponding backend via factory function `create_backend()`

### Requirement: api-anthropic backend
The system SHALL support Anthropic Messages API as an LLM backend (`api-anthropic`).

#### Scenario: Successful API call
- **WHEN** `api-anthropic` backend sends a message with valid API key
- **THEN** system calls `POST https://api.anthropic.com/v1/messages` with proper headers (`x-api-key`, `anthropic-version`) and returns the text response

#### Scenario: Missing API key
- **WHEN** `api-anthropic` is selected but `anthropic_api_key` is not configured
- **THEN** system returns an error indicating ANTHROPIC_API_KEY is required

### Requirement: api-gemini backend
The system SHALL support Google Gemini generateContent API as an LLM backend (`api-gemini`).

#### Scenario: Successful API call
- **WHEN** `api-gemini` backend sends a message with valid API key
- **THEN** system calls `POST https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent` and returns the text response

#### Scenario: Missing API key
- **WHEN** `api-gemini` is selected but `gemini_api_key` is not configured
- **THEN** system returns an error indicating GEMINI_API_KEY is required

### Requirement: cli-claude backend
The system SHALL support Claude CLI (`claude -p`) as an LLM backend (`cli-claude`).

#### Scenario: CLI not installed
- **WHEN** `cli-claude` is selected but `claude` command is not found in PATH
- **THEN** system returns an error indicating claude CLI is not installed

#### Scenario: Successful CLI call
- **WHEN** `cli-claude` backend sends a message
- **THEN** system executes `claude -p "<prompt>"` (with optional `--model` flag) and returns stdout

### Requirement: cli-gemini backend
The system SHALL support Gemini CLI (`gemini -p`) as an LLM backend (`cli-gemini`).

#### Scenario: CLI not installed
- **WHEN** `cli-gemini` is selected but `gemini` command is not found in PATH
- **THEN** system returns an error indicating gemini CLI is not installed

#### Scenario: Successful CLI call
- **WHEN** `cli-gemini` backend sends a message
- **THEN** system executes `gemini -p "<prompt>"` (with optional `--model` flag) and returns stdout

### Requirement: Model override
The system SHALL allow overriding the default model for each backend via `fetch_model` / `eval_model` config.

#### Scenario: Custom model specified
- **WHEN** `eval_model = "claude-opus-4-5-20250514"` is set in config
- **THEN** system uses the specified model instead of the default for eval
