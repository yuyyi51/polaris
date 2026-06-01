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
The system SHALL allow agents to create keyed inline memory entries in the active workspace.

#### Scenario: Record keyed text from an argument
- **WHEN** `polaris remember --key goal --text "task goal"` runs in an initialized workspace
- **THEN** the system appends an inline memory record containing the key `goal` and the provided text

#### Scenario: Record keyed text from standard input
- **WHEN** `polaris remember --key decision --stdin --title "Decision"` receives text on stdin in an initialized workspace
- **THEN** the system appends an inline memory record containing the key `decision`, the title, and stdin text

#### Scenario: Reject recording without a key
- **WHEN** `polaris remember --text "task goal"` runs in an initialized workspace
- **THEN** the system exits with an error explaining that `--key` is required
- **AND** no memory record is appended

#### Scenario: Reject duplicate key without replacement
- **WHEN** `polaris remember --key goal --text "new goal"` runs in an initialized workspace that already has an inline memory record whose key is `goal`
- **THEN** the system exits with an error explaining that the key already exists and replacement must be explicit
- **AND** the existing memory record remains unchanged

#### Scenario: Replace existing keyed memory
- **WHEN** `polaris remember --key goal --replace --text "new goal"` runs in an initialized workspace that already has an inline memory record whose key is `goal`
- **THEN** the system replaces the active memory record for `goal` with a record containing `new goal`
- **AND** later `polaris recall` output includes `new goal` and does not include the replaced text for `goal`

#### Scenario: Reject recording before initialization
- **WHEN** `polaris remember --key goal --text "task goal"` runs in a workspace without `.polaris/`
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
The system SHALL provide saved Polaris memory through `polaris recall`, including keys for keyed inline memory records.

#### Scenario: Recall mixed memory
- **WHEN** `polaris recall` runs in an initialized workspace with keyed short memories and long note records
- **THEN** the system prints a model-readable summary containing short memory keys, short memory text, note titles, and note file paths

#### Scenario: Recall legacy unkeyed memory
- **WHEN** `polaris recall` runs in an initialized workspace with an inline memory record created by an older Polaris version without a key
- **THEN** the system includes the memory text in recall output without error

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
The repository SHALL provide copyable hook examples that configure Polaris recall for compact-session recovery.

#### Scenario: Direct example configures compact session recall
- **WHEN** a user opens the bundled direct hook example
- **THEN** the example configures a `SessionStart` hook with matcher `compact` that runs `polaris hook session-start`

#### Scenario: Fallback example configures post-compact recall
- **WHEN** a user opens the bundled fallback hook example
- **THEN** the example configures a `PostCompact` hook that runs `polaris hook post-compact`
- **AND** the example configures a `PostToolUse` hook that runs `polaris hook post-tool-use`

#### Scenario: Examples are documented
- **WHEN** a user reads the Polaris README hook guidance
- **THEN** the README points to the bundled direct and fallback hook examples
- **AND** the README explains that `polaris` must be available to the hook command
- **AND** the README explains when to use the fallback example instead of the direct compact `SessionStart` example

#### Scenario: Examples preserve hook memory safety
- **WHEN** a user follows a bundled hook example
- **THEN** the configured hook commands prompt recall without inlining stored memory contents

### Requirement: Individual memory forgetting
The system SHALL allow agents to remove a keyed inline memory record from the active workspace without clearing all Polaris memory.

#### Scenario: Forget existing keyed memory
- **WHEN** `polaris forget goal` runs in an initialized workspace with an inline memory record whose key is `goal`
- **THEN** the system removes that record from active memory
- **AND** later `polaris recall` output does not include the forgotten record

#### Scenario: Reject forgetting an unknown key
- **WHEN** `polaris forget missing` runs in an initialized workspace with no inline memory record whose key is `missing`
- **THEN** the system exits with an error explaining that no memory exists for the key
- **AND** existing records remain unchanged

#### Scenario: Reject forgetting before initialization
- **WHEN** `polaris forget goal` runs in a workspace without `.polaris/`
- **THEN** the system exits with an error explaining that `polaris init` is required

#### Scenario: Forget preserves unrelated memory
- **WHEN** `polaris forget goal` runs in an initialized workspace with keyed inline records for `goal` and `plan` and a note record
- **THEN** the system removes only the inline record whose key is `goal`
- **AND** the `plan` record and note record remain recallable

### Requirement: Post-compact fallback hook prompt
The system SHALL provide fallback hook commands that record completion of conversation compaction and prompt the agent to run `polaris recall` on the next post-tool-use hook when recallable memory exists.

#### Scenario: Record pending recall after post compact
- **WHEN** `polaris hook post-compact` receives hook JSON with `hook_event_name: "PostCompact"` in an initialized workspace
- **THEN** the system records pending compact-recall state under `.polaris/`
- **AND** the command exits successfully without stdout

#### Scenario: Prompt after pending post tool use
- **WHEN** `polaris hook post-tool-use` receives hook JSON with `hook_event_name: "PostToolUse"` in an initialized workspace with stored memory and pending compact-recall state
- **THEN** the system prints JSON with `hookSpecificOutput.hookEventName: "PostToolUse"` and `hookSpecificOutput.additionalContext` instructing the agent to run `polaris recall`

#### Scenario: Consume pending recall state
- **WHEN** `polaris hook post-tool-use` checks a pending compact-recall state
- **THEN** the system consumes the pending state
- **AND** a later `polaris hook post-tool-use` invocation without a new `polaris hook post-compact` invocation exits successfully without stdout

#### Scenario: Do not inline memory in fallback hook output
- **WHEN** `polaris hook post-tool-use` emits fallback hook JSON
- **THEN** the `additionalContext` value contains only the recall instruction and MUST NOT contain stored memory contents

#### Scenario: Stay quiet when fallback has no memory
- **WHEN** `polaris hook post-tool-use` receives hook JSON with `hook_event_name: "PostToolUse"` in an initialized workspace with pending compact-recall state and no stored memory
- **THEN** the system consumes the pending state
- **AND** the command exits successfully without stdout

#### Scenario: Ignore unrelated fallback events
- **WHEN** `polaris hook post-compact` or `polaris hook post-tool-use` receives hook JSON whose `hook_event_name` does not match that command's expected event
- **THEN** the command exits successfully without stdout

#### Scenario: Stay quiet before initialization
- **WHEN** `polaris hook post-compact` or `polaris hook post-tool-use` runs in a workspace without `.polaris/`
- **THEN** the command exits successfully without stdout

