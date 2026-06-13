## ADDED Requirements

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

### Requirement: Memory prune suggestions
The system SHALL suggest stale or redundant memory cleanup candidates without mutating memory.

#### Scenario: Suggest prune candidates
- **WHEN** `polaris prune --suggest` runs in an initialized workspace with old `state` or `log` memory records
- **THEN** the command exits successfully and prints cleanup suggestions with reasons
- **AND** active memory records remain unchanged

#### Scenario: Suggest prune candidates as JSON
- **WHEN** `polaris prune --suggest --json` runs in an initialized workspace
- **THEN** the command exits successfully and prints machine-readable suggestions including candidate keys, reasons, and suggested commands

#### Scenario: Report no prune suggestions
- **WHEN** `polaris prune --suggest` runs in an initialized workspace with no cleanup candidates
- **THEN** the command exits successfully and reports that no prune suggestions are available

### Requirement: Memory compact suggestions
The system SHALL suggest memory consolidation candidates without mutating memory.

#### Scenario: Suggest compact candidates
- **WHEN** `polaris compact --suggest` runs in an initialized workspace with multiple related keyed inline memories
- **THEN** the command exits successfully and prints consolidation suggestions with reasons
- **AND** active memory records remain unchanged

#### Scenario: Suggest compact candidates as JSON
- **WHEN** `polaris compact --suggest --json` runs in an initialized workspace
- **THEN** the command exits successfully and prints machine-readable suggestions including source keys, proposed target keys, reasons, and suggested commands

#### Scenario: Report no compact suggestions
- **WHEN** `polaris compact --suggest` runs in an initialized workspace with no consolidation candidates
- **THEN** the command exits successfully and reports that no compact suggestions are available
