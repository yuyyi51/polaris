## 1. Installer Script

- [x] 1.1 Add `scripts/install-codex-skill.sh` to install the bundled `skills/codex/polaris/` folder.
- [x] 1.2 Support `POLARIS_CODEX_SKILLS_DIR` and `CODEX_HOME` destination selection.
- [x] 1.3 Make reinstall behavior replace only the destination `polaris` skill directory.

## 2. Documentation

- [x] 2.1 Update README skill installation guidance to prefer the script.
- [x] 2.2 Keep the manual copy command documented as a fallback.

## 3. Tests

- [x] 3.1 Add an integration test that runs the script with an overridden destination.
- [x] 3.2 Assert the installed `SKILL.md` matches the bundled repository skill.
- [x] 3.3 Assert reinstall replaces stale destination content.

## 4. Validation

- [x] 4.1 Run OpenSpec validation/status for `add-skill-install-script`.
- [x] 4.2 Run `cargo fmt --check`.
- [x] 4.3 Run `cargo test`.
- [x] 4.4 Run `cargo clippy`.
