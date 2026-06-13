## 1. Dependency Check

- [x] 1.1 Confirm `add-memory-hygiene-commands` behavior is implemented or adjust this change to the current command surface.
- [x] 1.2 Confirm `add-memory-lifecycle-metadata` behavior is implemented or adjust suggestion rules to the available metadata.

## 2. Rename Workflow

- [x] 2.1 Add CLI tests for successful `polaris rename old.key new.key`.
- [x] 2.2 Add tests for missing source keys, existing destination keys, and unchanged memory after failed rename.
- [x] 2.3 Implement storage and CLI support for renaming keyed inline memories while preserving text and metadata.

## 3. Merge Draft Workflow

- [x] 3.1 Add tests for `polaris merge --into <target> <source>...` draft creation and non-mutating behavior.
- [x] 3.2 Add tests for missing source keys and empty target key rejection.
- [x] 3.3 Implement `.polaris/maintenance/` draft creation with deterministic target, source metadata, and source text sections.
- [x] 3.4 Document the editable draft format enough for `merge apply` to parse it reliably.

## 4. Merge Apply Workflow

- [x] 4.1 Add tests for confirmed merge apply writing or replacing the target key.
- [x] 4.2 Add tests for `--forget-sources`, unconfirmed apply rejection, and invalid draft rejection.
- [x] 4.3 Implement merge draft parsing and confirmed apply behavior as one locked storage mutation.
- [x] 4.4 Keep active memory unchanged when apply validation fails.

## 5. Suggestion Commands

- [x] 5.1 Add tests for `polaris prune --suggest` human output, JSON output, and no-suggestion output.
- [x] 5.2 Add tests for `polaris compact --suggest` human output, JSON output, and no-suggestion output.
- [x] 5.3 Implement deterministic prune suggestion rules for old volatile records, duplicate exact text, and superseded metadata.
- [x] 5.4 Implement deterministic compact suggestion rules for shared key prefixes and related lifecycle groups.
- [x] 5.5 Ensure suggest commands never mutate `.polaris/memories.jsonl`.

## 6. Documentation And Skill

- [x] 6.1 Update README with rename, merge draft/apply, prune suggest, and compact suggest workflows.
- [x] 6.2 Update the bundled Codex Polaris skill with safe maintenance workflow guidance.

## 7. Verification

- [x] 7.1 Run `openspec validate add-memory-maintenance-workflows`.
- [x] 7.2 Run `cargo fmt --check`.
- [x] 7.3 Run `cargo test`.
- [x] 7.4 Run `cargo clippy`.
