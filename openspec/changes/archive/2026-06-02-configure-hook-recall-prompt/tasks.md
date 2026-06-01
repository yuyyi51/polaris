## 1. Tests

- [x] 1.1 Add CLI hook tests proving default `SessionStart` and fallback `PostToolUse` output still instructs `polaris recall` without inlining stored memory.
- [x] 1.2 Add CLI hook tests for workspace `.polaris/config.toml` using a custom `[hooks].recall_prompt` without `{{recall}}`.
- [x] 1.3 Add CLI hook tests for workspace `.polaris/config.toml` using `{{recall}}` and verifying recall output is inlined for both hook paths.
- [x] 1.4 Add CLI hook tests proving workspace config takes precedence over `~/.polaris/config.toml`, and user config is used when workspace config is absent.

## 2. Config Loading And Rendering

- [x] 2.1 Add TOML parsing support for `[hooks].recall_prompt`.
- [x] 2.2 Implement prompt config lookup in this order: active workspace `.polaris/config.toml`, then `~/.polaris/config.toml`, then built-in default.
- [x] 2.3 Implement hook prompt rendering that replaces every literal `{{recall}}` with `PolarisStore::recall()` output and otherwise leaves configured prompt text unchanged.
- [x] 2.4 Return clear path-specific errors for malformed or unreadable config files while treating missing config files as normal.

## 3. Hook Integration

- [x] 3.1 Refactor `polaris hook session-start` output generation to use the shared configured prompt renderer.
- [x] 3.2 Refactor `polaris hook post-tool-use` fallback output generation to use the same renderer after pending state is consumed.
- [x] 3.3 Preserve existing quiet conditions for non-matching events, uninitialized workspaces, empty memory, and consumed fallback state.

## 4. Documentation

- [x] 4.1 Update README hook guidance with `.polaris/config.toml` and `~/.polaris/config.toml` precedence.
- [x] 4.2 Document that `{{recall}}` is the only inline recall placeholder and that using it exposes `polaris recall` output in hook context.
- [x] 4.3 Keep bundled hook examples on safe default behavior and update wording only if needed.

## 5. Verification

- [x] 5.1 Run `cargo fmt --check`.
- [x] 5.2 Run `cargo test`.
- [x] 5.3 Run `cargo clippy`.
- [x] 5.4 Run `openspec status --change configure-hook-recall-prompt`.
