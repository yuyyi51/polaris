# Polaris

Polaris is a Rust CLI for storing workspace-local agent context under `.polaris/` and recalling it after conversation compaction.

## MVP Commands

```bash
polaris init
polaris status --json
polaris remember --key goal --text "task goal" --title "Goal"
polaris remember --key decision --stdin --title "Decision"
polaris remember --key goal --replace --text "updated task goal"
polaris forget goal
polaris note create --title "Architecture notes"
polaris recall
polaris clear --yes
polaris hook session-start
polaris hook post-compact
polaris hook post-tool-use
```

Inline memories are keyed. Use a short stable key such as `goal`, `plan`, `decision.storage`, `blocker`, or `verification`. Reusing a key fails unless `--replace` is provided, which makes overwrites explicit. Use `polaris forget <key>` to remove one keyed inline memory, or `polaris clear --yes` to clear all Polaris memory and note files.

## Codex Hook

Configure your agent CLI to call Polaris after conversation compaction. By default, hook commands only ask the agent to run `polaris recall`; they do not inline stored memory.

If your CLI supports compact `SessionStart` hooks, use the direct example at `examples/codex-hooks/hooks.json`, or adapt this `.codex/hooks.json` snippet:

```json
{
  "hooks": {
    "SessionStart": [
      {
        "matcher": "compact",
        "hooks": [
          {
            "type": "command",
            "command": "polaris hook session-start",
            "statusMessage": "Checking Polaris recall"
          }
        ]
      }
    ]
  }
}
```

If your CLI does not trigger a compact `SessionStart` hook after compaction, use the fallback example at `examples/codex-hooks/post-compact-hooks.json`. It records compact completion with `polaris hook post-compact`, then checks that pending state on `polaris hook post-tool-use`.

The examples assume `polaris` is available on `PATH`. If you install the binary somewhere else, replace the `polaris hook ...` commands with the appropriate absolute command path.

Run `polaris init` in a workspace before expecting hook output. If `.polaris/` is absent, hook commands exit successfully without output.

To customize the hook prompt, add `[hooks].recall_prompt` to `.polaris/config.toml` in the workspace:

```toml
[hooks]
recall_prompt = "Polaris has saved context. Run `polaris recall` before continuing."
```

If `.polaris/config.toml` is absent, Polaris checks `~/.polaris/config.toml`. Workspace config takes precedence over user config; the two files are not merged. If neither file exists, Polaris uses the built-in prompt.

The only prompt placeholder is `{{recall}}`. When present, Polaris replaces every `{{recall}}` with the current `polaris recall` output and includes that text directly in hook context:

```toml
[hooks]
recall_prompt = """
Recovered Polaris context:

{{recall}}
"""
```

Using `{{recall}}` exposes `polaris recall` output to the agent context. Without that placeholder, a configured prompt is emitted as-is and stored memory is not inlined unless the prompt text itself contains it.

## TraeCLI Integration

Polaris also ships a TraeCLI (Coco) host adapter. Each `polaris hook` subcommand accepts `--target <codex|traecli>`; the default is `codex`, which preserves the existing Codex behavior byte-for-byte. Pass `--target traecli` to switch the stdin/stdout shape to TraeCLI's hook contract: stdout uses `hookSpecificOutput.additionalContext` without the `hookEventName` wrapper.

TraeCLI does not have Codex's `SessionStart`+`compact` matcher; instead, its dedicated `post_compact` event is the recall trigger. Configure only that event with the bundled example at `examples/traecli-hooks/hooks.json`, or adapt this snippet for your TraeCLI hooks file:

```json
{
  "hooks": {
    "PostCompact": [
      {
        "matcher": "*",
        "hooks": [
          {
            "type": "command",
            "command": "polaris hook post-compact --target traecli"
          }
        ]
      }
    ]
  }
}
```

`polaris hook session-start --target traecli` and `polaris hook post-tool-use --target traecli` are intentional no-ops, leaving the CLI surface ready for future TraeCLI events.

The same `[hooks].recall_prompt` configuration and safety rules apply: the TraeCLI hook does **not** inline stored memory unless `recall_prompt` explicitly contains the `{{recall}}` placeholder. Without that placeholder, `additionalContext` only instructs the agent to run `polaris recall`.

## Polaris Skill

This repository ships a host-agnostic Polaris skill at `skills/polaris/`. The same `skills/polaris/` folder is copied to any markdown-skill host. To install it, copy the folder into the host's skills directory:

```bash
cp -R skills/polaris ~/.codex/skills/
cp -R skills/polaris ~/.coco/skills/
```

The bundled skill is intentionally minimal. It only requires `SKILL.md`; it does not include or require `agents/openai.yaml`.
