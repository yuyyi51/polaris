## ADDED Requirements

### Requirement: Hook recall prompt configuration
The system SHALL allow Codex hook recall prompts to be configured by TOML files while preserving the built-in prompt when no configuration exists.

#### Scenario: Use workspace hook prompt before user hook prompt
- **WHEN** `.polaris/config.toml` in the active workspace and `~/.polaris/config.toml` both define `[hooks].recall_prompt`
- **THEN** hook output uses the workspace `recall_prompt`

#### Scenario: Use user hook prompt when workspace config is absent
- **WHEN** `.polaris/config.toml` is absent and `~/.polaris/config.toml` defines `[hooks].recall_prompt`
- **THEN** hook output uses the user `recall_prompt`

#### Scenario: Use built-in prompt when no hook config exists
- **WHEN** neither `.polaris/config.toml` nor `~/.polaris/config.toml` exists
- **THEN** hook output uses the built-in recall instruction
- **AND** the output MUST NOT contain stored memory contents

#### Scenario: Inline recall with placeholder
- **WHEN** the selected `recall_prompt` contains `{{recall}}`
- **THEN** hook output replaces each `{{recall}}` placeholder with the current `polaris recall` output

#### Scenario: Configured prompt without placeholder does not inline recall
- **WHEN** the selected `recall_prompt` does not contain `{{recall}}`
- **THEN** hook output uses the configured prompt text
- **AND** the output MUST NOT contain stored memory contents unless they are present in the configured prompt itself

## MODIFIED Requirements

### Requirement: Compact session hook prompt
The system SHALL provide a Codex `SessionStart` hook command that adds recall context after compaction.

#### Scenario: Prompt after compact session start
- **WHEN** `polaris hook session-start` receives Codex hook JSON with `hook_event_name: "SessionStart"` and `source: "compact"` in an initialized workspace with stored memory
- **THEN** the system prints JSON with `hookSpecificOutput.hookEventName: "SessionStart"` and `hookSpecificOutput.additionalContext` rendered from the selected hook recall prompt

#### Scenario: Default compact prompt does not inline memory
- **WHEN** `polaris hook session-start` emits compact-session hook JSON without a configured hook recall prompt
- **THEN** the `additionalContext` value contains only the built-in recall instruction and MUST NOT contain stored memory contents

#### Scenario: Configured compact prompt can inline recall
- **WHEN** `polaris hook session-start` emits compact-session hook JSON and the selected hook recall prompt contains `{{recall}}`
- **THEN** the `additionalContext` value contains the current `polaris recall` output

#### Scenario: Ignore non-compact session starts
- **WHEN** `polaris hook session-start` receives Codex hook JSON with `source` other than `compact`
- **THEN** the system exits successfully without stdout

#### Scenario: Stay quiet before initialization
- **WHEN** `polaris hook session-start` runs in a workspace without `.polaris/`
- **THEN** the system exits successfully without stdout

### Requirement: Post-compact fallback hook prompt
The system SHALL provide fallback hook commands that record completion of conversation compaction and add recall context on the next post-tool-use hook when recallable memory exists.

#### Scenario: Record pending recall after post compact
- **WHEN** `polaris hook post-compact` receives hook JSON with `hook_event_name: "PostCompact"` in an initialized workspace
- **THEN** the system records pending compact-recall state under `.polaris/`
- **AND** the command exits successfully without stdout

#### Scenario: Prompt after pending post tool use
- **WHEN** `polaris hook post-tool-use` receives hook JSON with `hook_event_name: "PostToolUse"` in an initialized workspace with stored memory and pending compact-recall state
- **THEN** the system prints JSON with `hookSpecificOutput.hookEventName: "PostToolUse"` and `hookSpecificOutput.additionalContext` rendered from the selected hook recall prompt

#### Scenario: Consume pending recall state
- **WHEN** `polaris hook post-tool-use` checks a pending compact-recall state
- **THEN** the system consumes the pending state
- **AND** a later `polaris hook post-tool-use` invocation without a new `polaris hook post-compact` invocation exits successfully without stdout

#### Scenario: Default fallback prompt does not inline memory
- **WHEN** `polaris hook post-tool-use` emits fallback hook JSON without a configured hook recall prompt
- **THEN** the `additionalContext` value contains only the built-in recall instruction and MUST NOT contain stored memory contents

#### Scenario: Configured fallback prompt can inline recall
- **WHEN** `polaris hook post-tool-use` emits fallback hook JSON and the selected hook recall prompt contains `{{recall}}`
- **THEN** the `additionalContext` value contains the current `polaris recall` output

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
