## Why

Lifecycle metadata now separates long-term rules from current state, but agents still need a low-friction way to reclassify existing memories after the fact. Replacement metadata also shows that a key changed, but it does not yet let agents review what changed before trusting or further editing long-lived memory.

## What Changes

- Add a lifecycle move command for explicitly reclassifying existing memory records without rewriting their text.
- Support lifecycle moves for both keyed inline memory and note records, with stable selectors for each record type.
- Require confirmation for batch lifecycle moves and provide JSON output for agent review.
- Store the previous active inline memory record when `remember --replace` supersedes a key.
- Add `polaris diff --key <key>` to compare the current active inline memory with its latest replacement snapshot.
- Provide human-readable unified diff output and stable JSON diff output.
- Update README, bundled Codex skill guidance, and examples as needed so agents know how to use lifecycle moves and replacement diffs safely.

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `polaris-recall`: Add memory lifecycle move behavior, replacement snapshot persistence, and replacement diff review commands.

## Impact

- CLI surface: add lifecycle move and diff commands.
- Storage: add locked metadata rewrite behavior for lifecycle moves and a replacement history JSONL file for previous inline records.
- Output contracts: add stable JSON output for lifecycle move and diff.
- Tests: add end-to-end CLI coverage for inline and note lifecycle moves, replacement history, human diff output, JSON diff output, and legacy/no-history behavior.
- Documentation: update README, bundled Codex Polaris skill, and example materials where the new review workflow should be discoverable.
