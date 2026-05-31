## 1. CLI And Data Model

- [x] 1.1 Add optional `key` field to memory records and require `--key <key>` for new `polaris remember` inline records.
- [x] 1.2 Add `--replace` to `polaris remember` and reject duplicate keys unless replacement is explicit.
- [x] 1.3 Add top-level `polaris forget <key>` command that requires initialization and errors on unknown keys.
- [x] 1.4 Implement active-record rewrite helper using a temporary file and rename for replacement and forgetting.
- [x] 1.5 Update recall output to show keys for keyed inline records while preserving legacy unkeyed record recall.

## 2. Tests

- [x] 2.1 Update existing `remember` CLI tests to pass required keys.
- [x] 2.2 Add tests for missing key rejection and duplicate key rejection without `--replace`.
- [x] 2.3 Add tests for `remember --replace` ensuring only the replacement text is recallable for that key.
- [x] 2.4 Add tests for `forget <key>` success, unknown key failure, and preservation of unrelated inline and note records.
- [x] 2.5 Add a compatibility test for recalling a legacy unkeyed inline record.

## 3. Documentation And Skill

- [x] 3.1 Update `README.md` command examples and usage text to include `--key`, `--replace`, and `forget`.
- [x] 3.2 Update `skills/codex/polaris/SKILL.md` examples and guidance to use keyed memory commands.
- [x] 3.3 Ensure skill guidance distinguishes selected `polaris forget <key>` removal from full `polaris clear --yes`.

## 4. Verification

- [x] 4.1 Run `openspec validate add-keyed-memory-forget --strict` or the project-supported equivalent.
- [x] 4.2 Run `cargo fmt --check`.
- [x] 4.3 Run `cargo test`.
- [x] 4.4 Run `cargo clippy`.
