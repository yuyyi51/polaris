## Context

Polaris currently separates durable memory content from maintenance metadata such as lifecycle, replacement history, and touch freshness. `polaris recall` is a read operation, while `polaris touch` refreshes `updated_at` to confirm that a record is still current. Neither operation should become a proxy for whether a memory actually helped an agent.

Agents need a separate, explicit signal for memories that were used as cited context. The signal must remain local, inspectable, cheap to record in batches, and advisory rather than authoritative.

## Goals / Non-Goals

**Goals:**

- Add a `polaris cite` command that records explicit citation events for exact memories.
- Keep recall, freshness, and citation semantics distinct.
- Bind direct citation events to stable record ids, not only mutable keys.
- Preserve lineage so replacement and merge outputs can expose inherited citation context without inflating direct citation counts.
- Make citation metadata visible to agents through JSON summaries and maintenance prompts.
- Teach agents through the bundled skill to cite sparingly and batch citations with nearby work when practical.

**Non-Goals:**

- Do not automatically cite records during `polaris recall`.
- Do not treat `touch` as a usefulness signal.
- Do not add negative citation or "not useful" signals.
- Do not require free-form citation reasons.
- Do not hard-code deletion, retention, or merge decisions based on citation counts.
- Do not support broad citation selectors such as prefix or lifecycle.

## Decisions

### Store citations as a separate append-only event log

Polaris will store citation events in a separate `.polaris/citations.jsonl` file rather than embedding citation counters directly in `.polaris/memories.jsonl`.

Each citation event should include:

- `id`: event id.
- `record_id`: the memory record id that was cited.
- `cited_at`: event timestamp.
- `key`, `title`, `kind`, and `record_created_at` as convenience snapshots when available.

Rationale:

- Appending events matches the current inspectable JSONL model.
- Separate events avoid rewriting memory content for a lightweight usage signal.
- Snapshot metadata keeps old citation events understandable even if keys are later renamed or records are forgotten.

Alternative considered: update citation counters in each `MemoryRecord`. This is simpler to query but makes citation mutate the memory record itself, creates more contention with memory replacement, and loses event-level auditability.

### Resolve selectors to active records before recording

`polaris cite` will support exact selectors only:

- `--key <key>`
- `--keys <key1,key2,...>`
- `--id <record-id>`
- `--ids <id1,id2,...>`

The command will reject conflicting selectors. Single-record selectors will error when the target does not exist. Multi-record selectors will record citations for matched records and report missing keys or ids; if none match, the command will error. It will not support prefix, lifecycle, or kind selectors.

Rationale:

- Citation should mean "these exact memories affected this work", not "everything in this namespace was probably helpful".
- Multi-key and multi-id selectors make batching practical without broad accidental citation.

### Citation metadata is derived for summaries

`polaris list --json` will project citation metadata for each memory summary:

- `direct_cite_count`
- `last_cited_at`
- `inherited_cite_count`
- `citation_source_ids` or equivalent lineage metadata when inherited citations exist

Direct counts are computed from citation events whose `record_id` matches the active record. Inherited counts are computed from known lineage, including `replaced_from` and merge source ids. Inherited citation event counts should be de-duplicated when multiple lineage paths reach the same ancestor.

Rationale:

- Agents need visible facts for judgment without parsing separate JSONL files.
- Separating direct and inherited counts avoids pretending that a newly merged or replaced memory has already been cited directly.

### Preserve lineage for replacements and merges

Replacement already records `replaced_from`. Merge apply will be extended to preserve source record ids on the target record, for example with a `derived_from` list. Existing replacement history can be used to trace replaced records where available.

For a merge:

```text
source A direct citations = 3
source B direct citations = 1
target C direct citations = 0
target C inherited citations = 4
```

For a replacement:

```text
old record direct citations = 2
new record direct citations = 0
new record inherited citations = 2
```

Rationale:

- A new text record has not itself been cited until an agent uses that exact content.
- Historical citation still matters as source context for review and maintenance.

### Keep output low-noise

Default successful output should identify the cited count and selected labels. `--quiet` should suppress normal success output and only report errors.

Rationale:

- Citation is expected to be occasional but potentially paired with other commands.
- Quiet mode supports skill guidance such as `polaris cite --keys goal,decision.product_model --quiet; cargo test`.

### Make maintenance citation-aware but not citation-driven

Prune and compact suggestions may include citation metadata in JSON prompt context or suggestion explanations, but citation counts will not become hard deletion or retention rules.

Rationale:

- A never-cited memory may still be important.
- A heavily cited memory may still be obsolete after the underlying decision changes.

### Update skill guidance rather than enforcing agent behavior

The bundled skill will teach:

- `recall` means shown context.
- `touch` means still fresh.
- `cite` means explicitly used as evidence.
- Do not cite every recalled record.
- Prefer batching citations using `--keys`/`--ids` and `--quiet`.
- Cite during natural checkpoints or in the same shell invocation as nearby work when practical.

Rationale:

- Polaris cannot know agent cognition automatically.
- Skill guidance is the right layer for behavior norms and cost-conscious tool use.

## Risks / Trade-offs

- Citation events can still be noisy if agents overuse the command. -> Mitigate with skill guidance, exact selectors only, and no automatic recall citation.
- Derived inherited counts add query complexity. -> Keep lineage fields simple and test replacement/merge cases explicitly.
- Citation events for forgotten records may accumulate. -> Preserve them as audit history initially; future maintenance can consider citation event cleanup if needed.
- Multi-command shell batching can hide which command failed if callers chain commands poorly. -> Document examples that use simple `;` batching for best-effort citation and preserve normal command errors.
- Existing records will have zero citation metadata. -> Treat this as normal legacy state, not evidence that the records lack value.

## Migration Plan

- Add `.polaris/citations.jsonl` lazily when the first citation is recorded.
- Treat missing citation files as empty citation history.
- Treat records without lineage fields as having no inherited citations.
- Preserve existing memory and replacement history formats through optional fields.

## Open Questions

None.
