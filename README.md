# Polaris

Polaris is a Rust CLI for storing workspace-local agent context under `.polaris/` and recalling it after conversation compaction.

## MVP Commands

```bash
polaris init
polaris status --json
polaris remember --key goal --text "task goal" --title "Goal"
polaris remember --key decision --stdin --title "Decision" --lifecycle durable
polaris remember --key state.branch --text "working on filters" --lifecycle state
polaris remember --key goal --replace --text "updated task goal"
polaris diff --key goal
polaris diff --key goal --json
polaris lifecycle move --key state.branch --to state
polaris lifecycle move --prefix state. --from durable --to state --yes
polaris lifecycle move --id <record-id> --to archive
polaris lifecycle move --kind note --from log --to archive --yes
polaris rename old.key new.key
polaris forget goal
polaris forget goal plan
polaris forget --prefix decision. --yes
polaris note create --title "Architecture notes" --lifecycle archive
polaris list --keys
polaris list --json
polaris list --json --lifecycle state
polaris recall
polaris recall --key goal
polaris recall --prefix decision.
polaris recall --exclude state.
polaris recall --lifecycle durable
polaris merge --into decision.summary decision.storage decision.hooks
polaris merge apply .polaris/maintenance/merge-decision-summary-<id>.md --yes
polaris merge apply .polaris/maintenance/merge-decision-summary-<id>.md --yes --forget-sources
polaris prune --suggest
polaris prune --suggest --json
polaris compact --suggest
polaris compact --suggest --json
polaris clear --yes
polaris hook session-start
polaris hook post-compact
polaris hook post-tool-use
```

Inline memories are keyed. Use a short stable key such as `goal`, `plan`, `decision.storage`, `blocker`, or `verification`. Reusing a key fails unless `--replace` is provided, which makes overwrites explicit.

Memory records may include lifecycle metadata:

- `durable`: stable project knowledge and decisions. This is the default for new and legacy records.
- `state`: current workspace state that may become stale.
- `log`: short work logs and handoff breadcrumbs.
- `archive`: historical context that should stay available but is not normally recalled.

Use `--lifecycle <durable|state|log|archive>` with `polaris remember` or `polaris note create`. Replacing a keyed inline memory preserves its existing lifecycle unless a new `--lifecycle` is provided. Replacement summaries include `updated_at`, `replacement_count`, and `replaced_from`. Polaris also saves the previous active inline record to `.polaris/replacement-history.jsonl` for later diff review.

Use `polaris lifecycle move` to reclassify existing memory without rewriting memory text or note files:

```bash
polaris lifecycle move --key state.branch --to state
polaris lifecycle move --prefix state. --from durable --to state --yes
polaris lifecycle move --id <record-id> --to archive
polaris lifecycle move --kind note --from log --to archive --yes
polaris lifecycle move --prefix state. --to state --yes --json
```

Single-record moves selected by `--key` or `--id` do not require confirmation. Batch moves selected by `--prefix` or `--kind` require `--yes`. Notes are selected by record id or kind; note title matching is intentionally unsupported.

Use `polaris diff --key <key>` after `remember --replace` to review what changed in the latest replacement:

```bash
polaris diff --key goal
polaris diff --key goal --json
```

Diff compares the current active keyed inline memory with the latest saved replacement snapshot. Existing records replaced before replacement history existed remain valid, but `diff --key` reports that no snapshot is available for those keys.

Use `polaris list --keys` for one key per line, or `polaris list --json` for machine-readable record summaries without inline memory text. Add `--lifecycle <value>` to `polaris list --json` to inspect one lifecycle. Use `polaris recall --key <key>`, `polaris recall --prefix <prefix>`, `polaris recall --exclude <prefix>`, or `polaris recall --lifecycle <value>` to recover only relevant context. Lifecycle recall composes with exact key, prefix, and exclude-prefix filters.

`polaris status --json` includes lifecycle counts and advisory `stale_volatile_memory` hints for old `state` and `log` records. These hints are machine-readable and do not change command exit status.

Use `polaris forget <key>` or `polaris forget <key>...` to remove selected keyed inline memories. Use `polaris forget --prefix <prefix> --yes` for confirmed prefix cleanup, or `polaris clear --yes` to clear all Polaris memory and note files.

Use `polaris rename <old-key> <new-key>` to rename a keyed inline memory without changing its text or metadata. Rename rejects missing sources and existing destination keys.

Use `polaris merge --into <target-key> <source-key>...` to create an editable markdown draft under `.polaris/maintenance/`. Draft creation does not mutate active memory. Edit the text under `## Merged Memory`, then apply it explicitly:

```bash
polaris merge apply .polaris/maintenance/merge-decision-summary-<id>.md --yes
```

Add `--forget-sources` to remove the source keyed inline memories in the same confirmed apply operation. Invalid drafts and unconfirmed apply commands leave active memory unchanged.

Use `polaris prune --suggest` and `polaris compact --suggest` to inspect deterministic maintenance suggestions without mutating memory. Prune suggestions flag old volatile records, duplicate exact text, and replacement metadata. Compact suggestions flag shared key prefixes and related lifecycle groups. Both commands include an agent review prompt by default so the caller can inspect candidates before taking action.

For a copyable safe-edit workflow that combines lifecycle moves and replacement diffs, see `examples/polaris-maintenance/edit-review.md`.

With `--json`, suggest commands return an envelope:

```json
{
  "suggestions": [],
  "prompt": "...",
  "prompt_context": {
    "status": {},
    "keys": []
  }
}
```

Customize maintenance prompts in `.polaris/config.toml` or `~/.polaris/config.toml`. Copyable examples are available under `examples/polaris-config/`:

- `minimal-maintenance-prompts.toml`: minimal prune and compact prompt templates
- `conservative-prune.toml`: stricter deletion review prompt
- `merge-focused-compact.toml`: merge-oriented compact review prompt
- `full-config.toml`: hook prompt plus maintenance prompts

A minimal maintenance prompt config looks like this:

```toml
[maintenance.prompts]
prune = """
Review these prune candidates before deleting anything:
{{suggestions}}

Status:
{{status}}

Keys:
{{keys}}
"""

compact = """
Review these compact candidates before merging anything:
{{suggestions}}

Status:
{{status}}

Keys:
{{keys}}
"""
```

Supported maintenance prompt placeholders are `{{suggestions}}`, `{{status}}`, and `{{keys}}`. Maintenance prompts do not support `{{recall}}`; use targeted `polaris recall --key <key>` or `polaris recall --prefix <prefix>` after reviewing the generated prompt.

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

## Codex Skill

This repository includes a copyable Codex skill at `skills/codex/polaris/`. To install it into a Codex environment, copy the folder into your skills directory:

```bash
cp -R skills/codex/polaris ~/.codex/skills/
```

The bundled skill is intentionally minimal. It only requires `SKILL.md`; it does not include or require `agents/openai.yaml`.
