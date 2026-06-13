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

#### Scenario: Recall by multiple exact keys
- **WHEN** `polaris recall --keys goal,plan,state.branch` runs in an initialized workspace with matching keyed inline memories
- **THEN** the command exits successfully and prints the matching memories in the requested key order
- **AND** the command does not print unrelated memory records

#### Scenario: Recall by multiple exact keys with missing keys
- **WHEN** `polaris recall --keys goal,missing` runs in an initialized workspace with a keyed inline memory for `goal` and no keyed inline memory for `missing`
- **THEN** the command exits successfully and prints the `goal` memory
- **AND** the output reports `missing` as a missing key

#### Scenario: Recall by multiple exact keys with all keys missing
- **WHEN** `polaris recall --keys missing,absent` runs in an initialized workspace with no matching keyed inline memories
- **THEN** the command exits successfully and reports that no matching Polaris memory is stored
- **AND** the output reports the missing keys

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

#### Scenario: Recall multiple keys with lifecycle and exclude filters
- **WHEN** `polaris recall --keys goal,state.branch --lifecycle durable --exclude state.` runs in an initialized workspace with durable `goal` and state `state.branch` records
- **THEN** the command exits successfully and prints the `goal` memory
- **AND** the output reports `state.branch` as missing or filtered

#### Scenario: Reject conflicting recall selectors
- **WHEN** `polaris recall --key goal --prefix decision.` runs in an initialized workspace
- **THEN** the command exits with an error explaining that exact key and prefix selectors conflict

#### Scenario: Reject multi-key recall selector conflicts
- **WHEN** `polaris recall --keys goal,plan --key goal` runs in an initialized workspace
- **THEN** the command exits with an error explaining that `--keys` conflicts with other exact key selectors

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

### Requirement: Memory lifecycle move
The system SHALL allow agents to reclassify existing memory records by lifecycle without rewriting memory text or note files.

#### Scenario: Move inline memory lifecycle by key
- **WHEN** `polaris lifecycle move --key state.branch --to state` runs in an initialized workspace with a keyed inline memory `state.branch`
- **THEN** the command exits successfully and changes that memory lifecycle to `state`
- **AND** the memory text and `created_at` timestamp remain unchanged
- **AND** the memory has an `updated_at` timestamp

#### Scenario: Move inline memory lifecycle by prefix
- **WHEN** `polaris lifecycle move --prefix state. --from durable --to state --yes` runs in an initialized workspace with durable keyed inline memories whose keys start with `state.`
- **THEN** the command exits successfully and changes matching memories from `durable` to `state`
- **AND** active memory text remains unchanged

#### Scenario: Move note memory lifecycle by id
- **WHEN** `polaris lifecycle move --id <record-id> --to archive` runs in an initialized workspace with a note memory whose id is `<record-id>`
- **THEN** the command exits successfully and changes that note memory lifecycle to `archive`
- **AND** the note path, note file contents, title, and `created_at` timestamp remain unchanged
- **AND** the note memory has an `updated_at` timestamp

#### Scenario: Move memory lifecycle by kind
- **WHEN** `polaris lifecycle move --kind note --from log --to archive --yes` runs in an initialized workspace with log note memories
- **THEN** the command exits successfully and changes matching note memories from `log` to `archive`
- **AND** inline memories remain unchanged

#### Scenario: Refuse unconfirmed batch lifecycle move
- **WHEN** `polaris lifecycle move --prefix state. --to state` runs without `--yes`
- **THEN** the command exits with an error explaining that batch lifecycle moves require `--yes`
- **AND** existing memory records remain unchanged

#### Scenario: Reject ambiguous lifecycle move selectors
- **WHEN** `polaris lifecycle move --key state.branch --id <record-id> --to state` runs in an initialized workspace
- **THEN** the command exits with an error explaining that lifecycle move selectors conflict
- **AND** existing memory records remain unchanged

#### Scenario: Report lifecycle move as JSON
- **WHEN** `polaris lifecycle move --prefix state. --to state --yes --json` runs in an initialized workspace
- **THEN** the command exits successfully and prints a JSON object containing moved records and skipped records
- **AND** each moved record includes stable metadata such as id, kind, previous lifecycle, new lifecycle, and any available key, title, or path

### Requirement: Replacement metadata
The system SHALL preserve lightweight update metadata and saved previous snapshots for keyed inline memory replacements.

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

