## 1. LlmBackend Trait Extension

- [x] 1.1 Add `send_message_with_schema()` method to `LlmBackend` trait in `src/llm/mod.rs` with default implementation that falls back to `send_message()`

## 2. Anthropic API tool_use Implementation

- [x] 2.1 Add `Tool`, `InputSchema`, `ToolChoice` structs to `api_anthropic.rs` for request serialization
- [x] 2.2 Add `tools` and `tool_choice` fields to `MessageRequest`
- [x] 2.3 Extend `ContentBlock` deserialization to handle `tool_use` type with `input` field (serde tagged enum)
- [x] 2.4 Implement `send_message_with_schema()` override for `ApiAnthropicBackend` — builds request with tools, extracts `input` from tool_use response block

## 3. JSON Schema Definitions

- [x] 3.1 Define eval response JSON schema in `cmd/eval.rs` as `serde_json::Value` constant (matching `EvalResponse` struct)
- [x] 3.2 Define discover response JSON schema in `cmd/discover.rs` as `serde_json::Value` constant (matching `DiscoverResponse` struct)

## 4. Caller Integration

- [x] 4.1 Update `cmd/eval.rs` to call `send_message_with_schema()` with eval schema instead of `send_message()`
- [x] 4.2 Update `cmd/discover.rs` to call `send_message_with_schema()` with discover schema instead of `send_message()`

## 5. Tests

- [x] 5.1 Add unit test for `ContentBlock` deserialization of `tool_use` type
- [x] 5.2 Add unit test verifying default `send_message_with_schema()` falls back to `send_message()`
- [x] 5.3 Run full test suite to verify no regressions
