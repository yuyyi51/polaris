# Polaris

Polaris is a Rust CLI for storing workspace-local agent context under `.polaris/` and recalling it after conversation compaction.

## MVP Commands

```bash
polaris init
polaris status --json
polaris remember --text "task goal" --title "Goal"
polaris remember --stdin --title "Decision"
polaris note create --title "Architecture notes"
polaris recall
polaris clear --yes
polaris hook session-start
```

## Codex Hook

Configure Codex to call Polaris on compact session starts. The hook only asks the agent to run `polaris recall`; it does not inline stored memory.

Example `.codex/hooks.json`:

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

Run `polaris init` in a workspace before expecting hook output. If `.polaris/` is absent, the hook exits successfully without output.

## Codex Skill

This repository includes a copyable Codex skill at `skills/codex/polaris/`. To install it into a Codex environment, copy the folder into your skills directory:

```bash
cp -R skills/codex/polaris ~/.codex/skills/
```

The bundled skill is intentionally minimal. It only requires `SKILL.md`; it does not include or require `agents/openai.yaml`.
