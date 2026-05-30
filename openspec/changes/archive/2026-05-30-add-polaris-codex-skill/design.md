## Context

Polaris now has a working CLI and a finalized `polaris-recall` specification, but the agent-facing usage workflow still lives mostly in project docs and conversation history. Users need a copyable Codex skill that teaches agents to use Polaris consistently in their own environments.

The skill will live inside the repository at `skills/codex/polaris/` so it can be versioned with Polaris and copied into a user's Codex skills directory. The system skill-creator guidance marks `SKILL.md` as required and `agents/openai.yaml` as optional/recommended. Polaris will choose the minimum copyable form and ship only `SKILL.md`.

## Goals / Non-Goals

**Goals:**
- Provide a Codex-compatible Polaris skill as a project artifact.
- Teach agents the practical Polaris workflow: status, init, remember, note creation, recall, clear, and compact-session recovery.
- Make compact-session recovery unambiguous: if context says to run `polaris recall`, run it before other work.
- Keep the skill concise enough to load comfortably in Codex.
- Validate the skill folder with the available skill-creator validation script.

**Non-Goals:**
- Do not change Polaris CLI runtime behavior.
- Do not install the skill into the user's global Codex skills directory automatically.
- Do not add `agents/openai.yaml`, scripts, references, README files, or other auxiliary skill resources unless a future change explicitly asks for them.
- Do not make the skill depend on project-specific OpenSpec workflows.

## Decisions

### Store the skill under `skills/codex/polaris/`

The repository will include:

```text
skills/
  codex/
    polaris/
      SKILL.md
```

`SKILL.md` is the portable skill body and the only required skill file. The project will not include `agents/openai.yaml` because the intended installation path is a simple folder copy, and the agent-facing trigger/usage metadata belongs in `SKILL.md` frontmatter.

Alternatives considered:
- Include `agents/openai.yaml`: useful for richer UI metadata, but unnecessary for Codex to discover and run the skill.
- Store only a raw markdown snippet in README: easier to read, but users would still need to convert it into a valid skill.

### Keep guidance command-oriented

The skill should instruct agents to run concrete Polaris commands instead of merely describing concepts. Required commands are:

- `polaris status --json`
- `polaris init`
- `polaris remember --text ...`
- `polaris remember --stdin ...`
- `polaris note create --title ...`
- `polaris recall`
- `polaris clear --yes`

Alternative considered: include a longer conceptual guide. That would be less useful during compact recovery, where short imperative guidance matters most.

### Treat recall reminders as highest priority

The skill must contain a top-level rule: when the current context instructs the agent to run `polaris recall`, do so immediately before other work. This mirrors the Polaris hook contract and reduces the chance that the agent acknowledges the reminder but forgets to recall.

### Generate and validate like a real skill

Implementation may use `skill-creator`'s `init_skill.py` if practical, but any optional generated resources must be removed. Validation should run `quick_validate.py` against `skills/codex/polaris/`.

Alternative considered: create files by hand. That is acceptable as a fallback, but using the generator reduces the chance of invalid metadata or folder layout.

## Risks / Trade-offs

- Skill may become too verbose -> Keep only procedural guidance and concrete examples in `SKILL.md`.
- Skill may over-record sensitive data -> Include a clear instruction not to store secrets, tokens, passwords, or credentials.
- Lack of UI metadata may make the skill less polished in skill lists -> Accept this in favor of a minimum portable package.
- Polaris may not be installed in a user's environment -> Instruct agents to report command failures clearly rather than pretending memory was recovered.
