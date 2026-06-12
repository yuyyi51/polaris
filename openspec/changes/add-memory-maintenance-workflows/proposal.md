## Why

After agents can list, filter, and classify memory, they still need workflows for consolidating short-lived records into durable knowledge and identifying cleanup candidates. Manual copy-edit-delete steps are slow and easy to get wrong when a workspace has many memories.

## What Changes

- Add a memory rename workflow for changing keyed inline memory keys without changing text content.
- Add a deterministic merge workflow that gathers multiple source memories into an editable draft for a target key.
- Add an explicit apply step for merge drafts that writes the target memory and optionally removes source memories after confirmation.
- Add `prune --suggest` and `compact --suggest` commands that emit cleanup suggestions without mutating memory.
- Use existing lifecycle and update metadata to make suggestions explainable and conservative.
- Keep all destructive maintenance actions explicitly confirmed.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `polaris-recall`: Adds higher-level memory maintenance workflows for renaming, merging, and suggesting cleanup.

## Impact

- Affected CLI surface: new maintenance commands such as `polaris rename`, `polaris merge`, `polaris prune --suggest`, and `polaris compact --suggest`.
- Affected implementation: storage mutation helpers, draft file handling, suggestion generation, tests, README, and the bundled Codex skill.
- Merge drafts may create temporary files under `.polaris/`; no external services or LLM dependency is required.
- This change is intended to follow `add-memory-hygiene-commands` and `add-memory-lifecycle-metadata`, because it depends on stable key inspection, filtered recall, lifecycle metadata, and update metadata.
