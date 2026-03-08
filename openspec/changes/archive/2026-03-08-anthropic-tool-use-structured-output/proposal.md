## Why

The `api-anthropic` backend sends plain text prompts and receives free-form text responses. eval/discover then regex-extract JSON from the response, which is fragile — the LLM sometimes wraps JSON in markdown fences, includes extraneous text, or produces malformed JSON. Anthropic's `tool_use` feature forces the model to return structured JSON matching a predefined schema, eliminating parsing failures and improving response reliability.

## What Changes

- Add `tools` field to `MessageRequest` in `api_anthropic.rs` with JSON schema definitions
- Add `tool_use` content block parsing to `ContentBlock` deserialization
- Extend `LlmBackend` trait with `send_message_with_schema()` method that accepts a JSON schema and returns structured JSON directly
- Default implementation of `send_message_with_schema()` falls back to `send_message()` for CLI backends (schema enforcement not possible)
- Define tool schemas for eval response (`EvalResponse`) and discover response (`DiscoverResponse`)
- Update eval and discover commands to use `send_message_with_schema()` when available

## Capabilities

### New Capabilities

### Modified Capabilities
- `llm-integration`: Adding `send_message_with_schema()` to `LlmBackend` trait and tool_use support to `api-anthropic` backend

## Impact

- `src/llm/mod.rs` — trait gets new method with default implementation
- `src/llm/api_anthropic.rs` — tool_use request/response handling
- `src/cmd/eval.rs` — use structured output when backend supports it
- `src/cmd/discover.rs` — same
- CLI backends unchanged (default fallback to text parsing)
- No DB changes, no config changes
