## 1. StockApi Trait

- [x] 1.1 Define `StockApi` trait in `src/jquants.rs` with `get_all_stock_info()` and `get_daily_quotes()` methods
- [x] 1.2 Implement `StockApi` for `JQuantsClient`
- [x] 1.3 Update `cmd::scan::run()` signature to accept `&dyn StockApi` instead of constructing `JQuantsClient` internally
- [x] 1.4 Update `main.rs` to construct `JQuantsClient` and pass as `&dyn StockApi` to scan

## 2. ServiceRuntime Trait

- [x] 2.1 Define `ServiceRuntime` trait in `src/cmd/service.rs` with methods for: write_file, remove_file, file_exists, create_dir_all, current_exe, run_command
- [x] 2.2 Implement `RealRuntime` struct that delegates to `std::fs` and `std::process::Command`
- [x] 2.3 Refactor service functions (install/uninstall/start/stop/status) to accept `&dyn ServiceRuntime`
- [x] 2.4 Update `main.rs` to construct `RealRuntime` and pass to service commands

## 3. Tests

- [x] 3.1 Add `MockStockApi` in test code and integration test for `cmd::scan::run()` with mock API + in-memory DB
- [x] 3.2 Add `MockRuntime` in service.rs tests and unit tests for install/uninstall verifying correct I/O operations
- [x] 3.3 Run full test suite and ensure all existing tests still pass
