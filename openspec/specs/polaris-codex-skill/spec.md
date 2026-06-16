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

### Requirement: Script-based skill installation
The repository SHALL provide a script that installs the bundled Polaris Codex skill into a Codex skills directory.

#### Scenario: Install into Codex home from environment
- **WHEN** a user runs the skill install script without a skills directory override
- **AND** `CODEX_HOME` is set
- **THEN** the script installs `skills/codex/polaris/` into `$CODEX_HOME/skills/polaris`

#### Scenario: Install into existing Codex app home
- **WHEN** a user runs the skill install script without a skills directory override
- **AND** `CODEX_HOME` is unset
- **AND** `$HOME/.codex-app` exists
- **THEN** the script installs `skills/codex/polaris/` into `$HOME/.codex-app/skills/polaris`

#### Scenario: Install into classic Codex home fallback
- **WHEN** a user runs the skill install script without a skills directory override
- **AND** `CODEX_HOME` is unset
- **AND** `$HOME/.codex-app` does not exist
- **THEN** the script installs `skills/codex/polaris/` into `$HOME/.codex/skills/polaris`

#### Scenario: Install into an overridden skills directory
- **WHEN** a user runs the skill install script with an explicit skills directory override
- **THEN** the script installs `skills/codex/polaris/` into `<override>/polaris`

#### Scenario: Reinstall bundled skill
- **WHEN** the destination `polaris` skill directory already exists
- **THEN** the script replaces that destination directory with the repository's bundled skill directory

#### Scenario: Preserve script scope
- **WHEN** a user runs the skill install script
- **THEN** the script MUST NOT modify Codex hook configuration
- **AND** the script MUST NOT initialize Polaris workspace memory

### Requirement: Polaris citation guidance
The skill SHALL teach agents how to use Polaris citation signals deliberately without confusing citation with recall or freshness.

#### Scenario: Skill distinguishes recall touch and cite
- **WHEN** an agent reads the Polaris skill memory guidance
- **THEN** the skill explains that `polaris recall` shows stored context
- **AND** the skill explains that `polaris touch` marks records as still fresh
- **AND** the skill explains that `polaris cite` marks records the agent explicitly used as evidence

#### Scenario: Skill discourages citing every recalled memory
- **WHEN** the skill describes citation workflow
- **THEN** the skill directs agents not to cite every record that appeared in recall output
- **AND** the skill directs agents to cite only records that actually affected reasoning, avoided repeated investigation, corrected an assumption, or shaped a decision

#### Scenario: Skill teaches batched citation
- **WHEN** the skill provides citation examples
- **THEN** the skill includes batched citation examples using `polaris cite --keys <keys> --quiet` or `polaris cite --ids <ids> --quiet`
- **AND** the skill explains that batching citations with nearby Polaris or shell work can reduce extra tool calls

#### Scenario: Skill keeps citation advisory
- **WHEN** the skill describes memory maintenance
- **THEN** the skill treats citation counts and memory creation time as review context for the agent
- **AND** the skill MUST NOT direct agents to forget, keep, or merge memories solely because of citation counts
