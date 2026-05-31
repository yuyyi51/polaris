## Why

Polaris already exposes a Codex `SessionStart` hook command, but users need a concrete repository example they can copy or adapt. Adding a project-bundled hook example makes the recall integration easier to discover and reduces configuration mistakes.

## What Changes

- Add a copyable Codex hooks example that runs `polaris hook session-start` on compact session starts.
- Document where the example lives and how users should adapt it for their Codex configuration.
- Keep the example aligned with the existing safety model: the hook should prompt `polaris recall` without inlining saved memory contents.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `polaris-recall`: Add a requirement that the repository provides a user-facing Codex hook example for compact-session recall.

## Impact

- Adds example/reference files under the repository, likely in an `examples/` path.
- Updates README guidance for configuring Codex hooks.
- May add tests or validation that the example JSON remains valid and references the supported `polaris hook session-start` command.
