## 1. CLI Subcommand Structure

- [x] 1.1 Add `Service` subcommand enum to `main.rs` with `Install`, `Uninstall`, `Start`, `Stop`, `Status` variants
- [x] 1.2 Add match arm for `Service` in main — route to `cmd::service` module (no DB or config needed)

## 2. Core Module

- [x] 2.1 Create `src/cmd/service.rs` with constants: plist label (`com.kekekabu.pipeline`), plist filename, and plist XML template string
- [x] 2.2 Implement `plist_path()` — returns `~/Library/LaunchAgents/com.kekekabu.pipeline.plist`
- [x] 2.3 Implement `generate_plist()` — resolves binary path via `std::env::current_exe()`, formats the template with absolute path and daily 08:00 schedule
- [x] 2.4 Implement platform guard — check `cfg!(target_os = "macos")` and bail with error on non-macOS

## 3. Subcommand Implementations

- [x] 3.1 Implement `install()` — generate plist, write to LaunchAgents directory, print installed path
- [x] 3.2 Implement `uninstall()` — run `launchctl bootout`, delete plist file, handle not-installed case
- [x] 3.3 Implement `start()` — run `launchctl bootstrap gui/{uid}`, handle not-installed case
- [x] 3.4 Implement `stop()` — run `launchctl bootout gui/{uid}/{label}`, handle not-running case
- [x] 3.5 Implement `status()` — check plist exists, run `launchctl print`, display label/path/running state

## 4. Integration

- [x] 4.1 Register `mod service` in `src/cmd/mod.rs`
- [x] 4.2 Add unit tests for `generate_plist()` — verify XML contains correct binary path and schedule
- [x] 4.3 Update CLAUDE.md commands section with `kabu service` subcommands
- [x] 4.4 Update README.md with launchd setup instructions using `kabu service`
