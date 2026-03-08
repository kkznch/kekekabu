## Context

The 4-step pipeline (discover → scan → fetch → eval) is currently run as separate processes chained with `&&`. Each process opens its own DB connection, loads config, and any failure aborts the entire chain. Per-stock errors in scan/fetch/eval propagate up with `?` and kill the whole command.

A `workflow` subcommand runs the pipeline as a single process. Each step reports results per-stock, and failures are isolated — a stock that fails in `scan` is excluded from subsequent steps, but all other stocks continue.

## Goals / Non-Goals

**Goals:**
- Single-process pipeline with shared DB connection and config
- Per-stock error isolation (failed stocks are excluded from later steps, not the whole pipeline)
- Summary report showing per-stock success/failure across steps
- `--skip` flag to skip steps (e.g., skip discover to start from scan)
- Update launchd plist to use `workflow run` instead of `&&` chain

**Non-Goals:**
- Modifying existing `cmd::scan::run()` / `cmd::fetch::run()` / `cmd::eval::run()` signatures (workflow calls them but doesn't change them)
- Wait... actually we CAN'T just call existing `run()` functions as-is because they bail on first error. We need to either refactor them or write the per-stock loop in workflow.rs.

**Revised approach:** The workflow module handles its own per-stock loop rather than calling the existing `run()` functions. It reuses the internal helpers (API calls, prompt building, parsing) but wraps each stock in error handling. This avoids changing existing command signatures.

## Decisions

### 1. Per-stock loop in workflow.rs

**Choice:** workflow.rs implements its own iteration over stocks, calling lower-level functions (JQuantsClient, LLM backend, DB operations) directly, with per-stock try-catch.

**Alternatives considered:**
- Modify existing `run()` functions to return partial results — changes their API contract, affects non-workflow usage
- Add `run_single_stock()` variants — duplicates code

**Rationale:** Workflow is the orchestrator. It handles the loop and error isolation. Existing commands remain unchanged for standalone use.

### 2. WorkflowReport structure

```rust
struct WorkflowReport {
    discover: Option<DiscoverResult>,
    stocks: Vec<StockWorkflowStatus>,
    errors: Vec<WorkflowError>,
}

struct StockWorkflowStatus {
    ticker: String,
    name: String,
    scan: StepStatus,     // Success / Skipped / Failed(reason)
    fetch: StepStatus,
    eval: StepStatus,
}

enum StepStatus {
    Success,
    Skipped,
    Failed(String),
}
```

### 3. --skip flag

**Choice:** `--skip discover` skips the discover step. Other valid values: `scan`, `fetch`. No point skipping `eval` (it's the last step).

**Rationale:** Common use case: daily run skips discover (only needed weekly). Matches existing cron pattern.

### 4. Launchd plist update

**Choice:** Change plist ProgramArguments from `/bin/sh -c "kabu discover && kabu scan ..."` to `kabu workflow run`.

**Rationale:** Eliminates shell dependency, gets per-stock error isolation automatically.

## Risks / Trade-offs

- **[Code duplication]** → Workflow duplicates some loop logic from scan/fetch/eval. Mitigation: kept minimal, workflow focuses on orchestration not business logic.
- **[Different behavior]** → `kabu scan` standalone vs `kabu workflow run`'s scan step may behave slightly differently. Mitigation: document that workflow provides error isolation while standalone commands fail-fast.
- **[Dependency on internal APIs]** → Workflow calls JQuantsClient, LLM backend directly. These are stable internal APIs.
