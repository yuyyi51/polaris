## Context

Polaris currently supports one hook path: `polaris hook session-start` reads hook JSON, detects `SessionStart` with `source: "compact"`, and emits a short instruction telling the agent to run `polaris recall`. That works for hook systems that fire a compact session-start event after compaction.

Some CLI hook systems fire a `PostCompact` event after compaction but do not provide a model-visible session-start hook at that point. They can still provide model-visible context on later `PostToolUse` hooks. Polaris can bridge that gap by recording a workspace-local "compact just happened" marker during `PostCompact`, then consuming that marker on the next `PostToolUse`.

## Goals / Non-Goals

**Goals:**

- Add `polaris hook post-compact` to mark that compaction has just completed for the workspace.
- Add `polaris hook post-tool-use` to emit the existing recall instruction once when a pending compact marker exists and recallable memory is present.
- Preserve the current `polaris hook session-start` behavior for tools that support compact session-start hooks.
- Keep all hook output memory-safe: hooks instruct the agent to run `polaris recall` and never inline stored memory.
- Keep the bundled skill source-agnostic so it describes the agent action, not the hook event that produced the reminder.

**Non-Goals:**

- Do not replace the existing `SessionStart` compact hook flow.
- Do not enforce recall before every tool call.
- Do not store compact summaries, prompts, tool arguments, tool outputs, or stored Polaris memory in hook state.
- Do not add cross-workspace or multi-user coordination.
- Do not add new runtime dependencies.

## Decisions

### Store fallback hook state in a dedicated file

Polaris will store fallback hook state in `.polaris/hook-state.json`, separate from `.polaris/state.json` and `.polaris/memories.jsonl`.

Example shape:

```json
{
  "schema_version": 1,
  "compact_recall_pending": true,
  "recorded_at": "2026-06-01T14:30:00Z"
}
```

Rationale: the pending compact marker is transient operational state, not memory content or initialization metadata. A separate file avoids migrating `state.json` and makes future cleanup simple.

Alternative considered: append a special memory record. That would pollute recall output and risks treating hook state as durable user context.

Alternative considered: write a zero-byte marker file. That is simple, but JSON leaves room for timestamps and schema versioning without changing the storage approach later.

### Make the fallback reminder one-shot

`polaris hook post-compact` records `compact_recall_pending: true` and exits without stdout. `polaris hook post-tool-use` checks for that pending marker and consumes it before deciding whether to emit reminder output.

If recallable memory exists, `post-tool-use` emits hook JSON with `hookSpecificOutput.hookEventName: "PostToolUse"` and the same `additionalContext` recall instruction used by the `SessionStart` compact hook. If no recallable memory exists, it stays quiet after consuming the marker.

Rationale: compaction creates one recovery opportunity. Repeating the same recall reminder on every tool call would create noise and could interfere with normal work.

Alternative considered: keep the marker until the agent actually runs `polaris recall`. Polaris cannot reliably observe that the model obeyed the reminder across all hook systems without adding more command coupling, so a one-shot reminder is simpler and predictable.

### Keep hook commands tolerant and quiet outside their event

Each hook command will parse the common hook fields it needs:

- `hook_event_name`
- `cwd`

The command will use the input `cwd` when present and otherwise fall back to the process current directory, matching the existing session-start behavior. It will ignore unrelated hook event names by exiting successfully with no stdout. If `.polaris/` is absent, hook commands also exit successfully with no stdout so global hook configuration remains safe.

Rationale: users may install hooks broadly across workspaces and tools. Quiet no-op behavior is already part of the existing Polaris hook contract.

Alternative considered: fail on mismatched hook events. That would make broad hook configuration brittle and inconsistent with the existing session-start hook.

### Provide separate direct and fallback examples

The existing copyable compact `SessionStart` example should remain available for tools that support it. The fallback `PostCompact` plus `PostToolUse` example should be documented as an alternative for tools that do not trigger compact session-start hooks after compaction.

Rationale: configuring both direct and fallback flows in a tool that supports both could produce duplicate recall reminders. Separate examples make the intended choice clear.

Alternative considered: put all hooks into one example file. That is convenient to copy, but it blurs the distinction between the preferred direct hook and the fallback flow.

### Keep skill recovery guidance source-agnostic

The Polaris skill should continue to say that if current context instructs the agent to run `polaris recall`, the agent must run it before other work. It should not say whether that reminder came from `SessionStart`, `PostCompact`, `PostToolUse`, or any other integration.

Rationale: hook integration details belong in README and examples. The skill is agent behavior guidance and should remain stable as hook sources evolve.

Alternative considered: document every supported hook source in the skill. That makes the skill more brittle and distracts from the action the agent must take.

## Risks / Trade-offs

- Workspace-level pending state can be consumed by another concurrent agent in the same workspace -> Polaris already uses workspace-local memory without multi-user coordination; document this as outside the scope of the fallback hook.
- A tool may never fire `PostToolUse` after `PostCompact` -> The pending state remains harmless and will be overwritten by a later `PostCompact`.
- The fallback flow can remind later than the direct `SessionStart` flow -> Keep the direct flow documented as preferred when available.
- Users may configure both direct and fallback flows together -> Provide separate examples and README guidance that explain when to use each flow.
- Hook input schemas may vary by tool -> Parse only common fields and ignore extra JSON fields.
