## 1. Replacement Draft Preview Tests

- [x] 1.1 Add CLI tests that `remember --replace --dry-run --text` creates a replacement draft and preview diff without mutating active memory or replacement history.
- [x] 1.2 Add CLI tests that `remember --replace --dry-run --stdin` stores stdin text under `## Replacement Memory` in the draft.
- [x] 1.3 Add CLI tests that `remember --dry-run` without `--replace` is rejected without mutating memory.
- [x] 1.4 Add CLI tests that `replace apply <draft> --yes` applies edited draft text through normal replacement metadata and history behavior.
- [x] 1.5 Add CLI tests that unconfirmed and invalid replacement draft apply commands leave memory unchanged.

## 2. Multi-Key Recall Tests

- [x] 2.1 Add CLI tests that `recall --keys goal,plan,state.branch` renders matching memories in requested key order.
- [x] 2.2 Add CLI tests that partial missing keys are reported while existing matching memories are recalled successfully.
- [x] 2.3 Add CLI tests that all-missing multi-key recall reports no matching memory and lists missing keys.
- [x] 2.4 Add CLI tests that `recall --keys` composes with lifecycle and exclude filters by reporting filtered keys as missing or filtered.
- [x] 2.5 Add CLI tests that `--keys` conflicts with `--key` and `--prefix`.

## 3. Touch Tests

- [x] 3.1 Add CLI tests that `touch --key` updates `updated_at` without changing content, lifecycle, creation time, or replacement metadata.
- [x] 3.2 Add CLI tests that `touch --keys` updates multiple explicit keyed inline memories without requiring `--yes`.
- [x] 3.3 Add CLI tests that `touch --id` updates a note record without changing note file contents.
- [x] 3.4 Add CLI tests that `touch --prefix <prefix> --yes` updates matching keyed inline memories and preserves non-matching records.
- [x] 3.5 Add CLI tests that unconfirmed prefix touch and conflicting touch selectors are rejected without mutating memory.
- [x] 3.6 Add CLI tests that `touch --keys ... --json` reports touched, missing, and skipped records.

## 4. Storage Implementation

- [x] 4.1 Implement replacement draft rendering under `.polaris/maintenance/` with target key, current metadata, proposed text, and editable `## Replacement Memory`.
- [x] 4.2 Implement replacement draft parsing and validation for `replace apply`.
- [x] 4.3 Ensure replacement draft apply reuses normal replacement semantics and writes replacement history.
- [x] 4.4 Extend recall filtering/rendering to support ordered multi-key recall and missing or filtered key reporting.
- [x] 4.5 Implement touch storage logic under the existing exclusive memory lock.
- [x] 4.6 Ensure touch preserves all non-`updated_at` fields and note file contents.

## 5. CLI Implementation

- [x] 5.1 Add `--dry-run` to `remember` and validate that it requires `--replace`.
- [x] 5.2 Add `polaris replace apply <draft> --yes`.
- [x] 5.3 Add `recall --keys <comma-separated-keys>` and selector conflict validation.
- [x] 5.4 Add `polaris touch` arguments for `--key`, `--keys`, `--id`, `--prefix`, `--yes`, and `--json`.
- [x] 5.5 Add human output for replacement draft preview, replacement apply, multi-key missing reports, and touch reports.
- [x] 5.6 Add stable JSON output for touch reports.

## 6. Documentation And Examples

- [x] 6.1 Update README command overview and maintenance sections with replace draft, multi-key recall, and touch examples.
- [x] 6.2 Update README storage notes to clarify dry-run does not mutate active memory or replacement history.
- [x] 6.3 Update `skills/codex/polaris/SKILL.md` to recommend replace draft preview before important durable replacements.
- [x] 6.4 Update `skills/codex/polaris/SKILL.md` with `recall --keys` and `touch` guidance for precise recall and state freshness.
- [x] 6.5 Add or update example materials showing the replace draft review and touch workflows.
- [x] 6.6 Ensure examples do not include secrets, credentials, tokens, or private user data.

## 7. Verification

- [x] 7.1 Run `openspec validate add-memory-review-convenience-tools`.
- [x] 7.2 Run `cargo fmt --check`.
- [x] 7.3 Run `cargo test`.
- [x] 7.4 Run `cargo clippy`.
- [x] 7.5 Run `git diff --check`.
