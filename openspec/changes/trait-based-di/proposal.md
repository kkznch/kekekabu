## Why

External dependencies (J-Quants API, filesystem, process execution) are called directly in command modules, making integration testing impossible without hitting real APIs and performing real I/O. The only DI abstraction is `LlmBackend` trait. Introducing trait abstractions for the remaining external boundaries enables mock-based testing and dry-run support across the codebase.

## What Changes

- Extract `StockApi` trait from `JQuantsClient` — enables mock API in scan/discover tests
- Extract `ServiceRuntime` trait from `service.rs` I/O — enables dry-run and testing of install/uninstall/start/stop/status without real filesystem or launchctl
- Add mock implementations in test code for both traits
- Add integration tests for `cmd::scan::run()` and `cmd::service` using mocks
- Wire DI through command function signatures (pass trait objects instead of concrete types)

## Capabilities

### New Capabilities
- `testability`: Trait abstractions for external I/O boundaries, mock implementations, and integration test infrastructure

### Modified Capabilities

## Impact

- `src/jquants.rs` — extract trait, `JQuantsClient` becomes one implementation
- `src/cmd/scan.rs` — accept `&dyn StockApi` instead of constructing `JQuantsClient` internally
- `src/cmd/discover.rs` — same pattern (uses `get_stock_info` for name lookup)
- `src/cmd/service.rs` — extract filesystem + process operations behind trait
- `src/main.rs` — construct concrete implementations and pass to commands
- `tests/` — new integration test files with mock implementations
- No DB changes (in-memory SQLite already works well for testing)
- No config changes
- No external dependency additions
