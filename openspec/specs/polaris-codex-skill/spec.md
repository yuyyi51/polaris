# polaris-codex-skill Specification

## Purpose
TBD - created by archiving change add-polaris-codex-skill. Update Purpose after archive.
## Requirements
### Requirement: Project-bundled Codex skill
The repository SHALL provide a Codex-compatible Polaris skill under `skills/codex/polaris/`.

#### Scenario: Skill folder is copyable
- **WHEN** a user copies `skills/codex/polaris/` into a Codex skills directory
- **THEN** the copied folder contains the files needed for Codex to discover and load the Polaris skill

#### Scenario: Skill uses minimum required file set
- **WHEN** the project provides `skills/codex/polaris/`
- **THEN** the skill folder contains `SKILL.md` and MUST NOT require `agents/openai.yaml`

#### Scenario: Skill uses valid frontmatter
- **WHEN** Codex reads `skills/codex/polaris/SKILL.md`
- **THEN** the file contains valid YAML frontmatter with `name` and `description`

### Requirement: Polaris recovery instruction
The skill SHALL instruct agents to run `polaris recall` immediately when context tells them to recall Polaris memory.

#### Scenario: Recall reminder is present
- **WHEN** an agent using the skill sees current context instructing it to run `polaris recall`
- **THEN** the skill directs the agent to run `polaris recall` before doing other work

#### Scenario: Recovery instruction is source-agnostic
- **WHEN** the skill describes recall reminder handling
- **THEN** the skill MUST NOT identify the hook event or hook source that produced the recall reminder

#### Scenario: Recall fails
- **WHEN** `polaris recall` fails because Polaris is unavailable or not initialized
- **THEN** the skill directs the agent to report the failure briefly and continue with available context

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

### Requirement: Skill validation
The project SHALL include validation steps for the bundled Polaris skill.

#### Scenario: Skill implementation is complete
- **WHEN** the Polaris skill files are created or updated
- **THEN** the implementation is validated with the skill-creator quick validation script when available

#### Scenario: Project checks run
- **WHEN** the Polaris skill change is completed
- **THEN** existing project verification commands still pass

