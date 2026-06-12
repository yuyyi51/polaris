## 1. Test Coverage

- [x] 1.1 Add CLI tests for `polaris list --keys` and `polaris list --json`, including note records and omission of inline text bodies.
- [x] 1.2 Add CLI tests for `polaris recall --key`, `--prefix`, repeatable `--exclude`, selector conflicts, and no-match output.
- [x] 1.3 Add CLI tests for multi-key `polaris forget`, unknown-key all-or-nothing behavior, and missing workspace behavior.
- [x] 1.4 Add CLI tests for `polaris forget --prefix <prefix> --yes`, unconfirmed prefix deletion, empty prefixes, no matches, and selector conflicts.

## 2. Storage And Filtering

- [x] 2.1 Introduce memory summary data for list output without including inline memory text.
- [x] 2.2 Add shared key-filtering logic for exact key, prefix, and excluded prefixes.
- [x] 2.3 Add storage methods for listing memory summaries and rendering filtered recall output.
- [x] 2.4 Add batch and prefix forget storage methods that mutate memory in one locked load/filter/write cycle.

## 3. CLI Commands

- [x] 3.1 Add `polaris list --keys` and `polaris list --json` argument parsing and output.
- [x] 3.2 Add recall filter flags and validation for conflicting exact-key and prefix selectors.
- [x] 3.3 Extend forget argument parsing for variadic keys, `--prefix`, and `--yes`.
- [x] 3.4 Print concise success and error messages for batch forget, prefix forget, and filtered recall no-match cases.

## 4. Documentation And Skill

- [x] 4.1 Update README command examples and memory hygiene guidance.
- [x] 4.2 Update the bundled Codex Polaris skill to prefer list/filter commands over scraping recall output.

## 5. Verification

- [x] 5.1 Run `openspec validate add-memory-hygiene-commands`.
- [x] 5.2 Run `cargo fmt --check`.
- [x] 5.3 Run `cargo test`.
- [x] 5.4 Run `cargo clippy`.
