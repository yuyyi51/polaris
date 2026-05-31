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
```

Inline memories are keyed. Use a short stable key such as `goal`, `plan`, `decision.storage`, `blocker`, or `verification`. Reusing a key fails unless `--replace` is provided, which makes overwrites explicit. Use `polaris forget <key>` to remove one keyed inline memory, or `polaris clear --yes` to clear all Polaris memory and note files.

## Codex Hook

Configure Codex to call Polaris on compact session starts. The hook only asks the agent to run `polaris recall`; it does not inline stored memory.

Copy the example at `examples/codex-hooks/hooks.json` into your Codex hooks configuration, or adapt this `.codex/hooks.json` snippet:

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

The example assumes `polaris` is available on `PATH`. If you install the binary somewhere else, replace `polaris hook session-start` with the appropriate absolute command path.

Run `polaris init` in a workspace before expecting hook output. If `.polaris/` is absent, the hook exits successfully without output.

## Codex Skill

This repository includes a copyable Codex skill at `skills/codex/polaris/`. To install it into a Codex environment, copy the folder into your skills directory:

```bash
cp -R skills/codex/polaris ~/.codex/skills/
```

The bundled skill is intentionally minimal. It only requires `SKILL.md`; it does not include or require `agents/openai.yaml`.
