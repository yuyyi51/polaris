## 1. Regression Coverage

- [x] 1.1 Add a CLI regression test for concurrent `remember` commands with distinct keys that verifies all successful records remain recallable and `memories.jsonl` has one JSON object per non-empty line.
- [x] 1.2 Add a CLI regression test for concurrent `remember --replace` commands that verifies replacements for unrelated keys are not lost.
- [x] 1.3 Add tests for appending after a memory file without a final newline and for ignoring blank JSONL lines.
- [x] 1.4 Add a malformed JSONL diagnostic test that asserts errors include `.polaris/memories.jsonl` and the physical line number.

## 2. Storage Locking

- [x] 2.1 Add a stable memory lock path under `.polaris/` and helper code for shared and exclusive file locks.
- [x] 2.2 Wrap mutating memory operations (`remember`, `forget`, `create_note`, and `clear`) in an exclusive lock covering each full logical operation.
- [x] 2.3 Wrap memory-loading read paths used by `recall`, `status --json`, and hook recall rendering in a shared lock.

## 3. JSONL Robustness

- [x] 3.1 Update append logic to inspect the existing file while locked and insert a separator newline before appending when the file is non-empty and lacks a final newline.
- [x] 3.2 Keep whitespace-only JSONL lines non-fatal when loading records.
- [x] 3.3 Add path and line-number context to JSON parse failures from `.polaris/memories.jsonl`.

## 4. Verification

- [x] 4.1 Run `openspec status --change fix-concurrent-memory-writes` and any relevant OpenSpec validation command.
- [x] 4.2 Run `cargo fmt --check`.
- [x] 4.3 Run `cargo test`.
- [x] 4.4 Run `cargo clippy`.
