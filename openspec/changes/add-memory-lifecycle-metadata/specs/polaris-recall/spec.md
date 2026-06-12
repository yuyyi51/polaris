## ADDED Requirements

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
