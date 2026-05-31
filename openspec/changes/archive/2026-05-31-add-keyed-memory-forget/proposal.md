## Why

Polaris memory entries are currently append-only and identified only by generated ids, which makes it awkward to maintain durable "current state" facts such as the active goal, latest plan, or current blocker. Agents need a stable, user-meaningful key for each remembered fact so they can intentionally replace or remove stale context without clearing the whole workspace.

## What Changes

- **BREAKING**: Require `polaris remember` callers to provide a non-empty `--key <key>` for inline memory records.
- Add keyed uniqueness for inline memories: `remember` refuses to create a second record with an existing key unless `--replace` is provided.
- Add `polaris forget <key>` to delete an existing memory record by key.
- Define replacement semantics so `remember --key <key> --replace` overwrites the existing record for that key instead of appending another active record.
- Update recall output and status/counting behavior as needed so forgotten or replaced records do not appear as active memory.
- Update the bundled Codex skill and README examples so documented Polaris usage matches the new required `--key` workflow.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `polaris-recall`: Inline memory recording becomes keyed, replacement is explicit, and individual records can be forgotten by key.
- `polaris-codex-skill`: Agent usage guidance and examples must use required memory keys and mention replacement and forgetting.

## Impact

- Affected code: CLI argument parsing in `src/cli.rs`, memory persistence and rewrite behavior in `src/storage.rs`, and integration tests in `tests/cli.rs`.
- Affected CLI surface: `polaris remember` gains required `--key` and optional `--replace`; `polaris forget <key>` is added.
- Affected data model: memory records need a stable key field; existing unkeyed records require a compatibility decision during recall/status and future writes.
- Affected documentation/artifacts: `README.md`, `skills/codex/polaris/SKILL.md`, and accepted OpenSpec requirements after archive.
