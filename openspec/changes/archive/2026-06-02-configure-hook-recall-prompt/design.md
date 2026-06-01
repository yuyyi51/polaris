## Context

Polaris currently emits a fixed Codex hook `additionalContext` string after compact `SessionStart` and fallback `PostToolUse` events. That string asks the agent to run `polaris recall` and intentionally does not include stored memory contents. This is safe by default, but it depends on the resumed agent following the instruction.

The change keeps the default behavior while allowing users to configure the hook prompt for a workspace or for their user account. A configured prompt can include the literal `{{recall}}` placeholder to inline the same text that `polaris recall` would print.

## Goals / Non-Goals

**Goals:**

- Allow hook prompt text to be configured from `.polaris/config.toml`.
- Fall back to `~/.polaris/config.toml` when the workspace config does not exist.
- Preserve the existing built-in hook prompt when neither config exists.
- Use `{{recall}}` as the only inline recall placeholder and replace it with current recall output.
- Share rendering behavior across `polaris hook session-start` and `polaris hook post-tool-use`.

**Non-Goals:**

- Add a new `inline_recall` flag or other separate switch.
- Add command-line hook prompt flags.
- Merge workspace and user config values.
- Inline note file contents beyond what `polaris recall` already prints.
- Change when hooks decide to stay quiet.

## Decisions

1. Use a minimal TOML config shape.

   The config file will support `[hooks].recall_prompt` as a string. TOML gives users multiline strings without encoding prompt text into Codex hook JSON. The implementation can add the `toml` crate for parsing unless an equivalent parser is already available.

   Alternative considered: hook command flags such as `--prompt-file` or `--inline-recall`. Flags keep configuration outside `.polaris/`, but they make hook examples harder to copy and create two places to update when prompt behavior changes.

2. Resolve config by precedence, not merging.

   The loader checks `.polaris/config.toml` in the active workspace first, then `~/.polaris/config.toml`, then returns no configured prompt. The first file that exists wins. Missing files are not errors; malformed or unreadable files should produce a clear error that names the file.

   Alternative considered: merge user config with workspace config. Merging is unnecessary for one field and creates surprising behavior if a workspace wants to deliberately avoid a user-level prompt.

3. Make `{{recall}}` the inline switch.

   The renderer replaces every literal `{{recall}}` occurrence in the selected prompt with `store.recall()`. If the configured prompt does not contain `{{recall}}`, the hook outputs the prompt as-is and does not inline stored memory. This keeps the user's requested model simple: the placeholder itself is the opt-in.

   Alternative considered: an explicit boolean such as `inline_recall`. That adds state that can conflict with the template contents and was intentionally rejected.

4. Keep default hook output unchanged.

   If no config file exists, hook output uses the current built-in prompt and does not inline stored memory. This preserves existing hook safety and keeps bundled examples safe by default.

## Risks / Trade-offs

- Configured inline recall can expose all `polaris recall` output to the agent context. Mitigation: make inline behavior opt-in through `{{recall}}`, keep examples on the default behavior, and document the privacy trade-off.
- Large recall output can make hook context noisy after compaction. Mitigation: inline only the existing recall summary, not note file contents, and leave users in control of whether to include the placeholder.
- Malformed config can interrupt hook output. Mitigation: report a concise error with the config path so users can fix the prompt instead of silently receiving unexpected defaults.
