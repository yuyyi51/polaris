## Context

The repository already ships a copyable Codex skill at `skills/codex/polaris/`, and README documents a manual `cp -R` installation command. There is no checked-in script for users or tests to exercise the supported install path.

## Goals / Non-Goals

**Goals:**
- Provide a one-command local installer for the bundled Polaris Codex skill.
- Keep the installer independent of the Rust CLI binary so users can install the skill from a source checkout.
- Make the destination configurable for tests and users with non-default Codex directories.
- Preserve the minimum skill package shape: the installer copies the existing `skills/codex/polaris/` directory as-is.

**Non-Goals:**
- Do not install hooks, modify Codex configuration, or run `polaris init`.
- Do not fetch remote skill content or install third-party skills.
- Do not add Rust code or runtime dependencies.
- Do not remove the documented manual copy path.

## Decisions

### Use a shell script under `scripts/`

Add `scripts/install-codex-skill.sh` because the install workflow is repository tooling, not CLI runtime behavior. A shell script keeps the path copyable and avoids adding a new Rust subcommand that would require building `polaris` before installing the skill guidance.

Alternative considered: add `polaris skill install`. That is more discoverable after the binary is installed, but it does not help the first-time setup case and expands the CLI surface for a repository packaging task.

### Default destination follows Codex home conventions

The script accepts `POLARIS_CODEX_SKILLS_DIR` to override the exact skills root. Without that override, it uses `$CODEX_HOME/skills` when `CODEX_HOME` is set, prefers an existing `$HOME/.codex-app/skills`, and falls back to `$HOME/.codex/skills`. This supports both current Codex app environments and classic Codex CLI layouts while giving tests and users an explicit destination override.

Alternative considered: always install into `~/.codex/skills`. That is simple, but it ignores users who set `CODEX_HOME` and misses current Codex app environments where the active home is `.codex-app`.

### Replace the bundled skill directory

The script removes any existing destination `polaris` skill directory before copying the repository version. That makes reruns predictable and updates stale installs. The script only touches the `polaris` subdirectory under the chosen skills root.

Alternative considered: fail if the destination already exists. That avoids overwriting local edits, but it makes the common "update my installed skill" workflow less useful. The narrow target path keeps the blast radius constrained.

## Risks / Trade-offs

- Existing local edits under the destination `polaris` skill directory can be overwritten -> The script prints the exact destination before replacing it and only modifies that one directory.
- Shell scripts are less portable to non-POSIX environments -> This repository already targets local developer workflows, and the manual copy command remains documented.
- Users may expect this to install Codex hooks too -> README and script output should describe the scope as skill installation only.

## Migration Plan

No data migration is required. Users can keep using the manual copy command or run the new script from the repository root.

## Open Questions

None.
