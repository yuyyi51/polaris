## Why

Long-running agent projects accumulate enough Polaris memories that full recall and one-key-at-a-time cleanup become inefficient. Agents need stable, machine-readable ways to inspect, filter, and remove selected memories without scraping human recall output or clearing the whole workspace.

## What Changes

- Add a stable `polaris list` command for inspecting stored memory records.
- Add key and prefix filters to `polaris recall` so agents can recover only task-relevant context.
- Extend `polaris forget` to remove multiple keyed inline memories in one command.
- Add explicit prefix deletion for keyed inline memories, guarded by confirmation.
- Preserve existing recall, status, remember, note, hook, and clear behavior when the new options are not used.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `polaris-recall`: Adds first-class memory hygiene commands and filtered recall behavior for existing workspace memory.

## Impact

- Affected CLI surface: `polaris list`, `polaris recall`, and `polaris forget`.
- Affected implementation: `src/cli.rs`, `src/storage.rs`, `tests/cli.rs`, README command documentation, and the bundled Codex skill guidance.
- No storage format migration is required; the new commands operate on existing `.polaris/memories.jsonl` records.
- No new external dependencies are expected.
