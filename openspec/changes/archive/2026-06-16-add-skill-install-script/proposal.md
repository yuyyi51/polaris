## Why

The bundled Polaris Codex skill is currently documented as a manual folder copy. A checked-in install script makes the common setup path easier to run, repeat, and validate without asking users to remember the target directory layout.

## What Changes

- Add a repository script that installs `skills/codex/polaris/` into a Codex skills directory.
- Make the script safe to rerun by replacing the target `polaris` skill directory atomically enough for local setup use.
- Allow users to override the destination skills root while defaulting to the standard Codex skills directory.
- Update README guidance to prefer the script while keeping the manual copy command discoverable.
- Add regression coverage that the script installs the bundled `SKILL.md` into the requested destination.

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `polaris-codex-skill`: Add a supported script-based installation path for the bundled Codex skill.

## Impact

- Affected files: a new repository script, README installation guidance, `tests/cli.rs`, and the `polaris-codex-skill` accepted spec after archive.
- Affected users: Polaris users who want to install the bundled skill into Codex with one command.
- Runtime behavior: no changes to the `polaris` CLI binary.
- Dependencies: no new Rust or external runtime dependencies are expected.
