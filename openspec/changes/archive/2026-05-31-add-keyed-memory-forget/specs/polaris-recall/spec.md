## ADDED Requirements

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

## MODIFIED Requirements

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
