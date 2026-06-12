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

### Requirement: Hook recall prompt configuration
The system SHALL allow Codex hook recall prompts to be configured by TOML files while preserving the built-in prompt when no configuration exists.

#### Scenario: Use workspace hook prompt before user hook prompt
- **WHEN** `.polaris/config.toml` in the active workspace and `~/.polaris/config.toml` both define `[hooks].recall_prompt`
- **THEN** hook output uses the workspace `recall_prompt`

#### Scenario: Use user hook prompt when workspace config is absent
- **WHEN** `.polaris/config.toml` is absent and `~/.polaris/config.toml` defines `[hooks].recall_prompt`
- **THEN** hook output uses the user `recall_prompt`

#### Scenario: Use built-in prompt when no hook config exists
- **WHEN** neither `.polaris/config.toml` nor `~/.polaris/config.toml` exists
- **THEN** hook output uses the built-in recall instruction
- **AND** the output MUST NOT contain stored memory contents

#### Scenario: Inline recall with placeholder
- **WHEN** the selected `recall_prompt` contains `{{recall}}`
- **THEN** hook output replaces each `{{recall}}` placeholder with the current `polaris recall` output

#### Scenario: Configured prompt without placeholder does not inline recall
- **WHEN** the selected `recall_prompt` does not contain `{{recall}}`
- **THEN** hook output uses the configured prompt text
- **AND** the output MUST NOT contain stored memory contents unless they are present in the configured prompt itself

### Requirement: Compact session hook prompt
The system SHALL provide a Codex `SessionStart` hook command that adds recall context after compaction.

#### Scenario: Prompt after compact session start
- **WHEN** `polaris hook session-start` receives Codex hook JSON with `hook_event_name: "SessionStart"` and `source: "compact"` in an initialized workspace with stored memory
- **THEN** the system prints JSON with `hookSpecificOutput.hookEventName: "SessionStart"` and `hookSpecificOutput.additionalContext` rendered from the selected hook recall prompt

#### Scenario: Default compact prompt does not inline memory
- **WHEN** `polaris hook session-start` emits compact-session hook JSON without a configured hook recall prompt
- **THEN** the `additionalContext` value contains only the built-in recall instruction and MUST NOT contain stored memory contents

#### Scenario: Configured compact prompt can inline recall
- **WHEN** `polaris hook session-start` emits compact-session hook JSON and the selected hook recall prompt contains `{{recall}}`
- **THEN** the `additionalContext` value contains the current `polaris recall` output

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
The system SHALL provide fallback hook commands that record completion of conversation compaction and add recall context on the next post-tool-use hook when recallable memory exists.

#### Scenario: Record pending recall after post compact
- **WHEN** `polaris hook post-compact` receives hook JSON with `hook_event_name: "PostCompact"` in an initialized workspace
- **THEN** the system records pending compact-recall state under `.polaris/`
- **AND** the command exits successfully without stdout

#### Scenario: Prompt after pending post tool use
- **WHEN** `polaris hook post-tool-use` receives hook JSON with `hook_event_name: "PostToolUse"` in an initialized workspace with stored memory and pending compact-recall state
- **THEN** the system prints JSON with `hookSpecificOutput.hookEventName: "PostToolUse"` and `hookSpecificOutput.additionalContext` rendered from the selected hook recall prompt

#### Scenario: Consume pending recall state
- **WHEN** `polaris hook post-tool-use` checks a pending compact-recall state
- **THEN** the system consumes the pending state
- **AND** a later `polaris hook post-tool-use` invocation without a new `polaris hook post-compact` invocation exits successfully without stdout

#### Scenario: Default fallback prompt does not inline memory
- **WHEN** `polaris hook post-tool-use` emits fallback hook JSON without a configured hook recall prompt
- **THEN** the `additionalContext` value contains only the built-in recall instruction and MUST NOT contain stored memory contents

#### Scenario: Configured fallback prompt can inline recall
- **WHEN** `polaris hook post-tool-use` emits fallback hook JSON and the selected hook recall prompt contains `{{recall}}`
- **THEN** the `additionalContext` value contains the current `polaris recall` output

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

