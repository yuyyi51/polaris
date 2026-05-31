## ADDED Requirements

### Requirement: Codex hook example
The repository SHALL provide a copyable Codex hooks example that configures Polaris recall for compact session starts.

#### Scenario: Example configures compact session recall
- **WHEN** a user opens the bundled Codex hooks example
- **THEN** the example configures a `SessionStart` hook with matcher `compact` that runs `polaris hook session-start`

#### Scenario: Example is documented
- **WHEN** a user reads the Polaris README hook guidance
- **THEN** the README points to the bundled Codex hooks example and explains that `polaris` must be available to the hook command

#### Scenario: Example preserves hook memory safety
- **WHEN** a user follows the bundled Codex hooks example
- **THEN** the configured hook uses the existing Polaris hook command that prompts recall without inlining stored memory contents