#### Scenario: Save previous replacement snapshot
- **WHEN** `polaris remember --key goal --replace --text "updated"` replaces an existing keyed inline memory
- **THEN** the system saves the previous active inline memory record as replacement history
- **AND** the active replacement record references the saved previous record through `replaced_from`
- **AND** later `polaris diff --key goal` can compare the saved previous text with the current active text

### Requirement: Memory replacement diff
The system SHALL allow agents to review the latest keyed inline memory replacement by comparing the active record with the latest saved replacement snapshot.

#### Scenario: Diff latest replacement
- **WHEN** `polaris diff --key goal` runs after `goal` has been replaced by `polaris remember --key goal --replace`
- **THEN** the command exits successfully and prints a unified diff between the previous snapshot text and the current active text
- **AND** the output identifies the key being compared

#### Scenario: Diff latest replacement as JSON
- **WHEN** `polaris diff --key goal --json` runs after `goal` has been replaced by `polaris remember --key goal --replace`
- **THEN** the command exits successfully and prints a JSON object containing the key, current record id, previous record id, and diff data
- **AND** the JSON output is stable enough for agents to inspect without parsing human diff text

#### Scenario: Report missing replacement snapshot
- **WHEN** `polaris diff --key goal` runs for a keyed inline memory that has no saved replacement snapshot
- **THEN** the command exits successfully and reports that no replacement snapshot is available for `goal`

#### Scenario: Reject diff for missing key
- **WHEN** `polaris diff --key missing` runs in an initialized workspace with no active keyed inline memory for `missing`
- **THEN** the command exits with an error explaining that no memory exists for the key

#### Scenario: Reject diff before initialization
- **WHEN** `polaris diff --key goal` runs in a workspace without `.polaris/`
- **THEN** the command exits with an error explaining that `polaris init` is required

### Requirement: Replacement draft preview
The system SHALL allow agents to preview and edit keyed inline memory replacements before mutating active memory.

#### Scenario: Create replacement draft preview from text
- **WHEN** `polaris remember --key goal --replace --dry-run --text "new goal"` runs in an initialized workspace with an existing keyed inline memory `goal`
- **THEN** the command exits successfully and creates a replacement draft under `.polaris/maintenance/`
- **AND** the command prints a preview diff between the current memory text and the proposed replacement text
- **AND** active memory records and replacement history remain unchanged

#### Scenario: Create replacement draft preview from stdin
- **WHEN** `polaris remember --key goal --replace --dry-run --stdin` receives replacement text in an initialized workspace with an existing keyed inline memory `goal`
- **THEN** the command exits successfully and creates a replacement draft containing the stdin text under `## Replacement Memory`
- **AND** active memory records and replacement history remain unchanged

#### Scenario: Reject dry-run without replace
- **WHEN** `polaris remember --key goal --dry-run --text "new goal"` runs in an initialized workspace
- **THEN** the command exits with an error explaining that `--dry-run` requires `--replace`
- **AND** active memory records remain unchanged

#### Scenario: Apply replacement draft
- **WHEN** `polaris replace apply .polaris/maintenance/<draft>.md --yes` runs for a valid edited replacement draft
- **THEN** the system replaces the target keyed inline memory using the edited draft content
- **AND** the replacement preserves normal replacement metadata behavior and writes replacement history
- **AND** later `polaris diff --key <key>` can compare the previous text with the applied replacement text

#### Scenario: Refuse unconfirmed replacement draft apply
- **WHEN** `polaris replace apply .polaris/maintenance/<draft>.md` runs without `--yes`
- **THEN** the command refuses to modify memory and explains that `--yes` is required

#### Scenario: Reject invalid replacement draft
- **WHEN** `polaris replace apply .polaris/maintenance/bad.md --yes` runs for a file that is not a valid Polaris replacement draft
- **THEN** the command exits with an error explaining that the draft cannot be applied
- **AND** active memory records remain unchanged

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

### Requirement: Memory key rename
The system SHALL allow agents to rename keyed inline memories without changing memory text.

#### Scenario: Rename existing memory key
- **WHEN** `polaris rename old.key new.key` runs in an initialized workspace with a keyed inline memory for `old.key`
- **THEN** the system changes the memory key to `new.key`
- **AND** later `polaris recall --key new.key` includes the original memory text
- **AND** later `polaris recall --key old.key` reports no matching Polaris memory

#### Scenario: Reject rename for missing source
- **WHEN** `polaris rename missing new.key` runs in an initialized workspace with no keyed inline memory for `missing`
- **THEN** the command exits with an error explaining that the source key does not exist
- **AND** existing records remain unchanged

