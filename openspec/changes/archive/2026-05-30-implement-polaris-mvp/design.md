## Context

Polaris starts from an empty Rust CLI scaffold. The project goal is to provide durable, workspace-local context for long-running agent work, especially after Codex compacts the conversation. The original design considered `PreToolUse`, but the MVP will rely on Codex `SessionStart` with `source: "compact"` because that is the direct lifecycle point after compaction.

Codex command hooks receive JSON on stdin. `SessionStart` matches on `source` values including `compact`, and can add model-visible context through `hookSpecificOutput.additionalContext`. Polaris will use that hook surface only to ask the agent to run `polaris recall`.

## Goals / Non-Goals

**Goals:**
- Provide a small Rust CLI that manages `.polaris/` in the current workspace.
- Let agents record short memory entries and create longer markdown note files.
- Let agents inspect and clear existing workspace memory before starting unrelated work.
- Let agents recall saved context in a model-readable format after compaction.
- Provide `polaris hook session-start` for Codex `SessionStart` compact events.
- Keep hook execution quiet in workspaces that have not run `polaris init`.

**Non-Goals:**
- Do not implement `PreToolUse`, `PostCompact`, or repeated tool-call enforcement in the MVP.
- Do not automatically inline memory contents from the hook.
- Do not modify `AGENTS.md` or any global project instruction file.
- Do not sync memory outside the local filesystem.
- Do not provide encryption, access control, or multi-user coordination in the MVP.

## Decisions

### Use `.polaris/` as the workspace boundary

Polaris will resolve the active workspace as the current working directory passed to the command, or the hook input `cwd` for hook execution. All persistent data lives under that directory:

```text
.polaris/
  state.json
  memories.jsonl
  docs/
```

`state.json` stores `schema_version` and basic metadata. `memories.jsonl` stores append-friendly records. `docs/` stores long markdown notes.

Alternative considered: place state under a user-level directory keyed by repository path. That avoids repository-local hidden files, but it makes inspection and cleanup less obvious and weakens the per-workspace mental model.

### Keep memory records append-friendly

Each memory record will include an id, creation timestamp, kind, title, and either inline text or a document path. JSON Lines keeps appends simple, makes recovery from partial writes easier than rewriting one large JSON document, and is easy to inspect manually.

Alternative considered: a single JSON database file. That is simpler to parse at first but more fragile for repeated appends and future migration.

### Make active CLI commands explicit when `.polaris/` is missing

Agent-facing commands such as `remember`, `note create`, `recall`, and `clear` will report that Polaris has not been initialized when `.polaris/` is absent. `status --json` will still succeed and return `initialized: false` so an agent can safely check for stale memory at the beginning of a session. `init` is the only command that creates the directory.

Hook commands are the exception: `polaris hook session-start` exits successfully with no output when `.polaris/` is absent. This allows a user-level hook configuration to be enabled globally without disrupting unrelated workspaces.

### Use `SessionStart compact` as the only MVP hook trigger

The hook command will read the Codex hook JSON from stdin and only emit context when:

- `hook_event_name` is `SessionStart`
- `source` is `compact`
- `.polaris/` exists for the hook `cwd`
- recallable memory exists

When those conditions are met, stdout will be JSON shaped for Codex `SessionStart`:

```json
{
  "hookSpecificOutput": {
    "hookEventName": "SessionStart",
    "additionalContext": "Polaris has saved workspace context for this project. Run `polaris recall` immediately before doing any more work."
  }
}
```

The hook must not include saved memory text. The agent must explicitly run `polaris recall` so the recall step is visible, debuggable, and under the same command interface as normal usage.

Alternative considered: `PreToolUse` could remind the agent on first tool call after compaction. It is excluded because `SessionStart compact` is earlier and cleaner, and because tool interception is not a complete enforcement boundary.

### Provide focused CLI commands

MVP commands:

```text
polaris init
polaris status --json
polaris remember --text <text> [--title <title>]
polaris remember --stdin [--title <title>]
polaris note create --title <title>
polaris recall
polaris clear --yes
polaris hook session-start
```

`recall` defaults to human-readable Markdown-like output because it is meant for the agent to ingest. `status --json` is the structured inspection point for automation.

## Risks / Trade-offs

- Hook reminder can still be ignored by the model -> Keep the reminder short and imperative, and make `recall` output easy to consume.
- Local `.polaris/` can contain sensitive information -> Keep storage explicit, visible, and clearable; document that Polaris stores plaintext local files.
- JSONL records can accumulate stale context -> Provide `status --json` and `clear --yes` in MVP, with more selective deletion left for later.
- Hook behavior depends on Codex hook schema stability -> Keep hook parsing tolerant of extra fields and write tests around the current `SessionStart` shape.
- Empty note files can be recalled before the agent writes content -> Recall should list the note path and title, making the incomplete note visible instead of silently dropping it.
