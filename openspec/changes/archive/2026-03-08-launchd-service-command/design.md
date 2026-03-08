## Context

Users currently need to hand-write launchd plist XML to run `kabu` on a schedule. This is error-prone (XML syntax, absolute paths, environment variables). The yabai and Tailscale projects demonstrate that CLI tools can manage their own launchd integration with simple subcommands.

The `kabu` pipeline has a natural daily schedule: morning discovery/scan/fetch/eval, market-hours execution, evening reporting. A `kabu service` subcommand automates this setup.

## Goals / Non-Goals

**Goals:**
- One-command install: `kabu service install` creates and places the plist
- Lifecycle management: start / stop / status / uninstall
- Use modern `launchctl` API (`bootstrap`/`bootout` over legacy `load`/`unload`)
- Absolute binary paths — no dependency on `PATH`

**Non-Goals:**
- Linux systemd support (future work)
- Custom schedule configuration via CLI flags (keep it simple; users can edit the plist)
- Running as a system daemon (LaunchDaemons) — user agent only

## Decisions

### 1. Plist generation: string template with format!()
**Choice:** Embed plist XML as a Rust `const &str` template with `format!()` placeholders for the binary path.

**Alternatives considered:**
- `launchd` crate — small community crate, adds dependency for minimal benefit
- Programmatic XML builder — over-engineered for a fixed-structure plist

**Rationale:** This is the pattern used by Tailscale (hardcoded string) and yabai (sprintf template). Simple, zero dependencies, easy to audit.

### 2. Binary path resolution: std::env::current_exe()
**Choice:** Use `std::env::current_exe()` at install time to capture the absolute path.

**Rationale:** Eliminates PATH dependency entirely (Tailscale approach). If the binary moves, user re-runs `kabu service install`.

### 3. Service label: com.kekekabu.pipeline
**Choice:** Single plist for the entire daily pipeline, not per-command plists.

**Alternatives considered:**
- Separate plists per command (scan, eval, etc.) — too many files, harder to manage
- Wrapper shell script — adds a file, harder to maintain

**Rationale:** The pipeline is a single logical unit. The plist runs a shell command chaining the pipeline steps.

### 4. launchctl API: modern bootstrap/bootout
**Choice:** Use `launchctl bootstrap gui/{uid}` and `launchctl bootout gui/{uid}/{label}`.

**Rationale:** `load`/`unload` are legacy. yabai uses the modern API successfully.

### 5. Platform guard: cfg(target_os) + runtime check
**Choice:** Compile the module on all platforms but gate execution with `cfg!(target_os = "macos")` runtime check.

**Alternatives considered:**
- `#[cfg(target_os = "macos")]` compile-time gating — makes cross-compilation harder, hides the subcommand from `--help` on other platforms

**Rationale:** Better UX to show the command exists but explain it's macOS-only, rather than silently hiding it.

## Risks / Trade-offs

- **[Binary path staleness]** → If user moves/rebuilds the binary, the plist points to the old path. Mitigation: `kabu service status` shows the configured path; user re-runs `install`.
- **[Shell command in plist]** → ProgramArguments uses `/bin/sh -c` to chain pipeline commands. Mitigation: the chain is simple and deterministic (`&&` between `kabu` subcommands).
- **[No custom schedules]** → Users who want different times must edit the plist manually. Mitigation: `kabu service status` shows the plist path for easy editing. Future work could add `--hour` flag.
