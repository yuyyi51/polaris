## ADDED Requirements

### Requirement: Polaris data root selection
The system SHALL select one Polaris data root for every store-backed ordinary CLI command, including `init`, `status`, `remember`, `forget`, `rename`, `lifecycle`, `diff`, `replace`, `merge`, `prune`, `compact`, `list`, `note`, `recall`, `touch`, `cite`, and `clear`.

#### Scenario: Use the current workspace root by default
- **WHEN** a store-backed ordinary command runs without `POLARIS_ROOT`
- **THEN** the command uses `<process-cwd>/.polaris` as its data root
- **AND** `polaris status --json` reports that absolute path as `root`

#### Scenario: Use an absolute environment root exactly
- **WHEN** a store-backed ordinary command runs with `POLARIS_ROOT` set to an absolute path
- **THEN** the command uses that path as the exact data root
- **AND** the command MUST NOT append another `.polaris` path component
- **AND** `polaris status --json` reports that path as `root`
- **AND** no files are written under `<process-cwd>/.polaris`

#### Scenario: Reject an empty environment root
- **WHEN** a store-backed command attempts to select a data root with `POLARIS_ROOT` set to an empty value
- **THEN** the command exits with an error that identifies `POLARIS_ROOT` as invalid
- **AND** the command does not initialize or mutate a Polaris store

#### Scenario: Reject a relative environment root
- **WHEN** a store-backed command attempts to select a data root with `POLARIS_ROOT` set to a non-empty relative path
- **THEN** the command exits with an error explaining that `POLARIS_ROOT` must be an absolute path
- **AND** the command does not initialize or mutate a Polaris store

#### Scenario: Keep selected stores isolated
- **WHEN** both `<process-cwd>/.polaris` and the root selected by `POLARIS_ROOT` contain initialized Polaris data
- **THEN** the command reads or writes only the root selected by `POLARIS_ROOT`
- **AND** the system does not merge, migrate, or fall back to the cwd-based store

#### Scenario: Allow init to create a fresh overridden root
- **WHEN** `polaris init` runs with `POLARIS_ROOT` selecting an absolute path that does not yet exist
- **THEN** the system creates the directory and `state.json`, `memories.jsonl`, and `docs/` directly under the selected root
- **AND** `polaris status --json` reports `initialized: true` with that absolute path as `root`

#### Scenario: Preserve a pre-existing overridden root
- **WHEN** `POLARIS_ROOT` is set to an absolute path that already contains a Polaris data root
- **AND** `polaris init` runs
- **THEN** the system exits successfully without overwriting existing records
- **AND** `polaris recall` returns the previously stored memories

### Requirement: Hook data root selection
The system SHALL apply the same Polaris data root selection to `polaris hook session-start`, `polaris hook post-compact`, and `polaris hook post-tool-use` whenever those commands access storage.

#### Scenario: Environment root overrides hook cwd
- **WHEN** a matching hook invocation includes a `cwd` and runs with a non-empty absolute `POLARIS_ROOT`
- **THEN** the hook uses the root selected by `POLARIS_ROOT`
- **AND** it does not read or write `<hook-cwd>/.polaris`

#### Scenario: Hook uses payload cwd by default
- **WHEN** a matching hook invocation includes a `cwd` and runs without `POLARIS_ROOT`
- **THEN** the hook uses `<hook-cwd>/.polaris`

#### Scenario: Hook falls back to process cwd
- **WHEN** a matching hook invocation omits `cwd` and runs without `POLARIS_ROOT`
- **THEN** the hook uses `<process-cwd>/.polaris`

#### Scenario: Fallback hooks share overridden pending state
- **WHEN** matching `post-compact` and `post-tool-use` invocations run with the same `POLARIS_ROOT`
- **THEN** both commands read or write pending compact-recall state under that selected root
- **AND** recalled memory is read from that selected root

### Requirement: Root-contained Polaris artifacts
The system SHALL keep every newly created store artifact under the selected Polaris data root, and shall emit and store absolute paths for note files, replacement drafts, and merge drafts.

#### Scenario: Initialize an overridden root
- **WHEN** `polaris init` runs with `POLARIS_ROOT` selecting an uninitialized directory
- **THEN** the system creates `state.json`, `memories.jsonl`, and `docs/` directly under the selected root

