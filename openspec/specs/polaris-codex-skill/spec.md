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

#### Scenario: Compact reminder is present
- **WHEN** an agent using the skill sees a compact-session reminder to run `polaris recall`
- **THEN** the skill directs the agent to run `polaris recall` before doing other work

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
- **THEN** the skill directs the agent to ask the user before clearing it with `polaris clear --yes`

### Requirement: Memory recording guidance
The skill SHALL teach agents what information to save in Polaris and which commands to use.

#### Scenario: Short durable context exists
- **WHEN** an agent has concise task context worth preserving
- **THEN** the skill directs the agent to use `polaris remember --text` or `polaris remember --stdin`

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

