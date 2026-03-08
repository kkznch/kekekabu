## 1. CLI Structure

- [x] 1.1 Add `Workflow` subcommand to `main.rs` with `Run` variant and `--skip` option (values: discover, scan, fetch)
- [x] 1.2 Add match arm in main — route to `cmd::workflow::run()`, passing conn, config, and skip list
- [x] 1.3 Register `mod workflow` in `src/cmd/mod.rs`

## 2. Data Structures

- [x] 2.1 Define `StepStatus` enum (Success, Skipped, Failed(String)) in `src/cmd/workflow.rs`
- [x] 2.2 Define `StockWorkflowStatus` struct with ticker, name, and per-step status
- [x] 2.3 Define `WorkflowReport` struct with discover result, stock statuses, and error list; implement Serialize

## 3. Pipeline Orchestration

- [x] 3.1 Implement discover step — call `cmd::discover::run()`, catch errors, populate report
- [x] 3.2 Implement scan step — iterate watchlist stocks, call JQuantsClient per-stock with error isolation, save prices to DB
- [x] 3.3 Implement fetch step — iterate scan-succeeded stocks, call LLM backend per-stock with error isolation, save fetch results
- [x] 3.4 Implement eval step — iterate fetch-succeeded stocks, build prompt + call LLM per-stock with error isolation, save evaluations
- [x] 3.5 Wire all steps together in `run()` function with `--skip` support

## 4. Launchd Integration

- [x] 4.1 Update plist template in `service.rs` to use `{bin} workflow run` instead of `/bin/sh -c` chain
- [x] 4.2 Update plist generation tests to verify new ProgramArguments format

## 5. Integration

- [x] 5.1 Add unit test for WorkflowReport serialization
- [x] 5.2 Run full test suite to verify no regressions
- [x] 5.3 Update CLAUDE.md with `kabu workflow run` command
