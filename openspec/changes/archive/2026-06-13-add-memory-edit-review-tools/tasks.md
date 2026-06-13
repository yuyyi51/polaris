## 1. Lifecycle Move Tests

- [x] 1.1 Add CLI tests for moving a keyed inline memory lifecycle by `--key`.
- [x] 1.2 Add CLI tests for moving keyed inline memory lifecycle by `--prefix` with `--from`, `--to`, and `--yes`.
- [x] 1.3 Add CLI tests for moving a note memory lifecycle by `--id`.
- [x] 1.4 Add CLI tests for moving memory lifecycle by `--kind note` and `--kind inline`.
- [x] 1.5 Add CLI tests that batch lifecycle moves without `--yes` are rejected and leave memory unchanged.
- [x] 1.6 Add CLI tests that conflicting lifecycle move selectors are rejected.
- [x] 1.7 Add CLI tests for `polaris lifecycle move --json` output including moved and skipped records.

## 2. Replacement Diff Tests

- [x] 2.1 Add CLI tests that `remember --replace` saves the previous active inline memory as replacement history.
- [x] 2.2 Add CLI tests that `polaris diff --key <key>` prints a unified diff for the latest replacement.
- [x] 2.3 Add CLI tests that `polaris diff --key <key> --json` prints stable machine-readable diff data.
- [x] 2.4 Add CLI tests that `polaris diff --key <key>` reports no snapshot for legacy or never-replaced records.
- [x] 2.5 Add CLI tests that diff rejects missing keys and uninitialized workspaces with actionable errors.

## 3. Storage Implementation

- [x] 3.1 Add replacement history JSONL path management under `.polaris/`.
- [x] 3.2 Persist the previous active inline memory record to replacement history during `remember --replace`.
- [x] 3.3 Read replacement history safely with malformed-record diagnostics consistent with memory JSONL handling.
- [x] 3.4 Implement lifecycle move storage logic under the existing exclusive memory lock.
- [x] 3.5 Ensure lifecycle moves preserve memory text, note paths, note file contents, `created_at`, `replacement_count`, and `replaced_from`.
- [x] 3.6 Ensure records already at the target lifecycle are reported as skipped rather than rewritten.

## 4. CLI Implementation

- [x] 4.1 Add `polaris lifecycle move` command arguments for `--key`, `--prefix`, `--id`, `--kind`, `--from`, `--to`, `--yes`, and `--json`.
- [x] 4.2 Validate lifecycle move selector conflicts and confirmation rules before mutating storage.
- [x] 4.3 Add human lifecycle move output that reports moved and skipped records concisely.
- [x] 4.4 Add `polaris diff --key <key>` and `--json` command arguments.
- [x] 4.5 Render human diff output as a unified diff.
- [x] 4.6 Render JSON diff output with key, current record id, previous record id, and diff data.

## 5. Documentation And Examples

- [x] 5.1 Update README command overview and memory maintenance sections with lifecycle move and diff examples.
- [x] 5.2 Update README storage/config notes to mention replacement history behavior where relevant.
- [x] 5.3 Update `skills/codex/polaris/SKILL.md` to recommend lifecycle moves for metadata-only reclassification and `diff --key` before trusting replacements.
- [x] 5.4 Add or update example materials showing safe lifecycle migration and replacement diff review.
- [x] 5.5 Ensure examples do not include secrets, credentials, tokens, or private user data.

## 6. Verification

- [x] 6.1 Run `openspec validate add-memory-edit-review-tools`.
- [x] 6.2 Run `cargo fmt --check`.
- [x] 6.3 Run `cargo test`.
- [x] 6.4 Run `cargo clippy`.
- [x] 6.5 Run `git diff --check`.
