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

3. If existing memory appears related to the current task, recall it.

```bash
polaris recall
```

4. If existing memory appears unrelated or stale, ask the user before clearing it.

```bash
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
polaris remember --text "Implement the Polaris Codex skill under skills/codex/polaris." --title "Goal"
```

For multi-line context:

```bash
printf '%s\n' "Key decisions:
- The skill is copyable from skills/codex/polaris/.
- It only requires SKILL.md.
- It must not require agents/openai.yaml." | polaris remember --stdin --title "Polaris skill decisions"
```

For long context, create a note and write details to the returned path:

```bash
polaris note create --title "Implementation notes"
```

## During Work

Update Polaris whenever durable task state changes:

```bash
polaris remember --title "Decision" --text "Use SessionStart source=compact as the recovery trigger."
polaris remember --title "Verification" --text "cargo fmt, cargo test, and cargo clippy passed."
```

Refresh memory before major pauses, risky edits, or handoffs.

## Safety

Do not store secrets, tokens, passwords, credentials, private keys, or other sensitive material in Polaris. Polaris stores plaintext local files under `.polaris/`.
