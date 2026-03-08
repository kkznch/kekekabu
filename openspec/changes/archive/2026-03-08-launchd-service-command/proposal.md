## Why

Users need to set up `kabu` as a periodic job on macOS (daily scan, weekly discover, etc.). Currently they must hand-write launchd plist XML files, which is error-prone and tedious. A `kabu service` subcommand would generate, install, and manage launchd plist files automatically — matching the pattern used by yabai and Tailscale.

## What Changes

- Add `kabu service install` — generates plist from the current binary path and places it in `~/Library/LaunchAgents/`
- Add `kabu service uninstall` — removes the plist
- Add `kabu service start` / `stop` / `status` — wraps `launchctl` commands
- Plist uses absolute paths (no PATH dependency), embeds config for morning pipeline + evening report schedules
- No new external dependencies required

## Capabilities

### New Capabilities
- `launchd-service`: macOS launchd plist generation, installation, and lifecycle management via CLI subcommands

### Modified Capabilities
<!-- No existing spec requirements change — this is a new standalone capability -->

## Impact

- `src/main.rs` — new `Service` subcommand added to CLI
- `src/cmd/service.rs` — new module implementing plist generation and launchctl operations
- No database changes
- No config file changes (reads existing binary path at install time)
- macOS only — the subcommand should error gracefully on non-macOS platforms
