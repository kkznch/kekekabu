## Why

The pipeline is currently chained via shell `&&` in launchd (`kabu discover && kabu scan && ...`). If any single stock fails in `scan` or `fetch`, the entire pipeline aborts — `eval` never runs for the stocks that succeeded. Each command also opens its own DB connection and loads config separately. A `kabu workflow` subcommand runs the entire pipeline in a single process with per-stock error isolation and a summary report.

## What Changes

- Add `kabu workflow run` subcommand that chains discover → scan → fetch → eval internally
- Per-stock error isolation: if a stock fails in scan, it's excluded from fetch/eval but others continue
- Single DB connection and config load for the entire pipeline
- `WorkflowReport` output showing per-stock status and per-step errors
- `--skip` flag to start from a specific step (e.g. `--skip discover` to start from scan)
- Update launchd plist template to use `kabu workflow run` instead of shell `&&` chaining

## Capabilities

### New Capabilities
- `workflow`: Pipeline orchestration with per-stock error isolation and status reporting

### Modified Capabilities
- `launchd-service`: Update plist template to use `kabu workflow run`

## Impact

- `src/main.rs` — new `Workflow` subcommand
- `src/cmd/workflow.rs` — new module orchestrating discover → scan → fetch → eval
- `src/cmd/service.rs` — update plist template command
- No changes to existing cmd modules (discover/scan/fetch/eval) — workflow calls their `run()` functions
- No DB schema changes
