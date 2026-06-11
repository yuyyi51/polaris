## ADDED Requirements

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
