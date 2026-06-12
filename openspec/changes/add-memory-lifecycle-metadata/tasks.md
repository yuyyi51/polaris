## 1. Test Coverage

- [ ] 1.1 Add CLI tests for `remember --lifecycle` and `note create --lifecycle`, including invalid lifecycle rejection.
- [ ] 1.2 Add tests that legacy records without lifecycle load as durable in list, recall, and status output.
- [ ] 1.3 Add tests for `list --json --lifecycle` and `recall --lifecycle`, including composition with key and prefix filters.
- [ ] 1.4 Add tests for replacement metadata, including `updated_at`, `replacement_count`, preserved `created_at`, preserved lifecycle, and lifecycle override.
- [ ] 1.5 Add tests for `status --json` lifecycle counts and stale volatile-memory hints.

## 2. Data Model

- [ ] 2.1 Add a `MemoryLifecycle` enum with durable, state, log, and archive values.
- [ ] 2.2 Add optional lifecycle, updated_at, replacement_count, and replaced_from fields to memory records with backward-compatible deserialization.
- [ ] 2.3 Treat missing lifecycle values as durable in summary, recall, status, and filter logic.
- [ ] 2.4 Preserve original creation metadata and update replacement metadata during keyed inline replacement.

## 3. CLI And Filtering

- [ ] 3.1 Add `--lifecycle` parsing and validation to `remember` and `note create`.
- [ ] 3.2 Add lifecycle filtering to list and recall commands.
- [ ] 3.3 Ensure lifecycle filtering composes with existing exact-key, prefix, and exclude-prefix filters.
- [ ] 3.4 Extend list JSON summaries with lifecycle and replacement metadata.

## 4. Status Output

- [ ] 4.1 Add lifecycle count summaries to `polaris status --json`.
- [ ] 4.2 Add advisory stale volatile-memory hints for old state and log records.
- [ ] 4.3 Keep status behavior compatible for uninitialized workspaces and malformed JSONL diagnostics.

## 5. Documentation And Skill

- [ ] 5.1 Update README examples for lifecycle-aware remember, note, list, recall, and status usage.
- [ ] 5.2 Update the bundled Codex Polaris skill with lifecycle guidance and examples.

## 6. Verification

- [ ] 6.1 Run `openspec validate add-memory-lifecycle-metadata`.
- [ ] 6.2 Run `cargo fmt --check`.
- [ ] 6.3 Run `cargo test`.
- [ ] 6.4 Run `cargo clippy`.
