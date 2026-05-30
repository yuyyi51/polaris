# Polaris Development Guide

This repository is developing `polaris`, a Rust CLI that helps agents preserve and recall important task context after Codex conversation compaction.

## Required Workflow

- When starting a new session, resuming after context compaction, or continuing Polaris implementation, use the `openspec-apply-change` skill before making implementation changes.
- The current active OpenSpec change is `implement-polaris-mvp` unless the user explicitly chooses another change.
- Start by running:

```bash
openspec list --json
openspec status --change implement-polaris-mvp --json
openspec instructions apply --change implement-polaris-mvp --json
```

- Read every context file returned by `openspec instructions apply`. For the current spec-driven change, expect:
  - `openspec/changes/implement-polaris-mvp/proposal.md`
  - `openspec/changes/implement-polaris-mvp/design.md`
  - `openspec/changes/implement-polaris-mvp/specs/polaris-recall/spec.md`
  - `openspec/changes/implement-polaris-mvp/tasks.md`
- Implement pending tasks in order and immediately update the matching checkbox in `tasks.md` from `- [ ]` to `- [x]` after each completed task.
- If implementation reveals a design or spec problem, pause and update/ask about the OpenSpec artifacts instead of silently drifting from them.

## Current MVP Decisions

- Build Polaris as a Rust CLI.
- Store all project-local memory under `.polaris/` in the active workspace.
- Support initialization, status, short memory recording, long note creation, recall, clearing stale memory, and a Codex hook command.
- Use Codex `SessionStart` with `source: "compact"` as the MVP recovery trigger.
- The hook must only instruct the agent to run `polaris recall`.
- The hook must not inline stored memory contents.
- Do not implement `PreToolUse`, `PostCompact`, or repeated tool-call enforcement in the MVP unless the OpenSpec change is updated.

## Verification

Before claiming implementation work is complete, run:

```bash
cargo fmt
cargo test
cargo clippy
openspec status --change implement-polaris-mvp
```

Report any command that cannot be run or fails, including the important error details.
