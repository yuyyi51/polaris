## MODIFIED Requirements

### Requirement: Start and resume workflow guidance
The skill SHALL teach agents how to start or resume substantial work with Polaris.

#### Scenario: Workspace status is unknown
- **WHEN** an agent starts or resumes substantial work
- **THEN** the skill directs the agent to run `polaris status --json`

#### Scenario: Workspace should use Polaris
- **WHEN** substantial work is long-running, multi-step, risky, or likely to survive compaction
- **THEN** the skill directs the agent to initialize Polaris with `polaris init` if needed

#### Scenario: Existing memory may be stale
- **WHEN** `polaris status --json` shows existing memory that may not belong to the current task
- **THEN** the skill directs the agent to ask the user before removing memory
- **AND** the skill presents `polaris forget <key>` for selected keyed records and `polaris clear --yes` only for clearing all memory

### Requirement: Memory recording guidance
The skill SHALL teach agents what keyed information to save in Polaris and which commands to use.

#### Scenario: Short durable context exists
- **WHEN** an agent has concise task context worth preserving
- **THEN** the skill directs the agent to use `polaris remember --key <key> --text` or `polaris remember --key <key> --stdin`

#### Scenario: Existing durable key should change
- **WHEN** an agent needs to update a previously remembered fact for the same key
- **THEN** the skill directs the agent to use `polaris remember --key <key> --replace` with `--text` or `--stdin`

#### Scenario: Durable context should be removed
- **WHEN** an agent needs to remove a selected remembered fact
- **THEN** the skill directs the agent to use `polaris forget <key>` for keyed inline memory

#### Scenario: Long durable context exists
- **WHEN** an agent has longer context worth preserving
- **THEN** the skill directs the agent to create a note with `polaris note create --title` and write details to the returned path

#### Scenario: Sensitive information appears
- **WHEN** task context includes secrets, tokens, passwords, credentials, or private keys
- **THEN** the skill directs the agent not to store that information in Polaris
