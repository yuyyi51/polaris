## ADDED Requirements

### Requirement: Hook target selection
The system SHALL allow each Polaris hook command to select between Codex (default) and TraeCLI host adapters via a `--target <host>` argument.

#### Scenario: Default target stays Codex
- **WHEN** `polaris hook session-start`, `polaris hook post-compact`, or `polaris hook post-tool-use` is invoked without `--target`
- **THEN** the system uses the Codex host adapter
- **AND** stdin parsing and stdout JSON match the existing Codex behavior byte-for-byte

#### Scenario: Explicit Codex target
- **WHEN** any `polaris hook` command is invoked with `--target codex`
- **THEN** the system uses the Codex host adapter (identical to default)

#### Scenario: TraeCLI target accepted
- **WHEN** any `polaris hook` command is invoked with `--target traecli`
- **THEN** the system uses the TraeCLI host adapter

#### Scenario: Reject unknown target
- **WHEN** any `polaris hook` command is invoked with `--target` set to a value other than `codex` or `traecli`
- **THEN** the system exits with a non-zero status and an error message naming the invalid value
- **AND** no recall output is emitted

### Requirement: TraeCLI post-compact recall hook
The system SHALL emit Polaris recall context directly on TraeCLI's `post_compact` event without writing or consuming any pending state file.

#### Scenario: Prompt after TraeCLI post compact
- **WHEN** `polaris hook post-compact --target traecli` receives hook JSON with `hook_event_name: "PostCompact"` in an initialized workspace with stored memory
- **THEN** the system prints JSON containing `hookSpecificOutput.additionalContext` rendered from the selected hook recall prompt
- **AND** the JSON output MUST NOT contain a `hookEventName` field
- **AND** the system MUST NOT write a pending compact-recall state file

#### Scenario: Default TraeCLI prompt does not inline memory
- **WHEN** `polaris hook post-compact --target traecli` emits hook JSON without a configured hook recall prompt
- **THEN** the `additionalContext` value contains only the built-in recall instruction
- **AND** the output MUST NOT contain stored memory contents

#### Scenario: Configured TraeCLI prompt can inline recall
- **WHEN** `polaris hook post-compact --target traecli` emits hook JSON and the selected hook recall prompt contains `{{recall}}`
- **THEN** the `additionalContext` value contains the current `polaris recall` output

#### Scenario: TraeCLI post-compact stays quiet without memory
- **WHEN** `polaris hook post-compact --target traecli` runs in an initialized workspace with no stored memory
- **THEN** the system exits successfully without stdout

#### Scenario: TraeCLI post-compact stays quiet before initialization
- **WHEN** `polaris hook post-compact --target traecli` runs in a workspace without `.polaris/`
- **THEN** the system exits successfully without stdout

#### Scenario: TraeCLI post-compact ignores unrelated events
- **WHEN** `polaris hook post-compact --target traecli` receives hook JSON whose `hook_event_name` is not `PostCompact`
- **THEN** the system exits successfully without stdout

### Requirement: TraeCLI session-start and post-tool-use stay quiet
The system SHALL accept `--target traecli` on `polaris hook session-start` and `polaris hook post-tool-use` and treat them as no-ops, leaving room for future TraeCLI events without changing CLI surface.

#### Scenario: TraeCLI session-start is a noop
- **WHEN** `polaris hook session-start --target traecli` runs with any stdin in any workspace state
- **THEN** the system exits successfully without stdout
- **AND** the system MUST NOT write or consume any pending compact-recall state file

#### Scenario: TraeCLI post-tool-use is a noop
- **WHEN** `polaris hook post-tool-use --target traecli` runs with any stdin in any workspace state
- **THEN** the system exits successfully without stdout
- **AND** the system MUST NOT write or consume any pending compact-recall state file

### Requirement: TraeCLI hook example
The repository SHALL provide a copyable TraeCLI hook example that configures Polaris recall on the TraeCLI `post_compact` event.

#### Scenario: Example configures TraeCLI post-compact recall
- **WHEN** a user opens the bundled TraeCLI hook example
- **THEN** the example configures a `PostCompact` hook that runs `polaris hook post-compact --target traecli`

#### Scenario: TraeCLI example documented in README
- **WHEN** a user reads the Polaris README hook guidance
- **THEN** the README references the bundled TraeCLI example
- **AND** the README documents the `--target` argument and its `codex` (default) / `traecli` values

#### Scenario: TraeCLI example preserves hook memory safety
- **WHEN** a user follows the bundled TraeCLI hook example
- **THEN** the configured hook command prompts recall without inlining stored memory contents
