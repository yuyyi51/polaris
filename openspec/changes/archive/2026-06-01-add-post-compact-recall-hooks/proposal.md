## Why

Some CLI tools that support hooks do not fire a compact-session `SessionStart` hook immediately after conversation compaction. Polaris needs a fallback hook flow that can notice compaction completion and remind the agent to run `polaris recall` on the next available model-visible hook.

## What Changes

- Add a `polaris hook post-compact` command that records a workspace-local pending compact-recall state after a `PostCompact` hook fires.
- Add a `polaris hook post-tool-use` command that checks for the pending compact-recall state and, when recallable memory exists, emits the same recall instruction used by the compact `SessionStart` hook.
- Consume the pending compact-recall state after `post-tool-use` checks it so the fallback reminder is one-shot.
- Keep all hook outputs quiet before `polaris init`, when no recallable memory exists, or when hook input does not match the expected event.
- Update the hook examples and README to document both the existing compact `SessionStart` integration and the `PostCompact` plus `PostToolUse` fallback.
- Keep the Polaris skill source-agnostic: it should tell agents what to do when current context instructs recall, without naming which hook produced that reminder.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `polaris-recall`: Add fallback hook commands for tools that require `PostCompact` state tracking and `PostToolUse` reminder injection.
- `polaris-codex-skill`: Clarify recovery guidance so the skill remains independent of the specific hook source that produced a recall reminder.

## Impact

- Affected code: Rust CLI command surface in `src/cli.rs`, hook behavior in `src/hook.rs`, and workspace persistence in `src/storage.rs`.
- Affected runtime data: a new `.polaris/` hook state file for pending compact-recall state.
- Affected external integration: user hook configurations can call `polaris hook post-compact` and `polaris hook post-tool-use` in addition to the existing `polaris hook session-start`.
- Affected documentation/artifacts: `README.md`, bundled hook examples under `examples/`, bundled Polaris skill guidance, and tests.
- Dependencies: no new external dependencies are expected.