#### Scenario: Reject rename to existing destination
- **WHEN** `polaris rename old.key existing.key` runs in an initialized workspace where both keys already exist
- **THEN** the command exits with an error explaining that the destination key already exists
- **AND** existing records remain unchanged

### Requirement: Memory merge drafts
The system SHALL create editable merge drafts that gather multiple source memories without mutating active memory.

#### Scenario: Create merge draft
- **WHEN** `polaris merge --into model.summary keyA keyB` runs in an initialized workspace with keyed inline memories for `keyA` and `keyB`
- **THEN** the command creates a markdown draft under `.polaris/maintenance/`
- **AND** the draft includes the target key, source keys, source metadata, and source text
- **AND** active memory records remain unchanged

#### Scenario: Reject merge draft with missing source
- **WHEN** `polaris merge --into model.summary keyA missing` runs in an initialized workspace with no keyed inline memory for `missing`
- **THEN** the command exits with an error explaining which source key is missing
- **AND** no merge draft is created

#### Scenario: Reject merge draft with empty target
- **WHEN** `polaris merge --into "" keyA keyB` runs in an initialized workspace
- **THEN** the command exits with an error explaining that the target key must not be empty

### Requirement: Memory merge apply
The system SHALL apply edited merge drafts only through an explicit confirmed command.

#### Scenario: Apply merge draft
- **WHEN** `polaris merge apply .polaris/maintenance/<draft>.md --yes` runs for a valid edited merge draft
- **THEN** the system records or replaces the target keyed inline memory using the edited draft content
- **AND** the command reports the target key that was written

#### Scenario: Apply merge draft and remove sources
- **WHEN** `polaris merge apply .polaris/maintenance/<draft>.md --yes --forget-sources` runs for a valid edited merge draft
- **THEN** the system writes the target memory
- **AND** removes the draft source keyed inline memories from active memory

#### Scenario: Refuse unconfirmed merge apply
- **WHEN** `polaris merge apply .polaris/maintenance/<draft>.md` runs without `--yes`
- **THEN** the command refuses to modify memory and explains that `--yes` is required

#### Scenario: Reject invalid merge draft
- **WHEN** `polaris merge apply .polaris/maintenance/bad.md --yes` runs for a file that is not a valid Polaris merge draft
- **THEN** the command exits with an error explaining that the draft cannot be applied
- **AND** active memory records remain unchanged

### Requirement: Memory touch
The system SHALL allow agents to refresh memory `updated_at` timestamps without changing memory content or classification.

#### Scenario: Touch memory by key
- **WHEN** `polaris touch --key state.branch` runs in an initialized workspace with a keyed inline memory `state.branch`
- **THEN** the command exits successfully and updates that memory `updated_at`
- **AND** the memory text, lifecycle, `created_at`, `replacement_count`, and `replaced_from` remain unchanged

#### Scenario: Touch memory by keys
- **WHEN** `polaris touch --keys state.branch,state.queue` runs in an initialized workspace with keyed inline memories for both keys
- **THEN** the command exits successfully and updates `updated_at` for both memories
- **AND** the command does not require `--yes`

#### Scenario: Touch memory by id
- **WHEN** `polaris touch --id <record-id>` runs in an initialized workspace with a memory record whose id is `<record-id>`
- **THEN** the command exits successfully and updates that memory `updated_at`
- **AND** note file contents remain unchanged when the touched record is a note

#### Scenario: Touch memory by confirmed prefix
- **WHEN** `polaris touch --prefix state. --yes` runs in an initialized workspace with keyed inline memories whose keys start with `state.`
- **THEN** the command exits successfully and updates `updated_at` for matching memories
- **AND** non-matching memories remain unchanged

#### Scenario: Refuse unconfirmed prefix touch
- **WHEN** `polaris touch --prefix state.` runs without `--yes`
- **THEN** the command exits with an error explaining that prefix touch requires `--yes`
- **AND** existing memory records remain unchanged

#### Scenario: Reject ambiguous touch selectors
- **WHEN** `polaris touch --key state.branch --id <record-id>` runs in an initialized workspace
- **THEN** the command exits with an error explaining that touch selectors conflict
- **AND** existing memory records remain unchanged

#### Scenario: Report touch as JSON
- **WHEN** `polaris touch --keys state.branch,state.queue --json` runs in an initialized workspace
- **THEN** the command exits successfully and prints a JSON object containing touched records, missing keys, and skipped records
- **AND** each touched record includes stable metadata such as id, kind, previous `updated_at`, new `updated_at`, and any available key, title, or path

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
