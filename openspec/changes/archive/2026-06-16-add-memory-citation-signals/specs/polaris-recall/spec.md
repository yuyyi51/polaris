## ADDED Requirements

### Requirement: Memory citation recording
The system SHALL allow agents to explicitly cite memory records that were used as evidence for work without mutating recall or freshness metadata.

#### Scenario: Cite memory by key
- **WHEN** `polaris cite --key goal` runs in an initialized workspace with an active keyed inline memory `goal`
- **THEN** the command records one citation event for the active `goal` record id
- **AND** the command exits successfully
- **AND** the active memory record text, lifecycle, `created_at`, and `updated_at` remain unchanged

#### Scenario: Cite memory by id
- **WHEN** `polaris cite --id <record-id>` runs in an initialized workspace with a memory record whose id is `<record-id>`
- **THEN** the command records one citation event for `<record-id>`
- **AND** the command exits successfully
- **AND** the cited memory record contents and freshness metadata remain unchanged

#### Scenario: Cite multiple memories
- **WHEN** `polaris cite --keys goal,decision.product_model --quiet` runs in an initialized workspace with active keyed inline memories for both keys
- **THEN** the command records one citation event for each matching active record id
- **AND** the command exits successfully without normal success output

#### Scenario: Cite multiple memories with missing selectors
- **WHEN** `polaris cite --keys goal,missing` runs in an initialized workspace with an active keyed inline memory for `goal` and no memory for `missing`
- **THEN** the command records one citation event for `goal`
- **AND** the command reports `missing` as a missing key
- **AND** the command exits successfully

#### Scenario: Reject citation when no selector matches
- **WHEN** `polaris cite --keys missing,absent` runs in an initialized workspace with no matching active keyed inline memories
- **THEN** the command exits with an error explaining that no requested memory records were cited
- **AND** no citation event is recorded

#### Scenario: Reject broad citation selectors
- **WHEN** `polaris cite --prefix decision.` runs in an initialized workspace
- **THEN** the command exits with an argument error
- **AND** no citation event is recorded

#### Scenario: Reject citation selector conflicts
- **WHEN** `polaris cite --key goal --id <record-id>` runs in an initialized workspace
- **THEN** the command exits with an error explaining that citation selectors conflict
- **AND** no citation event is recorded

#### Scenario: Reject citation before initialization
- **WHEN** `polaris cite --key goal` runs in a workspace without `.polaris/`
- **THEN** the command exits with an error explaining that `polaris init` is required

### Requirement: Citation metadata listing
The system SHALL expose citation metadata in machine-readable memory summaries without requiring agents to parse citation event files.

#### Scenario: List citation metadata for cited memory
- **WHEN** `polaris list --json` runs in an initialized workspace with an active memory record that has been cited twice
- **THEN** the matching memory summary includes `direct_cite_count: 2`
- **AND** the summary includes `last_cited_at`
- **AND** the summary preserves the memory record `created_at`

#### Scenario: List citation metadata for uncited memory
- **WHEN** `polaris list --json` runs in an initialized workspace with an active memory record that has no citation events
- **THEN** the matching memory summary includes `direct_cite_count: 0`
- **AND** the summary includes `inherited_cite_count: 0`
- **AND** the summary does not report the memory as stale or removable solely because it has no citations

#### Scenario: Missing citation history is empty
- **WHEN** `polaris list --json` runs in an initialized workspace without `.polaris/citations.jsonl`
- **THEN** the command exits successfully
- **AND** memory summaries report zero direct and inherited citation counts

### Requirement: Citation lineage
The system SHALL preserve citation lineage across memory replacement and merge operations while distinguishing direct citations from inherited citations.

#### Scenario: Replacement inherits prior citations
- **WHEN** an active keyed inline memory has citation events
- **AND** `polaris remember --key goal --replace --text "updated goal"` replaces that memory
- **THEN** the new active `goal` record has `direct_cite_count: 0` in `polaris list --json`
- **AND** the new active `goal` summary reports inherited citation count from the replaced record
- **AND** the prior citation events remain associated with the replaced record id

#### Scenario: Merge inherits source citations
- **WHEN** `polaris merge apply .polaris/maintenance/<draft>.md --yes` creates a merged target from cited source memories
- **THEN** the merged target summary has `direct_cite_count: 0` until the target itself is cited
- **AND** the merged target summary reports inherited citation count from the source records
- **AND** the inherited citation count does not double-count citation events reachable through multiple lineage paths

#### Scenario: Direct citation after lineage
- **WHEN** a replacement or merged target with inherited citations is later cited directly
- **THEN** the target summary increments `direct_cite_count`
- **AND** the target summary preserves inherited citation count separately

### Requirement: Citation-aware maintenance context
The system SHALL make citation metadata available as advisory context for memory review without making citation counts automatic maintenance decisions.

#### Scenario: Prune suggestions remain advisory with citation data
- **WHEN** `polaris prune --suggest --json` runs in an initialized workspace with cited and uncited candidate memories
- **THEN** the command exits successfully
- **AND** `prompt_context` includes citation summaries for candidate records with `created_at`, `direct_cite_count`, and `inherited_cite_count`
- **AND** the command does not suggest deletion solely because a memory has zero citations

#### Scenario: Compact suggestions remain advisory with citation data
- **WHEN** `polaris compact --suggest --json` runs in an initialized workspace with cited and uncited related memories
- **THEN** the command exits successfully
- **AND** `prompt_context` includes citation summaries for source records with `created_at`, `direct_cite_count`, and `inherited_cite_count`
- **AND** the command does not suggest merging solely because memories have citation metadata

### Requirement: Citation event persistence
The system SHALL preserve citation events as valid JSONL when citation commands run concurrently with other memory operations.

#### Scenario: Concurrent citation recording
- **WHEN** multiple `polaris cite --key <key>` commands run concurrently in the same initialized workspace
- **THEN** each successful command records complete citation events
- **AND** `.polaris/citations.jsonl` contains one complete JSON object per non-empty line
- **AND** later `polaris list --json` exits successfully

#### Scenario: Ignore blank citation lines
- **WHEN** `.polaris/citations.jsonl` contains blank or whitespace-only lines between valid citation events
- **THEN** `polaris list --json` ignores those blank lines
- **AND** the command exits successfully

#### Scenario: Report malformed citation record location
- **WHEN** `.polaris/citations.jsonl` contains a malformed non-empty line
- **AND** `polaris list --json` reads citation metadata
- **THEN** the command exits with an error that includes `.polaris/citations.jsonl`
- **AND** the error includes the physical line number of the malformed line
- **AND** the error includes the underlying JSON parse failure
