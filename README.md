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

Configure your agent CLI to call Polaris after conversation compaction. Hook commands only ask the agent to run `polaris recall`; they do not inline stored memory.

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

## Codex Skill

This repository includes a copyable Codex skill at `skills/codex/polaris/`. To install it into a Codex environment, copy the folder into your skills directory:

```bash
cp -R skills/codex/polaris ~/.codex/skills/
```

The bundled skill is intentionally minimal. It only requires `SKILL.md`; it does not include or require `agents/openai.yaml`.
