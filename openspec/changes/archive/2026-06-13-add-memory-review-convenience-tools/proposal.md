## Why

Recent memory edit tools make lifecycle correction and post-replace diff review safer, but common review flows still require extra round trips. Agents need to preview replacements before writing, recall several exact keys at once, and refresh still-valid state records without rewriting their contents.

## What Changes

- Add replace draft preview through `polaris remember --replace --dry-run`, producing an editable replacement draft and preview diff without mutating active memory.
- Add `polaris replace apply <draft> --yes` to apply an edited replacement draft through the normal replacement path.
- Add `polaris recall --keys <key1,key2,...>` for precise multi-key recall in input order, with missing keys reported without failing the command.
- Add `polaris touch` to update `updated_at` for selected memory records without changing text, lifecycle, creation time, or replacement metadata.
- Support touch by exact key, comma-separated keys, record id, and confirmed prefix.
- Update README, bundled Codex skill guidance, and examples so the safer preview/recall/touch workflows are discoverable.

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `polaris-recall`: Add replacement draft preview/apply behavior, multi-key recall, and memory touch behavior.

## Impact

- CLI surface: extend `remember`, add `replace apply`, add `recall --keys`, and add `touch`.
- Storage: add replace draft creation/parsing and metadata-only timestamp updates under the existing lock discipline.
- Output contracts: add stable JSON output for touch and maintain stable recall behavior for multi-key selectors.
- Tests: add end-to-end CLI coverage for dry-run drafts, apply, multi-key recall ordering/missing keys, touch selectors, confirmation rules, JSON output, and no-mutation guarantees.
- Documentation: update README, bundled Codex Polaris skill, and examples with safe replacement preview and state-refresh workflows.
