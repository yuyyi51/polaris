## ADDED Requirements

### Requirement: Maintenance prompt configuration
The system SHALL allow users to configure separate agent review prompt templates for prune and compact maintenance suggestions.

#### Scenario: Use workspace maintenance prompts
- **WHEN** `.polaris/config.toml` defines `[maintenance.prompts].prune` and `[maintenance.prompts].compact`
- **THEN** `polaris prune --suggest` renders the configured prune prompt
- **AND** `polaris compact --suggest` renders the configured compact prompt

#### Scenario: Use user maintenance prompts when workspace config is absent
- **WHEN** `.polaris/config.toml` is absent and `~/.polaris/config.toml` defines maintenance prompt templates
- **THEN** prune and compact suggestion commands use the user configured templates

#### Scenario: Fall back to built-in maintenance prompts
- **WHEN** no selected config file defines a maintenance prompt for the suggestion command
- **THEN** the command renders a built-in conservative agent review prompt for that command

#### Scenario: Render supported maintenance prompt placeholders
- **WHEN** a maintenance prompt template contains `{{suggestions}}`, `{{status}}`, or `{{keys}}`
- **THEN** the system replaces those placeholders with the current suggestion data, workspace status data, and known keyed inline memory keys
- **AND** the rendered prompt does not inline full memory recall content

## MODIFIED Requirements

### Requirement: Memory prune suggestions
The system SHALL suggest stale or redundant memory cleanup candidates with agent review prompts and without mutating memory.

#### Scenario: Suggest prune candidates
- **WHEN** `polaris prune --suggest` runs in an initialized workspace with old `state` or `log` memory records
- **THEN** the command exits successfully and prints cleanup suggestions with reasons
- **AND** the command prints an agent prompt for reviewing prune candidates
- **AND** active memory records remain unchanged

#### Scenario: Suggest prune candidates as JSON
- **WHEN** `polaris prune --suggest --json` runs in an initialized workspace
- **THEN** the command exits successfully and prints a JSON object containing `suggestions`, `prompt`, and `prompt_context`
- **AND** `suggestions` includes machine-readable candidate keys, reasons, and suggested commands
- **AND** `prompt_context` includes workspace status data and known keyed inline memory keys

#### Scenario: Report no prune suggestions
- **WHEN** `polaris prune --suggest` runs in an initialized workspace with no cleanup candidates
- **THEN** the command exits successfully and reports that no prune suggestions are available
- **AND** the command still prints an agent prompt for reviewing the empty result

### Requirement: Memory compact suggestions
The system SHALL suggest memory consolidation candidates with agent review prompts and without mutating memory.

#### Scenario: Suggest compact candidates
- **WHEN** `polaris compact --suggest` runs in an initialized workspace with multiple related keyed inline memories
- **THEN** the command exits successfully and prints consolidation suggestions with reasons
- **AND** the command prints an agent prompt for reviewing compact candidates
- **AND** active memory records remain unchanged

#### Scenario: Suggest compact candidates as JSON
- **WHEN** `polaris compact --suggest --json` runs in an initialized workspace
- **THEN** the command exits successfully and prints a JSON object containing `suggestions`, `prompt`, and `prompt_context`
- **AND** `suggestions` includes machine-readable source keys, proposed target keys, reasons, and suggested commands
- **AND** `prompt_context` includes workspace status data and known keyed inline memory keys

#### Scenario: Report no compact suggestions
- **WHEN** `polaris compact --suggest` runs in an initialized workspace with no consolidation candidates
- **THEN** the command exits successfully and reports that no compact suggestions are available
- **AND** the command still prints an agent prompt for reviewing the empty result
