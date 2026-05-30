## 1. Project Setup

- [x] 1.1 Add Rust dependencies for CLI parsing, JSON serialization, timestamps, and temporary-directory tests.
- [x] 1.2 Replace the hello-world entry point with a CLI command router matching the MVP command surface.
- [x] 1.3 Create module boundaries for workspace paths, storage, command handlers, and hook handling.

## 2. Workspace Storage

- [x] 2.1 Implement workspace resolution from the current directory for normal commands and hook `cwd` for hook commands.
- [x] 2.2 Implement `polaris init` to create `.polaris/state.json`, `.polaris/memories.jsonl`, and `.polaris/docs/` idempotently.
- [x] 2.3 Implement JSONL memory record types for inline text entries and long note pointers.
- [x] 2.4 Implement storage helpers to load, append, count, and clear memory records and note files.

## 3. Agent Commands

- [x] 3.1 Implement `polaris status --json`, including `initialized: false` behavior before initialization.
- [x] 3.2 Implement `polaris remember --text <text> [--title <title>]`.
- [x] 3.3 Implement `polaris remember --stdin [--title <title>]`.
- [x] 3.4 Implement `polaris note create --title <title>` to create a markdown file and record a pointer entry.
- [x] 3.5 Implement `polaris recall` with model-readable output for inline memories and long note paths.
- [x] 3.6 Implement `polaris clear --yes` and refusal behavior when `--yes` is omitted.

## 4. Codex Hook Command

- [x] 4.1 Implement hook input parsing for `polaris hook session-start`.
- [x] 4.2 Make the hook emit no output when `.polaris/` is missing, the source is not `compact`, or no recallable memory exists.
- [x] 4.3 Make compact `SessionStart` with stored memory emit `hookSpecificOutput.additionalContext` instructing the agent to run `polaris recall`.
- [x] 4.4 Add a sample hook configuration snippet to project documentation or CLI help text.

## 5. Verification

- [x] 5.1 Add tests for initialization, idempotent initialization, and missing-workspace behavior.
- [x] 5.2 Add tests for status, remember, note creation, recall, and clear.
- [x] 5.3 Add tests for compact and non-compact `SessionStart` hook inputs.
- [x] 5.4 Run `cargo fmt`, `cargo test`, and `cargo clippy`.
- [x] 5.5 Run `openspec status --change implement-polaris-mvp` and confirm the change remains apply-ready.
