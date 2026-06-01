## Why

Polaris hook reminders currently rely on agents obeying a prompt to run `polaris recall` after conversation compaction. Users need a workspace-configurable hook prompt that can optionally inline the recall output so recovered context is available even when the agent does not follow the reminder.

## What Changes

- Add hook prompt configuration loaded from `.polaris/config.toml`, with fallback to `~/.polaris/config.toml`.
- Let configured hook prompts include `{{recall}}` to inline the current `polaris recall` output into Codex hook `additionalContext`.
- Keep the existing default behavior when no hook prompt is configured: hook output only instructs the agent to run `polaris recall`.
- Apply the same prompt rendering behavior to both compact `SessionStart` and fallback `PostToolUse` hook outputs.
- Document the configuration file locations, precedence, and `{{recall}}` placeholder behavior.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `polaris-recall`: compact-session and fallback hook prompt output becomes configurable and may inline recall output when the configured template includes `{{recall}}`.

## Impact

- Affected code: `src/hook.rs`, `src/storage.rs` or a new focused config module, and `src/cli.rs` if command wiring needs updated dependencies.
- Affected tests: CLI hook tests in `tests/cli.rs` for project config, user config fallback, default behavior, placeholder rendering, and both hook paths.
- Affected docs/examples: `README.md`, `examples/codex-hooks/*.json` only if wording or examples should mention prompt configuration.
- New dependency risk: TOML parsing may require a small dependency unless the existing dependency set already provides one.
