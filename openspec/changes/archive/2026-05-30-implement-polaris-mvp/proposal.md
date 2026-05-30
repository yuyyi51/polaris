## Why

Long-running agent tasks can lose critical context after conversation compaction, causing the agent to drift away from the user's goal or forget required project workflow. Polaris provides a local, session-scoped memory surface that survives compaction and nudges the agent to reload it at the right time.

## What Changes

- Add a Rust CLI named `polaris` for initializing local project memory, checking memory status, recording short notes, creating longer note files, clearing stale memory, and recalling saved context.
- Store Polaris data in the current workspace under `.polaris/`.
- Add a Codex hook entry point for `SessionStart` events where `source` is `compact`.
- Make the compact-session hook inject only an instruction to run `polaris recall`; it must not inline stored memory content.
- Keep hook behavior quiet when `.polaris/` is absent so users can enable hooks globally without breaking unrelated projects.
- Do not implement `PreToolUse` recall enforcement in this MVP.

## Capabilities

### New Capabilities
- `polaris-recall`: Local agent memory storage, recall output, and Codex compact-session hook prompting.

### Modified Capabilities
- None.

## Impact

- Affected code: Rust CLI entry point and supporting modules under `src/`.
- Affected runtime data: `.polaris/` directory in the active workspace.
- Affected external integration: Codex hooks configured by users to call `polaris hook session-start` for `SessionStart` events.
- Dependencies: CLI argument parsing, JSON serialization, and local filesystem persistence are expected.
