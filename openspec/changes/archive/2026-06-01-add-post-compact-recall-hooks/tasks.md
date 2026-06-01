## 1. Hook State Storage

- [x] 1.1 Add `.polaris/hook-state.json` persistence types and path helpers in storage without changing existing memory record formats.
- [x] 1.2 Add storage methods to mark compact recall pending and consume pending compact recall state.
- [x] 1.3 Ensure transient hook state remains quiet before initialization and is cleared or ignored safely when Polaris memory is cleared.

## 2. Hook Command Implementation

- [x] 2.1 Add `polaris hook post-compact` and `polaris hook post-tool-use` CLI subcommands.
- [x] 2.2 Refactor hook input parsing and recall instruction output so `session-start` and `post-tool-use` share the memory-safe reminder text.
- [x] 2.3 Implement `post-compact` event matching, `cwd` fallback, initialized-workspace checks, and pending state recording with no stdout.
- [x] 2.4 Implement `post-tool-use` event matching, pending state consumption, recallable-memory checks, and `PostToolUse` additional context output.
- [x] 2.5 Keep mismatched hook events, missing `.polaris/`, missing pending state, and no-memory fallback cases successful and quiet.

## 3. Documentation, Examples, And Skill

- [x] 3.1 Keep the direct compact `SessionStart` hook example available for tools that support it.
- [x] 3.2 Add a separate fallback hook example that configures `PostCompact` to run `polaris hook post-compact` and `PostToolUse` to run `polaris hook post-tool-use`.
- [x] 3.3 Update README command and hook guidance to explain when to use the direct example versus the fallback example and that `polaris` must be available to hook commands.
- [x] 3.4 Review and update `skills/codex/polaris/SKILL.md` so recall reminder handling stays source-agnostic and does not name the hook source.

## 4. Tests And Verification

- [x] 4.1 Add CLI tests for post-compact pending state recording and post-tool-use recall reminder output.
- [x] 4.2 Add CLI tests that post-tool-use consumes pending state and stays quiet on later invocations without a new post-compact event.
- [x] 4.3 Add CLI tests for no-memory, uninitialized workspace, and mismatched-event fallback hook behavior.
- [x] 4.4 Add tests that hook output never includes stored memory contents.
- [x] 4.5 Add or update tests that parse bundled hook examples and verify the expected hook commands.
- [x] 4.6 Run `openspec validate add-post-compact-recall-hooks --strict`.
- [x] 4.7 Run `cargo fmt --check`, `cargo test`, and `cargo clippy`.
