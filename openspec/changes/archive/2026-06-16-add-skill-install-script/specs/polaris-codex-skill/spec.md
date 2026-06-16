## ADDED Requirements

### Requirement: Script-based skill installation
The repository SHALL provide a script that installs the bundled Polaris Codex skill into a Codex skills directory.

#### Scenario: Install into Codex home from environment
- **WHEN** a user runs the skill install script without a skills directory override
- **AND** `CODEX_HOME` is set
- **THEN** the script installs `skills/codex/polaris/` into `$CODEX_HOME/skills/polaris`

#### Scenario: Install into existing Codex app home
- **WHEN** a user runs the skill install script without a skills directory override
- **AND** `CODEX_HOME` is unset
- **AND** `$HOME/.codex-app` exists
- **THEN** the script installs `skills/codex/polaris/` into `$HOME/.codex-app/skills/polaris`

#### Scenario: Install into classic Codex home fallback
- **WHEN** a user runs the skill install script without a skills directory override
- **AND** `CODEX_HOME` is unset
- **AND** `$HOME/.codex-app` does not exist
- **THEN** the script installs `skills/codex/polaris/` into `$HOME/.codex/skills/polaris`

#### Scenario: Install into an overridden skills directory
- **WHEN** a user runs the skill install script with an explicit skills directory override
- **THEN** the script installs `skills/codex/polaris/` into `<override>/polaris`

#### Scenario: Reinstall bundled skill
- **WHEN** the destination `polaris` skill directory already exists
- **THEN** the script replaces that destination directory with the repository's bundled skill directory

#### Scenario: Preserve script scope
- **WHEN** a user runs the skill install script
- **THEN** the script MUST NOT modify Codex hook configuration
- **AND** the script MUST NOT initialize Polaris workspace memory
