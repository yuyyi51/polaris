## ADDED Requirements

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
