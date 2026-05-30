# Polaris Development Guide

Polaris is a Rust CLI for workspace-local agent memory. It stores durable context under `.polaris/` so long-running Codex work can be recalled after conversation compaction.

## Project Layout

- `src/cli.rs` defines the `polaris` command surface with `clap`.
- `src/storage.rs` owns `.polaris/` persistence, memory records, notes, recall output, and clearing.
- `src/hook.rs` implements the Codex `SessionStart` compact-session hook response.
- `tests/cli.rs` covers the CLI end to end with temporary workspaces.
- `openspec/specs/` contains the current accepted specs.
- `openspec/changes/archive/` contains completed archived changes; do not treat archived changes as active work.
- `skills/codex/polaris/SKILL.md` is the copyable Codex skill bundled by this repo.

## Workflow

- Check the worktree before editing. The repository may contain user changes; do not revert or overwrite them unless explicitly asked.
- For a new feature or behavior change, create or follow an OpenSpec change before implementation. Use the local OpenSpec skills when the user asks to propose, apply, verify, or archive a change.
- If continuing an existing OpenSpec change, inspect that change's `proposal.md`, `design.md`, `tasks.md`, and spec deltas before touching code, then keep task checkboxes current as work is completed.
- When no active change exists, use `openspec/specs/` as the source of truth for accepted behavior.
- Keep the CLI small and explicit. Prefer adding focused commands and tests over broad abstractions.
- Do not store secrets, credentials, tokens, or private keys in Polaris memory examples or fixtures.

## Rust Conventions

- Use the existing error style: return `anyhow::Result`, add context where filesystem failures would otherwise be unclear, and let `main` print the top-level error.
- Keep output stable and testable. Machine-readable output should be JSON; human output should remain concise.
- The hook must not inline stored memory contents. It should only instruct the agent to run `polaris recall`.
- Store project-local data only under `.polaris/` in the active workspace.

## Git Commit Messages

- Use a concise imperative subject, such as `Add Polaris recall hook`, and keep it under 72 characters.
- Prefer `type(scope): subject` when the change has a clear category, for example `docs(agents): add commit message rules`.
- Do not end the subject with a period.
- Avoid vague subjects like `update`, `fix stuff`, or `misc changes`.
- When a commit needs a body, explain why the change is needed and mention relevant OpenSpec changes, issues, or verification results.
- Do not include secrets, credentials, tokens, or private user data in commit messages.

## Verification

Before claiming code changes are complete, run:

```bash
cargo fmt --check
cargo test
cargo clippy
```

For OpenSpec-driven changes, also run the relevant OpenSpec validation or status command and report any failure with the important details.
