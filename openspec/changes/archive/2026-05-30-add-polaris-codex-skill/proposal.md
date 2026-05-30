## Why

Polaris can preserve context, but users still need a reliable way to teach Codex agents when and how to use it. A project-bundled Codex skill lets users copy the guidance into their own skills directory without rediscovering the workflow.

## What Changes

- Add a Codex-compatible Polaris skill under `skills/codex/polaris/`.
- Keep the skill focused on agent behavior: when to initialize Polaris, what to remember, when to recall, how to handle stale memory, and how to respond to compact-session hook reminders.
- Include only essential skill files so the folder can be copied directly into a user's Codex skills directory.
- Add validation steps to ensure the skill has valid frontmatter and matches the implemented Polaris CLI.

## Capabilities

### New Capabilities
- `polaris-codex-skill`: Project-bundled Codex skill that teaches agents how to use Polaris safely and effectively.

### Modified Capabilities
- None.

## Impact

- Affected files: new skill assets under `skills/codex/polaris/`.
- Affected users: Polaris users who want to install the guidance into their Codex environment.
- Affected behavior: no runtime CLI behavior changes are expected.
- Dependencies: validation may use the existing Codex skill-creator validation scripts if available.
