## Context

Anthropic Messages API supports `tool_use` — you define tools with JSON Schema input schemas, and the model returns a `tool_use` content block with structured JSON matching the schema. This is the official way to get structured output from Claude API.

Current flow: prompt → free-form text → regex extract JSON → serde parse
New flow: prompt + tool schema → tool_use content block → serde parse (guaranteed valid JSON)

CLI backends (`claude -p`, `gemini -p`) cannot enforce schemas, so they continue using the text-based flow.

## Goals / Non-Goals

**Goals:**
- Add `send_message_with_schema()` to `LlmBackend` trait with default fallback to `send_message()`
- Implement tool_use in `ApiAnthropicBackend` for structured JSON responses
- Define JSON schemas for eval and discover response types
- Callers transparently get structured output when backend supports it

**Non-Goals:**
- Gemini API structured output (different mechanism, future work)
- Changing CLI backend behavior
- Changing the response data structures themselves (EvalResponse, DiscoverResponse stay the same)
- Adding new tool definitions for fetch command (fetch response is free-form by design)

## Decisions

### 1. Trait extension: default method, not a new trait

**Choice:** Add `send_message_with_schema()` to `LlmBackend` with a default implementation that calls `send_message()` and returns the raw text.

```rust
async fn send_message_with_schema(
    &self,
    prompt: &str,
    max_tokens: u32,
    tool_name: &str,
    tool_description: &str,
    schema: serde_json::Value,
) -> Result<String> {
    // Default: ignore schema, use plain text
    self.send_message(prompt, max_tokens).await
}
```

**Alternatives considered:**
- New `StructuredLlmBackend` trait — requires downcasting or double dispatch, over-engineered
- Generic type parameter with schema — adds complexity for marginal type safety gain

**Rationale:** Default methods mean existing backends (CLI, Gemini API) need zero changes. Only `ApiAnthropicBackend` overrides.

### 2. Schema as serde_json::Value

**Choice:** Pass schema as `serde_json::Value` rather than a generic type or macro-generated schema.

**Rationale:** Simple, no new dependencies. Each caller constructs the schema as a JSON value using `serde_json::json!()`. The schemas are static and small.

### 3. Caller-side parsing unchanged

**Choice:** `send_message_with_schema()` returns `String`. The caller still parses with `serde_json::from_str()`. The difference is the string is now guaranteed to be valid JSON (from tool_use) vs best-effort extracted JSON (from free text).

**Rationale:** Minimal change to calling code. The existing `parse_eval_response()` / `parse_discover_response()` functions continue to work — they just get cleaner input. The `extract_json()` step becomes a no-op when the backend returns structured output directly.

### 4. Tool_use request format

```json
{
  "model": "...",
  "max_tokens": 4096,
  "tools": [{
    "name": "eval_stock",
    "description": "Evaluate a stock...",
    "input_schema": { ... json schema ... }
  }],
  "tool_choice": { "type": "tool", "name": "eval_stock" },
  "messages": [{ "role": "user", "content": "..." }]
}
```

`tool_choice` with `type: "tool"` forces the model to use the specified tool, guaranteeing structured output.

### 5. Response parsing

The response contains a `tool_use` content block:
```json
{
  "content": [
    { "type": "tool_use", "id": "...", "name": "eval_stock", "input": { ... } }
  ]
}
```

Extract `input` field as the structured JSON string.

## Risks / Trade-offs

- **[Schema drift]** → If `EvalResponse` struct changes but schema isn't updated, deserialization fails. Mitigation: schemas are co-located with their response types.
- **[API version dependency]** → tool_use requires `anthropic-version: 2023-06-01` which we already use.
- **[Token usage]** → tool definitions consume input tokens. Mitigation: schemas are small (~200 tokens each).
