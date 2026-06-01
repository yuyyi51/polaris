## MODIFIED Requirements

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
