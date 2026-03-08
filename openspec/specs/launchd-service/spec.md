## ADDED Requirements

### Requirement: Install launchd service
The system SHALL generate a launchd plist file and place it in `~/Library/LaunchAgents/` when the user runs `kabu service install`.

#### Scenario: Successful install
- **WHEN** user runs `kabu service install`
- **THEN** the system generates a plist file at `~/Library/LaunchAgents/com.kekekabu.pipeline.plist` with the current binary's absolute path and pipeline schedule, and prints the installed path to stdout

#### Scenario: Already installed
- **WHEN** user runs `kabu service install` and the plist file already exists
- **THEN** the system SHALL overwrite the existing plist and print a message indicating it was updated

#### Scenario: Non-macOS platform
- **WHEN** user runs `kabu service install` on a non-macOS platform
- **THEN** the system SHALL exit with an error message indicating launchd is macOS-only

### Requirement: Uninstall launchd service
The system SHALL remove the launchd plist file and stop the service when the user runs `kabu service uninstall`.

#### Scenario: Successful uninstall
- **WHEN** user runs `kabu service uninstall` and the plist exists
- **THEN** the system SHALL run `launchctl bootout` to stop the service and delete the plist file

#### Scenario: Not installed
- **WHEN** user runs `kabu service uninstall` and no plist exists
- **THEN** the system SHALL print a message indicating no service is installed

### Requirement: Start launchd service
The system SHALL load and start the launchd service when the user runs `kabu service start`.

#### Scenario: Successful start
- **WHEN** user runs `kabu service start` and the plist is installed
- **THEN** the system SHALL run `launchctl bootstrap` to load the service

#### Scenario: Not installed
- **WHEN** user runs `kabu service start` and no plist exists
- **THEN** the system SHALL exit with an error suggesting `kabu service install` first

### Requirement: Stop launchd service
The system SHALL stop the launchd service when the user runs `kabu service stop`.

#### Scenario: Successful stop
- **WHEN** user runs `kabu service stop` and the service is loaded
- **THEN** the system SHALL run `launchctl bootout` to unload the service

#### Scenario: Not running
- **WHEN** user runs `kabu service stop` and the service is not loaded
- **THEN** the system SHALL print a message indicating the service is not running

### Requirement: Show service status
The system SHALL display the current launchd service status when the user runs `kabu service status`.

#### Scenario: Service installed and running
- **WHEN** user runs `kabu service status` and the service is loaded
- **THEN** the system SHALL display the service label, plist path, and running state

#### Scenario: Service not installed
- **WHEN** user runs `kabu service status` and no plist exists
- **THEN** the system SHALL display "Not installed"

### Requirement: Plist uses absolute paths
The generated plist SHALL use the absolute path to the `kabu` binary, determined at install time, and SHALL NOT depend on `PATH` environment variable.

#### Scenario: Binary path resolution
- **WHEN** the system generates a plist
- **THEN** the `ProgramArguments` SHALL contain the absolute path from `std::env::current_exe()`

### Requirement: Pipeline schedule configuration
The generated plist SHALL configure a daily pipeline schedule using `StartCalendarInterval`.

#### Scenario: Default schedule
- **WHEN** user runs `kabu service install` without schedule options
- **THEN** the plist SHALL configure the pipeline to run at 08:00 daily
