## ADDED Requirements

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

## MODIFIED Requirements

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
