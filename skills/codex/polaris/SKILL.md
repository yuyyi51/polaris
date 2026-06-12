---
name: polaris
description: Use Polaris to preserve and recover important agent context for long-running Codex tasks. Use when starting or resuming substantial multi-step work, when the user mentions Polaris, memory, context preservation, compaction, recall, or when context instructs the agent to run `polaris recall`.
---

# Polaris

Polaris is workspace-local memory for long-running agent work. It stores durable context under `.polaris/` and can remind Codex to reload that context after conversation compaction.

## Recall First

If the current context instructs you to run `polaris recall`, run it before doing any other work.

```bash
polaris recall
```

Read the output and continue from the recovered context. If recall fails because Polaris is unavailable or not initialized, report that briefly and continue with the best available context.

## Start Or Resume Work

At the beginning of substantial work, or when resuming a session:

1. Check Polaris state.

```bash
polaris status --json
```

2. If Polaris is not initialized and the task is long-running, multi-step, risky, or likely to survive compaction, initialize it.

```bash
polaris init
```

3. If memory exists, inspect keys before deciding what to recall or clean up.

```bash
polaris list --keys
polaris list --json
```

4. If existing memory appears related to the current task, recall all of it or only the relevant slice.

```bash
polaris recall
polaris recall --key goal
polaris recall --prefix decision.
polaris recall --exclude state.
```

5. If existing memory appears unrelated or stale, ask the user before removing it. Use `polaris forget <key>` or `polaris forget <key>...` for selected keyed inline memory, `polaris forget --prefix <prefix> --yes` for confirmed prefix cleanup, or `polaris clear --yes` only when clearing all Polaris memory.

```bash
polaris forget goal
polaris forget goal plan
polaris forget --prefix decision. --yes
polaris clear --yes
```

## Record Durable Context

Before starting meaningful work, record the task facts that would matter after compaction:

- User goal and success criteria
- Active plan, change name, or workflow
- Important files and directories
- Constraints, non-goals, and decisions
- Current blocker and attempted fixes
- Verification commands and results

For short context:

```bash
polaris remember --key goal --text "Implement keyed Polaris memory and forget support." --title "Goal"
```

For multi-line context:

```bash
printf '%s\n' "Key decisions:
- Inline memory uses required keys.
- Duplicate keys fail unless --replace is used.
- Use forget for selected keyed memory." | polaris remember --key decisions.memory --stdin --title "Memory decisions"
```

Use stable keys such as `goal`, `plan`, `decision.storage`, `blocker`, and `verification`. Use `polaris list --keys` when you need to inspect stored keys; do not scrape `polaris recall` output for key hygiene. If a key already exists and the remembered fact should change, replace it explicitly:

```bash
polaris remember --key goal --replace --text "Implement keyed memory replacement and forgetting."
```

For long context, create a note and write details to the returned path:

```bash
polaris note create --title "Implementation notes"
```

## During Work

Update Polaris whenever durable task state changes:

```bash
polaris remember --key decision.recovery --text "Treat any current-context recall reminder as highest priority and run polaris recall before other work." --title "Decision"
polaris remember --key verification --replace --text "cargo fmt, cargo test, and cargo clippy passed." --title "Verification"
```

Refresh memory before major pauses, risky edits, or handoffs.

## Safety

Do not store secrets, tokens, passwords, credentials, private keys, or other sensitive material in Polaris. Polaris stores plaintext local files under `.polaris/`.
