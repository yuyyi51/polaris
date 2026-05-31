## 1. Example Configuration

- [x] 1.1 Add `examples/codex-hooks/hooks.json` with a `SessionStart` compact matcher that runs `polaris hook session-start`.
- [x] 1.2 Keep the example minimal and copyable, using only supported command hook fields.

## 2. Documentation

- [x] 2.1 Update README hook guidance to point to the bundled example.
- [x] 2.2 Document that `polaris` must be available to the hook command, and users can adapt the command path if needed.
- [x] 2.3 Reiterate that the hook prompts recall and does not inline stored memory.

## 3. Validation

- [x] 3.1 Add or update tests that parse the example JSON and verify it references `polaris hook session-start` under the compact `SessionStart` hook.
- [x] 3.2 Run `openspec validate add-codex-hook-example --strict`.
- [x] 3.3 Run `cargo fmt --check`.
- [x] 3.4 Run `cargo test`.
- [x] 3.5 Run `cargo clippy`.
