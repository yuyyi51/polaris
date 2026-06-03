## 1. Skill bundle move

- [x] 1.1 Run `git mv skills/codex/polaris skills/polaris` (preserving `SKILL.md`)
- [x] 1.2 Remove the now-empty `skills/codex/` directory if it has no other contents
- [x] 1.3 Edit `skills/polaris/SKILL.md` frontmatter `description`: replace "long-running Codex tasks" with host-agnostic phrasing such as "long-running agent tasks"; verify no other `Codex` token remains in `description`
- [x] 1.4 Edit `skills/polaris/SKILL.md` body first paragraph: replace "remind Codex to reload" with "remind the agent to reload"; verify no other host-specific phrasing remains in the body except generic command examples

## 2. README and AGENTS sync

- [x] 2.1 Update `README.md` "Codex Skill" section: rename heading to host-agnostic (e.g. "Polaris Skill"); reference `skills/polaris/` as the source; show install commands for both `~/.codex/skills/` and `~/.coco/skills/` (TraeCLI/Coco); state that the same folder is copied to any host
- [x] 2.2 Update `AGENTS.md` line 13 (or current line referencing `skills/codex/polaris/SKILL.md`) to point to the new path `skills/polaris/SKILL.md` and to describe the bundle as host-agnostic; do not rewrite other AGENTS.md content

## 3. Verification

- [x] 3.1 Run `cargo fmt --check`
- [x] 3.2 Run `cargo test`
- [x] 3.3 Run `cargo clippy --all-targets --all-features -- -D warnings`
- [x] 3.4 Run `openspec validate make-skill-host-agnostic --strict`
- [x] 3.5 Run `openspec status --change make-skill-host-agnostic` and confirm 4/4 artifacts complete
- [x] 3.6 Sanity-check the relocated skill: `ls skills/polaris/SKILL.md` exists and `grep -i 'codex' skills/polaris/SKILL.md` returns no host-specific narrative (the file may still contain incidental matches in command names — confirm any remaining matches are generic, not host-binding)
