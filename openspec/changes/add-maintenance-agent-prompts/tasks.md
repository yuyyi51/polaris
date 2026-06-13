## 1. Test Coverage

- [x] 1.1 Add CLI tests that `prune --suggest` and `compact --suggest` human output includes default agent prompts.
- [x] 1.2 Add CLI tests that `prune --suggest --json` and `compact --suggest --json` return an envelope with `suggestions`, `prompt`, and `prompt_context`.
- [x] 1.3 Add CLI tests for workspace `.polaris/config.toml` maintenance prompt overrides.
- [x] 1.4 Add CLI tests for user `~/.polaris/config.toml` maintenance prompt fallback when workspace config is absent.
- [x] 1.5 Add tests that missing prompt keys fall back to built-in defaults and unsupported placeholders remain harmless literal text.
- [x] 1.6 Add tests that rendered maintenance prompts include suggestions, status, and known keys but do not inline memory text bodies.

## 2. Configuration And Prompt Rendering

- [x] 2.1 Extend TOML config parsing with optional `[maintenance.prompts].prune` and `[maintenance.prompts].compact` fields.
- [x] 2.2 Reuse workspace-over-user config precedence while allowing missing maintenance prompt fields to fall back to built-in defaults.
- [x] 2.3 Add conservative built-in prune and compact prompt templates.
- [x] 2.4 Implement placeholder rendering for `{{suggestions}}`, `{{status}}`, and `{{keys}}`.
- [x] 2.5 Ensure rendered maintenance prompts never inline full `polaris recall` output by default.

## 3. Suggestion Output

- [x] 3.1 Add a shared suggestion response envelope containing `suggestions`, `prompt`, and `prompt_context`.
- [x] 3.2 Change prune JSON output from a bare array to the response envelope.
- [x] 3.3 Change compact JSON output from a bare array to the response envelope.
- [x] 3.4 Append an `Agent prompt:` section to human prune output, including no-suggestion output.
- [x] 3.5 Append an `Agent prompt:` section to human compact output, including no-suggestion output.
- [x] 3.6 Keep suggest commands read-only and preserve `.polaris/memories.jsonl`.

## 4. Documentation And Skill

- [x] 4.1 Update README with default maintenance prompt behavior, JSON envelope shape, placeholders, and config examples.
- [x] 4.2 Update the bundled Codex Polaris skill to recommend using the generated prompt when reviewing maintenance suggestions.

## 5. Verification

- [x] 5.1 Run `openspec validate add-maintenance-agent-prompts`.
- [x] 5.2 Run `cargo fmt --check`.
- [x] 5.3 Run `cargo test`.
- [x] 5.4 Run `cargo clippy`.
