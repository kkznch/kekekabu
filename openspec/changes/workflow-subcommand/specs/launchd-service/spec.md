## MODIFIED Requirements

### Requirement: Plist uses absolute paths
The generated plist SHALL use the absolute path to the `kabu` binary, determined at install time, and SHALL NOT depend on `PATH` environment variable. The plist SHALL invoke `kabu workflow run` instead of a shell command chain.

#### Scenario: Binary path resolution
- **WHEN** the system generates a plist
- **THEN** the `ProgramArguments` SHALL contain the absolute path to `kabu` followed by `workflow` and `run`, without using `/bin/sh -c`
