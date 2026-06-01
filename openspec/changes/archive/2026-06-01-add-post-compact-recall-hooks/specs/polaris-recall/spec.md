## ADDED Requirements

### Requirement: Post-compact fallback hook prompt
The system SHALL provide fallback hook commands that record completion of conversation compaction and prompt the agent to run `polaris recall` on the next post-tool-use hook when recallable memory exists.

#### Scenario: Record pending recall after post compact
- **WHEN** `polaris hook post-compact` receives hook JSON with `hook_event_name: "PostCompact"` in an initialized workspace
- **THEN** the system records pending compact-recall state under `.polaris/`
- **AND** the command exits successfully without stdout

#### Scenario: Prompt after pending post tool use
- **WHEN** `polaris hook post-tool-use` receives hook JSON with `hook_event_name: "PostToolUse"` in an initialized workspace with stored memory and pending compact-recall state
- **THEN** the system prints JSON with `hookSpecificOutput.hookEventName: "PostToolUse"` and `hookSpecificOutput.additionalContext` instructing the agent to run `polaris recall`

#### Scenario: Consume pending recall state
- **WHEN** `polaris hook post-tool-use` checks a pending compact-recall state
- **THEN** the system consumes the pending state
- **AND** a later `polaris hook post-tool-use` invocation without a new `polaris hook post-compact` invocation exits successfully without stdout

#### Scenario: Do not inline memory in fallback hook output
- **WHEN** `polaris hook post-tool-use` emits fallback hook JSON
- **THEN** the `additionalContext` value contains only the recall instruction and MUST NOT contain stored memory contents

#### Scenario: Stay quiet when fallback has no memory
- **WHEN** `polaris hook post-tool-use` receives hook JSON with `hook_event_name: "PostToolUse"` in an initialized workspace with pending compact-recall state and no stored memory
- **THEN** the system consumes the pending state
- **AND** the command exits successfully without stdout

#### Scenario: Ignore unrelated fallback events
- **WHEN** `polaris hook post-compact` or `polaris hook post-tool-use` receives hook JSON whose `hook_event_name` does not match that command's expected event
- **THEN** the command exits successfully without stdout

#### Scenario: Stay quiet before initialization
- **WHEN** `polaris hook post-compact` or `polaris hook post-tool-use` runs in a workspace without `.polaris/`
- **THEN** the command exits successfully without stdout

## MODIFIED Requirements

### Requirement: Codex hook example
The repository SHALL provide copyable hook examples that configure Polaris recall for compact-session recovery.

#### Scenario: Direct example configures compact session recall
- **WHEN** a user opens the bundled direct hook example
- **THEN** the example configures a `SessionStart` hook with matcher `compact` that runs `polaris hook session-start`

#### Scenario: Fallback example configures post-compact recall
- **WHEN** a user opens the bundled fallback hook example
- **THEN** the example configures a `PostCompact` hook that runs `polaris hook post-compact`
- **AND** the example configures a `PostToolUse` hook that runs `polaris hook post-tool-use`

#### Scenario: Examples are documented
- **WHEN** a user reads the Polaris README hook guidance
- **THEN** the README points to the bundled direct and fallback hook examples
- **AND** the README explains that `polaris` must be available to the hook command
- **AND** the README explains when to use the fallback example instead of the direct compact `SessionStart` example

#### Scenario: Examples preserve hook memory safety
- **WHEN** a user follows a bundled hook example
- **THEN** the configured hook commands prompt recall without inlining stored memory contents