### Requirement: Concurrent memory persistence
The system SHALL preserve `.polaris/memories.jsonl` as valid JSONL when multiple Polaris processes access workspace memory concurrently.

#### Scenario: Concurrent keyed memory recording
- **WHEN** multiple `polaris remember --key <key> --text <text>` commands run concurrently in the same initialized workspace with distinct keys
- **THEN** each successful command records exactly one memory entry
- **AND** `.polaris/memories.jsonl` contains one complete JSON object per non-empty line
- **AND** later `polaris recall` and `polaris status --json` exit successfully

#### Scenario: Concurrent keyed memory replacement
- **WHEN** multiple `polaris remember --key <key> --replace --text <text>` commands run concurrently in the same initialized workspace
- **THEN** each successful command's replacement is preserved unless a later successful replacement for the same key supersedes it
- **AND** unrelated memory records are not lost by stale whole-file rewrites
- **AND** later `polaris recall` and `polaris status --json` exit successfully

#### Scenario: Append after missing final newline
- **WHEN** `.polaris/memories.jsonl` is non-empty and does not end with a newline
- **AND** `polaris remember --key <key> --text <text>` records a new memory
- **THEN** the new record is separated from the previous record by a newline
- **AND** later `polaris recall` exits successfully

#### Scenario: Readers wait for consistent memory state
- **WHEN** one Polaris process is mutating `.polaris/memories.jsonl`
- **AND** another Polaris process runs a command that reads memory records
- **THEN** the reading process observes a consistent memory file state
- **AND** it does not fail because of a partially written record from the mutating process

### Requirement: Memory JSONL diagnostics
The system SHALL report malformed `.polaris/memories.jsonl` records with actionable file and line context while ignoring blank lines.

#### Scenario: Ignore blank memory lines
- **WHEN** `.polaris/memories.jsonl` contains blank or whitespace-only lines between valid memory records
- **THEN** `polaris recall` and `polaris status --json` ignore those blank lines
- **AND** both commands exit successfully

#### Scenario: Report malformed memory record location
- **WHEN** `.polaris/memories.jsonl` contains a malformed non-empty line
- **AND** `polaris recall` or `polaris status --json` reads memory records
- **THEN** the command exits with an error that includes `.polaris/memories.jsonl`
- **AND** the error includes the physical line number of the malformed line
- **AND** the error includes the underlying JSON parse failure.

### Requirement: Memory listing
The system SHALL provide stable list commands for inspecting stored Polaris memory without parsing human recall output.

#### Scenario: List keyed inline memory keys
- **WHEN** `polaris list --keys` runs in an initialized workspace with keyed inline memory records and note records
- **THEN** the command exits successfully and prints one inline memory key per line
- **AND** the output does not include note records that have no key

#### Scenario: List memory summaries as JSON
- **WHEN** `polaris list --json` runs in an initialized workspace with inline memory records and note records
- **THEN** the command exits successfully and prints a JSON array of memory summary objects
- **AND** each summary includes stable metadata for the record such as id, kind, created_at, and any available key, title, or path
- **AND** inline memory text bodies are not included in the summaries

#### Scenario: Reject list before initialization
- **WHEN** `polaris list --json` or `polaris list --keys` runs in a workspace without `.polaris/`
- **THEN** the command exits with an error explaining that `polaris init` is required

### Requirement: Filtered context recall
The system SHALL allow agents to recall selected keyed inline memories without printing the entire workspace memory set.

#### Scenario: Recall by exact key
- **WHEN** `polaris recall --key goal` runs in an initialized workspace with keyed inline records for `goal` and `plan`
- **THEN** the command exits successfully and prints the `goal` memory
- **AND** the output does not include the `plan` memory

#### Scenario: Recall by key prefix
- **WHEN** `polaris recall --prefix decision.` runs in an initialized workspace with keyed inline records for `decision.storage`, `decision.hooks`, and `goal`
- **THEN** the command exits successfully and prints the memories whose keys start with `decision.`
- **AND** the output does not include the `goal` memory

