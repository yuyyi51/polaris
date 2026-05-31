# polaris-recall Specification

## Purpose
TBD - created by archiving change implement-polaris-mvp. Update Purpose after archive.
## Requirements
### Requirement: Workspace memory initialization
The system SHALL initialize Polaris memory only when the user or agent runs `polaris init`.

#### Scenario: Initialize an empty workspace
- **WHEN** `polaris init` runs in a workspace without `.polaris/`
- **THEN** the system creates `.polaris/state.json`, `.polaris/memories.jsonl`, and `.polaris/docs/`

#### Scenario: Reinitialize an existing workspace
- **WHEN** `polaris init` runs in a workspace that already has `.polaris/`
- **THEN** the system preserves existing memories and exits successfully

### Requirement: Workspace memory status
The system SHALL provide machine-readable status for the active workspace with `polaris status --json`.

#### Scenario: Status before initialization
- **WHEN** `polaris status --json` runs in a workspace without `.polaris/`
- **THEN** the system exits successfully and reports `initialized: false`

#### Scenario: Status after records exist
- **WHEN** `polaris status --json` runs in an initialized workspace with stored memories and note files
- **THEN** the system reports `initialized: true`, the Polaris root path, memory count, and document count

### Requirement: Stale memory clearing
The system SHALL allow an agent to clear stale workspace memory only through an explicit command.

#### Scenario: Clear initialized memory
- **WHEN** `polaris clear --yes` runs in an initialized workspace
- **THEN** the system removes stored memory records and note files while keeping `.polaris/` initialized

#### Scenario: Refuse implicit clearing
- **WHEN** `polaris clear` runs without `--yes`
- **THEN** the system refuses to clear memory and leaves existing records unchanged

### Requirement: Short memory recording
The system SHALL allow agents to append short memory entries to the active workspace.

#### Scenario: Record text from an argument
- **WHEN** `polaris remember --text "task goal"` runs in an initialized workspace
- **THEN** the system appends a memory record containing the provided text

#### Scenario: Record text from standard input
- **WHEN** `polaris remember --stdin --title "Decision"` receives text on stdin in an initialized workspace
- **THEN** the system appends a memory record containing the title and stdin text

#### Scenario: Reject recording before initialization
- **WHEN** `polaris remember --text "task goal"` runs in a workspace without `.polaris/`
- **THEN** the system exits with an error explaining that `polaris init` is required

### Requirement: Long note creation
The system SHALL allow agents to create long-form markdown note files under `.polaris/docs/`.

#### Scenario: Create a long note placeholder
- **WHEN** `polaris note create --title "Architecture notes"` runs in an initialized workspace
- **THEN** the system creates a markdown file under `.polaris/docs/` and records a memory entry pointing to that file

#### Scenario: Return the created note location
- **WHEN** a long note is created
- **THEN** the system prints the note id and file path so the agent can write detailed content to it

### Requirement: Context recall
The system SHALL provide saved Polaris memory through `polaris recall`.

#### Scenario: Recall mixed memory
- **WHEN** `polaris recall` runs in an initialized workspace with short memories and long note records
- **THEN** the system prints a model-readable summary containing short memory text, note titles, and note file paths

#### Scenario: Recall empty initialized memory
- **WHEN** `polaris recall` runs in an initialized workspace with no stored memories
- **THEN** the system exits successfully and reports that no Polaris memory is stored

#### Scenario: Reject recall before initialization
- **WHEN** `polaris recall` runs in a workspace without `.polaris/`
- **THEN** the system exits with an error explaining that `polaris init` is required

### Requirement: Compact session hook prompt
The system SHALL provide a Codex `SessionStart` hook command that prompts the agent to run `polaris recall` after compaction.

#### Scenario: Prompt after compact session start
- **WHEN** `polaris hook session-start` receives Codex hook JSON with `hook_event_name: "SessionStart"` and `source: "compact"` in an initialized workspace with stored memory
- **THEN** the system prints JSON with `hookSpecificOutput.hookEventName: "SessionStart"` and `hookSpecificOutput.additionalContext` instructing the agent to run `polaris recall`

#### Scenario: Do not inline memory in hook output
- **WHEN** `polaris hook session-start` emits compact-session hook JSON
- **THEN** the `additionalContext` value contains only the recall instruction and MUST NOT contain stored memory contents

#### Scenario: Ignore non-compact session starts
- **WHEN** `polaris hook session-start` receives Codex hook JSON with `source` other than `compact`
- **THEN** the system exits successfully without stdout

#### Scenario: Stay quiet before initialization
- **WHEN** `polaris hook session-start` runs in a workspace without `.polaris/`
- **THEN** the system exits successfully without stdout

### Requirement: Codex hook example
The repository SHALL provide a copyable Codex hooks example that configures Polaris recall for compact session starts.

#### Scenario: Example configures compact session recall
- **WHEN** a user opens the bundled Codex hooks example
- **THEN** the example configures a `SessionStart` hook with matcher `compact` that runs `polaris hook session-start`

#### Scenario: Example is documented
- **WHEN** a user reads the Polaris README hook guidance
- **THEN** the README points to the bundled Codex hooks example and explains that `polaris` must be available to the hook command

#### Scenario: Example preserves hook memory safety
- **WHEN** a user follows the bundled Codex hooks example
- **THEN** the configured hook uses the existing Polaris hook command that prompts recall without inlining stored memory contents
