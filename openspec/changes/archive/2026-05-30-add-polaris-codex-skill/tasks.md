## 1. Skill Scaffold

- [x] 1.1 Create `skills/codex/polaris/`.
- [x] 1.2 Ensure the skill folder contains `SKILL.md`.
- [x] 1.3 Ensure the skill folder does not contain `agents/openai.yaml` or generated placeholder resources.

## 2. Skill Content

- [x] 2.1 Write `SKILL.md` frontmatter with `name: polaris` and trigger-focused description text.
- [x] 2.2 Add the recall-first rule for compact-session reminders.
- [x] 2.3 Add start/resume workflow guidance for `polaris status --json`, `polaris init`, `polaris recall`, and stale memory handling.
- [x] 2.4 Add memory recording guidance for `polaris remember --text`, `polaris remember --stdin`, and `polaris note create --title`.
- [x] 2.5 Add safety guidance that agents must not store secrets, tokens, passwords, credentials, or private keys in Polaris.
- [x] 2.6 Add failure guidance for unavailable or uninitialized Polaris commands.

## 3. Documentation

- [x] 3.1 Update project documentation to mention the bundled skill location and copy/install intent.
- [x] 3.2 Document that the bundled skill is intentionally minimal and does not require `agents/openai.yaml`.

## 4. Verification

- [x] 4.1 Run the skill-creator quick validation script against `skills/codex/polaris/`.
- [x] 4.2 Inspect the skill content against the Polaris CLI command surface and the `polaris-recall` spec.
- [x] 4.3 Run existing project checks: `cargo fmt -- --check`, `cargo test`, and `cargo clippy -- -D warnings`.
- [x] 4.4 Run `openspec status --change add-polaris-codex-skill` and confirm the change remains apply-ready.
