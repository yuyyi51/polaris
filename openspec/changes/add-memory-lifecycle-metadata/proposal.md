## Why

Long-running projects mix stable knowledge, current workspace state, short work logs, and archived history in the same Polaris memory stream. Agents need lightweight metadata to distinguish durable context from volatile context and to understand when a memory was last updated.

## What Changes

- Add a lifecycle classification for memory records, such as `durable`, `state`, `log`, and `archive`.
- Let `polaris remember` set lifecycle metadata for new and replacement inline memories.
- Preserve lifecycle metadata in list summaries and allow list/recall filtering by lifecycle.
- Track lightweight update metadata for replacements, including `updated_at` and `replacement_count`.
- Surface lifecycle counts and stale-state hints in machine-readable status output.
- Preserve compatibility with existing memory records by treating records without lifecycle metadata as durable unless otherwise specified.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `polaris-recall`: Adds memory lifecycle and update metadata to existing workspace memory behavior.

## Impact

- Affected CLI surface: `polaris remember`, `polaris list`, `polaris recall`, and `polaris status --json`.
- Affected implementation: memory record serialization/deserialization, replacement logic, list/recall filters, status summaries, tests, README, and the bundled Codex skill.
- Storage remains JSONL, but records gain optional metadata fields. Existing records must continue to load without migration.
- This change is intended to follow `add-memory-hygiene-commands`, because it builds on stable list and filtered recall behavior.
