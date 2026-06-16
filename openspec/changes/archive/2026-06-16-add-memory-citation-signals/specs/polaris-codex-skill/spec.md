## ADDED Requirements

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
