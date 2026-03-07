## ADDED Requirements

### Requirement: Config file at ~/.config/kabu/config.toml
The system SHALL load configuration from `~/.config/kabu/config.toml` with sections: [api], [llm], [spec], [output].

#### Scenario: Config loaded successfully
- **WHEN** config.toml exists with valid TOML
- **THEN** system loads API keys, LLM backend settings, spec path, and output format

#### Scenario: Config file missing
- **WHEN** config.toml does not exist
- **THEN** system uses default values (fetch=cli-gemini, eval=cli-claude, spec=specs/template.yaml, format=json)

### Requirement: Environment variable overrides
The system SHALL allow overriding config values with environment variables: JQUANTS_API_KEY, ANTHROPIC_API_KEY, GEMINI_API_KEY.

#### Scenario: Env var takes precedence
- **WHEN** both config.toml and env var define jquants_api_key
- **THEN** system uses the env var value (env overrides config)

#### Scenario: Empty env var is ignored
- **WHEN** env var is set to empty string
- **THEN** system ignores it and uses config.toml value

### Requirement: Init command generates config and spec template
The system SHALL generate config.toml and specs/template.yaml when `kabu init` is run.

#### Scenario: First-time init
- **WHEN** user runs `kabu init` with no existing config
- **THEN** system creates config.toml with commented API key placeholders and specs/template.yaml

#### Scenario: Config already exists
- **WHEN** user runs `kabu init` with existing config.toml
- **THEN** system returns an error suggesting `--force` to overwrite

#### Scenario: Force overwrite
- **WHEN** user runs `kabu init --force`
- **THEN** system overwrites config.toml and regenerates specs/template.yaml

### Requirement: Spec template is always overwritten
The system SHALL always overwrite `specs/template.yaml` on init, as it is a reference template. User custom strategies use separate files.

#### Scenario: Template regenerated
- **WHEN** user runs `kabu init` (or `--force`)
- **THEN** system writes the latest template.yaml regardless of whether it existed
