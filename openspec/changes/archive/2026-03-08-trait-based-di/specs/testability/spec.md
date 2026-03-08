## ADDED Requirements

### Requirement: StockApi trait abstraction
The system SHALL define a `StockApi` trait that abstracts J-Quants API operations, and `JQuantsClient` SHALL implement this trait. Command modules SHALL accept `&dyn StockApi` instead of constructing clients internally.

#### Scenario: Scan uses injected StockApi
- **WHEN** `cmd::scan::run()` is called with a `&dyn StockApi` implementation
- **THEN** the function SHALL use the injected implementation for all stock API calls instead of constructing its own `JQuantsClient`

#### Scenario: Mock StockApi in tests
- **WHEN** a test creates a `MockStockApi` with predefined responses
- **THEN** `cmd::scan::run()` SHALL work with the mock without making real HTTP requests

### Requirement: ServiceRuntime trait abstraction
The system SHALL define a `ServiceRuntime` trait that abstracts filesystem and process operations in the service module. Each service subcommand function SHALL accept `&dyn ServiceRuntime`.

#### Scenario: Install uses injected runtime
- **WHEN** `cmd::service::install()` is called with a `&dyn ServiceRuntime`
- **THEN** the function SHALL use the injected runtime for file writes, directory creation, and binary path resolution

#### Scenario: Mock ServiceRuntime in tests
- **WHEN** a test creates a `MockRuntime` that records operations without performing real I/O
- **THEN** service functions SHALL work with the mock, and tests can verify which operations were requested

### Requirement: Real implementations preserve existing behavior
The `JQuantsClient` implementation of `StockApi` and the `RealRuntime` implementation of `ServiceRuntime` SHALL preserve all existing behavior exactly. No functional changes to production code paths.

#### Scenario: JQuantsClient still works through trait
- **WHEN** `main.rs` constructs `JQuantsClient` and passes it as `&dyn StockApi`
- **THEN** all existing scan/discover functionality SHALL work identically to before

#### Scenario: RealRuntime still works through trait
- **WHEN** `main.rs` constructs `RealRuntime` and passes it as `&dyn ServiceRuntime`
- **THEN** all existing service install/uninstall/start/stop/status SHALL work identically to before

### Requirement: Integration tests with mocks
The system SHALL include integration tests that exercise command logic using mock implementations, verifying behavior without real external I/O.

#### Scenario: Scan integration test with mock API
- **WHEN** an integration test calls `cmd::scan::run()` with a `MockStockApi` returning predefined quotes and an in-memory SQLite database
- **THEN** the test SHALL verify that prices are saved to DB and indicators are calculated correctly

#### Scenario: Service install test with mock runtime
- **WHEN** a unit test calls `cmd::service::install()` with a `MockRuntime`
- **THEN** the test SHALL verify that `generate_plist()` output was passed to the runtime's write method with the correct path