#### Scenario: Recall with excluded key prefix
- **WHEN** `polaris recall --exclude state.` runs in an initialized workspace with keyed inline records for `goal` and `state.branch`
- **THEN** the command exits successfully and prints the `goal` memory
- **AND** the output does not include the `state.branch` memory

#### Scenario: Recall prefix with excluded sub-prefix
- **WHEN** `polaris recall --prefix decision. --exclude decision.old.` runs in an initialized workspace with keyed inline records for `decision.current` and `decision.old.storage`
- **THEN** the command exits successfully and prints the `decision.current` memory
- **AND** the output does not include the `decision.old.storage` memory

#### Scenario: Reject conflicting recall selectors
- **WHEN** `polaris recall --key goal --prefix decision.` runs in an initialized workspace
- **THEN** the command exits with an error explaining that exact key and prefix selectors conflict

#### Scenario: Report no matching filtered recall
- **WHEN** `polaris recall --key missing` runs in an initialized workspace with no keyed inline memory for `missing`
- **THEN** the command exits successfully and reports that no matching Polaris memory is stored

### Requirement: Batch memory forgetting
The system SHALL allow agents to remove multiple keyed inline memories from the active workspace with one explicit command.

#### Scenario: Forget multiple existing keys
- **WHEN** `polaris forget goal plan` runs in an initialized workspace with keyed inline records for `goal`, `plan`, and `decision`
- **THEN** the system removes the `goal` and `plan` records from active memory
- **AND** later `polaris recall` output includes the `decision` record and does not include the forgotten records

#### Scenario: Reject batch forget with an unknown key
- **WHEN** `polaris forget goal missing` runs in an initialized workspace with a keyed inline record for `goal` and no keyed inline record for `missing`
- **THEN** the command exits with an error explaining which requested key is missing
- **AND** the existing `goal` record remains recallable

#### Scenario: Reject batch forget before initialization
- **WHEN** `polaris forget goal plan` runs in a workspace without `.polaris/`
- **THEN** the command exits with an error explaining that `polaris init` is required

### Requirement: Prefix memory forgetting
The system SHALL allow agents to remove keyed inline memories by key prefix only when the broad deletion is explicitly confirmed.

#### Scenario: Forget memories by confirmed prefix
- **WHEN** `polaris forget --prefix decision. --yes` runs in an initialized workspace with keyed inline records for `decision.storage`, `decision.hooks`, and `goal`
- **THEN** the system removes the records whose keys start with `decision.`
- **AND** later `polaris recall` output includes the `goal` record and does not include the forgotten records

#### Scenario: Refuse unconfirmed prefix forget
- **WHEN** `polaris forget --prefix decision.` runs in an initialized workspace
- **THEN** the command refuses to remove memory and explains that `--yes` is required

#### Scenario: Reject empty prefix forget
- **WHEN** `polaris forget --prefix "" --yes` runs in an initialized workspace
- **THEN** the command exits with an error explaining that prefix must not be empty
- **AND** existing records remain unchanged

#### Scenario: Reject prefix forget with no matches
- **WHEN** `polaris forget --prefix missing. --yes` runs in an initialized workspace with no keyed inline records whose keys start with `missing.`
- **THEN** the command exits with an error explaining that no memory matches the prefix
- **AND** existing records remain unchanged

#### Scenario: Reject conflicting forget selectors
- **WHEN** `polaris forget goal --prefix decision. --yes` runs in an initialized workspace
- **THEN** the command exits with an error explaining that positional keys and prefix deletion conflict

### Requirement: Memory lifecycle metadata
The system SHALL allow stored memory records to carry lifecycle metadata that distinguishes durable knowledge from volatile or archived context.

#### Scenario: Record inline memory with lifecycle
- **WHEN** `polaris remember --key branch --text "working on filters" --lifecycle state` runs in an initialized workspace
- **THEN** the system records an inline memory whose lifecycle is `state`
- **AND** later `polaris list --json` includes `lifecycle: "state"` for that record

