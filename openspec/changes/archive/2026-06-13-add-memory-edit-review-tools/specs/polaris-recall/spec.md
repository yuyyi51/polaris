## ADDED Requirements

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

## MODIFIED Requirements

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
