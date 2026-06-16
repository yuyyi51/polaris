## 1. Citation Storage Model

- [x] 1.1 Add citation event data structures and `.polaris/citations.jsonl` path handling.
- [x] 1.2 Implement citation JSONL loading with blank-line tolerance and malformed-line diagnostics.
- [x] 1.3 Implement append-only citation recording with memory locking and concurrent write safety.
- [x] 1.4 Treat a missing citation file as empty citation history.

## 2. Citation CLI

- [x] 2.1 Add `polaris cite` command arguments for `--key`, `--keys`, `--id`, `--ids`, and `--quiet`.
- [x] 2.2 Validate selector conflicts and reject broad selectors by omission from the CLI surface.
- [x] 2.3 Resolve key and id selectors to active records, recording matched citations and reporting missing selectors for multi-select commands.
- [x] 2.4 Return an error when a single selector is missing or when no multi-selector target matches.
- [x] 2.5 Ensure citation does not mutate memory text, lifecycle, `created_at`, or `updated_at`.

## 3. Citation Summaries And Lineage

- [x] 3.1 Add direct citation count and last cited timestamp to `polaris list --json` summaries.
- [x] 3.2 Add inherited citation count and lineage source metadata to summaries.
- [x] 3.3 Preserve replacement citation lineage through existing `replaced_from` metadata.
- [x] 3.4 Preserve merge source record ids on merged targets and compute inherited citations from those sources.
- [x] 3.5 De-duplicate inherited citation counts across overlapping lineage paths.

## 4. Maintenance Context

- [x] 4.1 Extend prune suggestion JSON prompt context with citation summaries for candidate records.
- [x] 4.2 Extend compact suggestion JSON prompt context with citation summaries for source records.
- [x] 4.3 Update built-in maintenance prompts to frame citation counts as advisory review context only.

## 5. Skill And Documentation

- [x] 5.1 Update README command examples and guidance for `recall`, `touch`, and `cite` semantics.
- [x] 5.2 Update `skills/codex/polaris/SKILL.md` with citation guidance, batching examples, and advisory maintenance language.
- [x] 5.3 Add skill/spec tests or existing validation coverage for the new citation guidance.

## 6. Verification

- [x] 6.1 Add CLI tests for citation recording by key/id, multi-selector behavior, quiet output, and missing selector handling.
- [x] 6.2 Add CLI tests for citation summary metadata, replacement inheritance, merge inheritance, and de-duplication.
- [x] 6.3 Add CLI tests for citation JSONL malformed-line diagnostics and concurrent citation recording.
- [x] 6.4 Run `openspec validate add-memory-citation-signals --strict`.
- [x] 6.5 Run `cargo fmt --check`.
- [x] 6.6 Run `cargo test`.
- [x] 6.7 Run `cargo clippy`.