#### Scenario: Record note with lifecycle
- **WHEN** `polaris note create --title "Release archive" --lifecycle archive` runs in an initialized workspace
- **THEN** the system records a note memory whose lifecycle is `archive`
- **AND** later `polaris list --json` includes `lifecycle: "archive"` for that note record

#### Scenario: Default legacy lifecycle
- **WHEN** `polaris list --json` reads a valid memory record that has no lifecycle field
- **THEN** the command treats the record as lifecycle `durable`
- **AND** the command exits successfully

#### Scenario: Reject invalid lifecycle
- **WHEN** `polaris remember --key goal --text "task" --lifecycle temporary` runs in an initialized workspace
- **THEN** the command exits with an error explaining the supported lifecycle values
- **AND** no memory record is appended

### Requirement: Lifecycle-aware recall and listing
The system SHALL allow agents to inspect and recall memory by lifecycle.

#### Scenario: List memory by lifecycle
- **WHEN** `polaris list --json --lifecycle state` runs in an initialized workspace with `state` and `durable` records
- **THEN** the command exits successfully and prints only summaries for records whose lifecycle is `state`

#### Scenario: Recall memory by lifecycle
- **WHEN** `polaris recall --lifecycle durable` runs in an initialized workspace with durable and state records
- **THEN** the command exits successfully and prints durable records
- **AND** the output does not include state records

#### Scenario: Combine lifecycle and key filters
- **WHEN** `polaris recall --prefix decision. --lifecycle durable` runs in an initialized workspace with durable and archived `decision.` records
- **THEN** the command exits successfully and prints only matching durable records whose keys start with `decision.`

#### Scenario: Report no lifecycle matches
- **WHEN** `polaris recall --lifecycle log` runs in an initialized workspace with no log records
- **THEN** the command exits successfully and reports that no matching Polaris memory is stored

### Requirement: Replacement metadata
The system SHALL preserve lightweight update metadata for keyed inline memory replacements.

#### Scenario: Replace records update metadata
- **WHEN** `polaris remember --key goal --replace --text "updated"` replaces an existing keyed inline memory
- **THEN** the active `goal` memory has an `updated_at` timestamp
- **AND** the active `goal` memory has `replacement_count` incremented from the previous active value
- **AND** later `polaris list --json` exposes the replacement metadata

#### Scenario: Preserve original creation time on replacement
- **WHEN** `polaris remember --key goal --replace --text "updated"` replaces an existing keyed inline memory that has `created_at`
- **THEN** the active replacement record preserves the original `created_at`
- **AND** the active replacement record has `updated_at` later than or equal to `created_at`

#### Scenario: Preserve lifecycle on replacement by default
- **WHEN** `polaris remember --key branch --replace --text "new branch"` replaces an existing keyed inline memory whose lifecycle is `state`
- **THEN** the replacement memory keeps lifecycle `state`

#### Scenario: Override lifecycle on replacement
- **WHEN** `polaris remember --key note --replace --text "archived" --lifecycle archive` replaces an existing keyed inline memory
- **THEN** the replacement memory has lifecycle `archive`

### Requirement: Lifecycle status summary
The system SHALL include lifecycle counts and advisory volatile-memory hints in machine-readable workspace status.

#### Scenario: Status includes lifecycle counts
- **WHEN** `polaris status --json` runs in an initialized workspace with durable, state, log, and archive memories
- **THEN** the command exits successfully and reports memory counts grouped by lifecycle

#### Scenario: Status includes stale volatile hints
- **WHEN** `polaris status --json` runs in an initialized workspace with old `state` or `log` memories
- **THEN** the command exits successfully and includes advisory stale-memory hint data for those volatile lifecycle records
- **AND** the hint does not cause the command to fail

#### Scenario: Status handles legacy records
- **WHEN** `polaris status --json` runs in an initialized workspace with memory records that have no lifecycle field
- **THEN** those records are counted as durable memory

