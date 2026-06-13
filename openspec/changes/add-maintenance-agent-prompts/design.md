## Context

`polaris prune --suggest` and `polaris compact --suggest` currently emit deterministic maintenance candidates based on shallow local rules. That is useful for finding records to inspect, but the rules cannot reliably decide whether a memory is still relevant, whether it conflicts with current work, or whether a group should be merged. Polaris already has a TOML configuration pattern for hook prompts, with workspace config taking precedence over user config.

## Goals / Non-Goals

**Goals:**

- Include an agent-facing review prompt by default in prune and compact suggestion output.
- Keep prune and compact prompts separate because deletion review and consolidation review have different workflows.
- Allow workspace and user configuration to override the default prompt text.
- Render prompt templates with `{{suggestions}}`, `{{status}}`, and `{{keys}}` placeholders.
- Keep suggestions read-only and avoid inlining memory body content by default.
- Change JSON output to an envelope because this version has not been distributed yet.

**Non-Goals:**

- No LLM integration or semantic scoring inside Polaris.
- No automatic deletion, merge apply, lifecycle change, or rename based on prompts.
- No `{{recall}}` placeholder in maintenance prompts in this change.
- No profile-specific prompt selection beyond workspace/user/default precedence.

## Decisions

1. Add `[maintenance.prompts]` to Polaris TOML config.

   Reusing `.polaris/config.toml` and `~/.polaris/config.toml` keeps configuration in the existing locations. The new keys should be:

   - `prune`: prompt template for prune suggestion review.
   - `compact`: prompt template for compact suggestion review.

   Workspace config takes precedence over user config as a whole, matching hook prompt behavior. If a selected config file omits one maintenance prompt, Polaris should use the built-in default for that prompt.

2. Render a prompt for every suggest command by default.

   Human output should append an `Agent prompt:` section after suggestions or after the no-suggestions message. JSON output should include `prompt` in the top-level response. This makes the feature useful without requiring agents to discover another flag.

3. Use `{{suggestions}}`, `{{status}}`, and `{{keys}}` placeholders only.

   `{{suggestions}}` gives the agent the candidate list and reasons. `{{status}}` gives machine-readable workspace counts and lifecycle summaries. `{{keys}}` gives the set of keyed inline memories available for targeted recall. Polaris should not support `{{recall}}` for maintenance prompts in this change because full recall can be large and may expose content before the agent has chosen a narrow inspection path.

4. Change JSON suggest output from a bare array to an envelope.

   The JSON shape should be:

   ```json
   {
     "suggestions": [],
     "prompt": "...",
     "prompt_context": {
       "status": {},
       "keys": []
     }
   }
   ```

   This is a breaking change, but the project has not been distributed yet and the envelope is the cleaner long-term contract.

5. Keep default prompts conservative.

   Built-in prune prompts should instruct agents to inspect candidates with `polaris recall --key`, preserve durable facts unless clearly superseded, and only forget stale or redundant records. Built-in compact prompts should instruct agents to recall source keys, verify coherence, create a merge draft, edit it, apply it, and use `--forget-sources` only when the merged target fully supersedes sources.

## Risks / Trade-offs

- Breaking JSON consumers -> Acceptable before distribution; document the new envelope in README and skill guidance.
- Prompt output can be verbose -> Default prompts are intentionally operational, and JSON consumers can choose the fields they need.
- Config file precedence may surprise users who expect merging -> Match existing hook prompt behavior and document that workspace config takes precedence while missing prompt keys fall back to defaults.
- Prompts may be treated as permission to mutate -> Default text must explicitly say suggestions are heuristic candidates and destructive commands require deliberate confirmation.
