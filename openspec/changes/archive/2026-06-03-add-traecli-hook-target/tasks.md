## 1. CLI surface

- [x] 1.1 Define `enum HookTarget { Codex, Traecli }` with `clap::ValueEnum` + `Default = Codex` (in `src/hook.rs` or `src/cli.rs`)
- [x] 1.2 Add `--target <HookTarget>` argument to each of `HookCommand::SessionStart`, `HookCommand::PostCompact`, `HookCommand::PostToolUse` in `src/cli.rs`

## 2. Hook adapter layer

- [x] 2.1 Refactor `src/hook.rs` so that `recall_text` rendering (recall instruction, `{{recall}}` substitution, `[hooks].recall_prompt` selection, "do not inline memory" guard) is host-agnostic
- [x] 2.2 Split `HookOutput` into Codex-shaped (with `hookEventName`) and TraeCLI-shaped (without `hookEventName`) variants; share `additionalContext` payload between them
- [x] 2.3 Update `session_start_output` to take a `HookTarget`; for `Codex`, keep current behavior; for `Traecli`, return `Ok(None)` (noop)
- [x] 2.4 Update `post_compact` to take a `HookTarget`; for `Codex`, keep current pending-state behavior; for `Traecli`, parse stdin (`hook_event_name == "PostCompact"`), validate workspace + memory, return TraeCLI-shaped `HookOutput` directly without writing pending state
- [x] 2.5 Update `post_tool_use_output` to take a `HookTarget`; for `Codex`, keep current behavior; for `Traecli`, return `Ok(None)` (noop)
- [x] 2.6 Wire CLI dispatch in `src/cli.rs` to pass `args.target` into the hook functions and serialize the correct output variant for each target

## 3. Tests

- [x] 3.1 Add unit tests in `src/hook.rs` covering: TraeCLI `post_compact` happy path emits `hookSpecificOutput.additionalContext` without `hookEventName`
- [x] 3.2 Add unit tests covering TraeCLI `post_compact` noop cases: uninitialized workspace, no memory, mismatched `hook_event_name`
- [x] 3.3 Add unit tests covering TraeCLI `session_start` and `post_tool_use` noops (no stdout, no pending state file mutation)
- [x] 3.4 Add unit tests asserting Codex default target (no `--target` flag) preserves existing byte-level output
- [x] 3.5 Add CLI end-to-end tests in `tests/cli.rs` for `polaris hook post-compact --target traecli` against an initialized fixture workspace
- [x] 3.6 Add CLI test asserting `polaris hook post-compact --target bogus` exits non-zero with an error mentioning the invalid value

## 4. Examples and documentation

- [x] 4.1 Add `examples/traecli-hooks/hooks.json` registering `PostCompact` to run `polaris hook post-compact --target traecli`
- [x] 4.2 Update `README.md` with a "TraeCLI Integration" section: link to the new example, document the `--target` argument values, point out that TraeCLI uses `post_compact` (not `SessionStart`+`compact`), reaffirm that hooks do not inline memory unless `{{recall}}` is configured

## 5. Verification

- [x] 5.1 Run `cargo fmt --check`
- [x] 5.2 Run `cargo test`
- [x] 5.3 Run `cargo clippy`
- [x] 5.4 Run `openspec validate add-traecli-hook-target --strict` (or equivalent project command) and report any failures