#### Scenario: Create notes under an overridden root with absolute path
- **WHEN** `polaris note create` runs with `POLARIS_ROOT`
- **THEN** the note file is created under `<selected-root>/docs/`
- **AND** the printed note path is absolute
- **AND** the recorded note `path` field is absolute and points to the created file

#### Scenario: Create notes in default mode with absolute path
- **WHEN** `polaris note create` runs without `POLARIS_ROOT`
- **THEN** the note file is created under `<process-cwd>/.polaris/docs/`
- **AND** the printed note path is absolute
- **AND** the recorded note `path` field is absolute and points to the created file

#### Scenario: Create replacement drafts with absolute path
- **WHEN** `polaris remember --key goal --replace --dry-run --text "new goal"` runs in an initialized workspace
- **THEN** the draft is created under the selected root's `maintenance/` directory
- **AND** the printed draft path is absolute
- **AND** `polaris replace apply <that-absolute-path> --yes` accepts the path and applies the draft

#### Scenario: Create merge drafts with absolute path
- **WHEN** `polaris merge --into goal.summary source.a source.b` runs in an initialized workspace
- **THEN** the draft is created under the selected root's `maintenance/` directory
- **AND** the printed draft path is absolute
- **AND** `polaris merge apply <that-absolute-path> --yes` accepts the path and applies the draft

#### Scenario: Persist auxiliary data under an overridden root
- **WHEN** a command writes replacement history, citations, memory locks, or hook state with `POLARIS_ROOT`
- **THEN** each file is written under the selected root
- **AND** no corresponding auxiliary file is written under `<process-cwd>/.polaris`

## MODIFIED Requirements

### Requirement: Hook recall prompt configuration
The system SHALL allow Codex hook recall prompts to be configured by TOML files while preserving the built-in prompt when no configuration exists.

#### Scenario: Use selected-root hook prompt before user hook prompt
- **WHEN** `<selected-root>/config.toml` and `~/.polaris/config.toml` both define `[hooks].recall_prompt`
- **THEN** hook output uses the selected-root `recall_prompt`

#### Scenario: Use user hook prompt when selected-root config is absent
- **WHEN** `<selected-root>/config.toml` is absent and `~/.polaris/config.toml` defines `[hooks].recall_prompt`
- **THEN** hook output uses the user `recall_prompt`

#### Scenario: Use built-in prompt when no hook config exists
- **WHEN** neither `<selected-root>/config.toml` nor `~/.polaris/config.toml` exists
- **THEN** hook output uses the built-in recall instruction
- **AND** the output MUST NOT contain stored memory contents

#### Scenario: Inline recall with placeholder
- **WHEN** the selected `recall_prompt` contains `{{recall}}`
- **THEN** hook output replaces each `{{recall}}` placeholder with the current `polaris recall` output

#### Scenario: Configured prompt without placeholder does not inline recall
- **WHEN** the selected `recall_prompt` does not contain `{{recall}}`
- **THEN** hook output uses the configured prompt text
- **AND** the output MUST NOT contain stored memory contents unless they are present in the configured prompt itself

### Requirement: Maintenance prompt configuration
The system SHALL allow users to configure separate agent review prompt templates for prune and compact maintenance suggestions.

#### Scenario: Use selected-root maintenance prompts
- **WHEN** `<selected-root>/config.toml` defines `[maintenance.prompts].prune` and `[maintenance.prompts].compact`
- **THEN** `polaris prune --suggest` renders the configured prune prompt
- **AND** `polaris compact --suggest` renders the configured compact prompt

#### Scenario: Use user maintenance prompts when selected-root config is absent
- **WHEN** `<selected-root>/config.toml` is absent and `~/.polaris/config.toml` defines maintenance prompt templates
- **THEN** prune and compact suggestion commands use the user configured templates

#### Scenario: Fall back to built-in maintenance prompts
- **WHEN** no selected config file defines a maintenance prompt for the suggestion command
- **THEN** the command renders a built-in conservative agent review prompt for that command

#### Scenario: Render supported maintenance prompt placeholders
- **WHEN** a maintenance prompt template contains `{{suggestions}}`, `{{status}}`, or `{{keys}}`
- **THEN** the system replaces those placeholders with the current suggestion data, workspace status data, and known keyed inline memory keys
- **AND** the rendered prompt does not inline full memory recall content
