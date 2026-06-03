## REMOVED Requirements

### Requirement: Project-bundled Codex skill
**Reason**: Replaced by host-agnostic `polaris-skill` capability. Polaris skill source moves to `skills/polaris/` and serves any markdown-skill host (Codex, TraeCLI/Coco, …).
**Migration**: See new requirement `Project-bundled Polaris skill` under capability `polaris-skill`.

### Requirement: Polaris recovery instruction
**Reason**: Reissued under `polaris-skill` so the recall guidance is no longer host-tagged. Behavior is unchanged.
**Migration**: See new requirement `Polaris recovery instruction` under capability `polaris-skill`.

### Requirement: Start and resume workflow guidance
**Reason**: Reissued under `polaris-skill` (same scenarios, no host-specific language).
**Migration**: See new requirement `Start and resume workflow guidance` under capability `polaris-skill`.

### Requirement: Memory recording guidance
**Reason**: Reissued under `polaris-skill` (same scenarios, no host-specific language).
**Migration**: See new requirement `Memory recording guidance` under capability `polaris-skill`.

### Requirement: Skill validation
**Reason**: Validation of the bundled skill is covered by repo-level verification commands and does not need a separate spec requirement once the skill is host-agnostic.
**Migration**: Run `cargo fmt --check`, `cargo test`, `cargo clippy`, and `openspec validate make-skill-host-agnostic --strict` per `AGENTS.md` Verification section. README continues to point to the bundled source.
