## Context

Currently, only `LlmBackend` has a trait abstraction. All other external boundaries — J-Quants API, filesystem, process execution — are called directly in command modules, making them untestable without real I/O.

The existing `LlmBackend` pattern (trait + factory + `Box<dyn Trait>`) is well-established and works. We extend this same pattern to the remaining external boundaries.

## Goals / Non-Goals

**Goals:**
- Trait abstraction for J-Quants API client (`StockApi`)
- Trait abstraction for service.rs I/O operations (`ServiceRuntime`)
- Mock implementations for testing
- Integration tests that exercise command logic without real APIs/filesystem
- Follow the same `async_trait` + `Box<dyn Trait>` pattern as `LlmBackend`

**Non-Goals:**
- DB trait abstraction (in-memory SQLite already provides testability)
- LLM mock (already trait-based; adding a mock is useful but out of scope here)
- DI container or framework (overkill for this codebase size)
- Changing public CLI behavior

## Decisions

### 1. StockApi trait in jquants.rs

**Choice:** Define `StockApi` trait alongside `JQuantsClient` in `src/jquants.rs`. `JQuantsClient` implements `StockApi`.

**Alternatives considered:**
- Separate `src/stock_api.rs` module — adds a file for just a trait definition, unnecessary indirection
- Generic type parameters `<T: StockApi>` — more complex than `&dyn StockApi`, gains little

**Rationale:** Co-locating trait and primary implementation is the simplest approach. `&dyn StockApi` matches the `Box<dyn LlmBackend>` pattern already in use.

### 2. ServiceRuntime trait in service.rs

**Choice:** Define `ServiceRuntime` trait within `src/cmd/service.rs` covering filesystem writes, file existence checks, and process execution (launchctl). Each public function takes `&dyn ServiceRuntime`.

**Alternatives considered:**
- Separate traits for filesystem and process execution — over-splitting; service.rs is the only consumer
- `--dry-run` flag without trait — simpler but doesn't enable testing

**Rationale:** A single trait keeps it simple. The `RealRuntime` struct calls real `std::fs` and `Command`. Tests use `MockRuntime` or `DryRunRuntime`.

### 3. Command function signatures

**Choice:** Change `cmd::scan::run()` to accept `&dyn StockApi` as parameter. Main constructs `JQuantsClient` and passes it. Same for `cmd::service::*` functions taking `&dyn ServiceRuntime`.

**Current:**
```rust
pub async fn run(conn: &Connection, config: &AppConfig, days: u32, refresh_master: bool) -> Result<Vec<ScanResult>>
```

**New:**
```rust
pub async fn run(conn: &Connection, config: &AppConfig, api: &dyn StockApi, days: u32, refresh_master: bool) -> Result<Vec<ScanResult>>
```

**Rationale:** Matches how `conn` and `config` are already passed — simple parameter injection, no frameworks.

### 4. Mock location

**Choice:** Mock implementations live in `#[cfg(test)] mod tests` within their respective files, plus test helper modules under `tests/`.

**Rationale:** Keeps mocks close to the code they mock. Test helper for shared mocks (e.g., `MockStockApi`) can be a `tests/helpers/` module.

## Risks / Trade-offs

- **[API surface change]** → Command function signatures change. Only `main.rs` calls them, so impact is minimal.
- **[Trait object overhead]** → Dynamic dispatch via `&dyn`. Negligible for CLI tool making HTTP calls.
- **[Maintenance cost]** → Two traits to maintain. Both are narrow (2-3 methods each), so cost is low.
- **[Mock fidelity]** → Mocks may not catch real API behavior changes. Mitigation: keep real `JQuantsClient` tests as manual integration tests.
