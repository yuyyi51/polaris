## Why

Polaris 当前的 hook 命令只输出 Codex 期望的 JSON 形态（`hookSpecificOutput.hookEventName` 包装 + `SessionStart`/`PostCompact`/`PostToolUse` 事件名），无法直接被 TraeCLI（Coco）host 消费。TraeCLI 在 `post_compact` 事件上有更直接的压缩恢复触发点，并且其 stdout schema 不需要 `hookEventName` 包装。增加一个 `--target` 参数让同一个二进制按 host 切换 stdin/stdout 适配层，是接入 TraeCLI 同时保持 Codex 行为不变的最小代价做法。

## What Changes

- 给 `polaris hook session-start` / `polaris hook post-compact` / `polaris hook post-tool-use` 三个子命令各增加一个 `--target <host>` 参数，取值为 `codex`（默认）或 `traecli`。
- TraeCLI 的 stdin schema 与 Codex 在公共字段（`cwd`、`hook_event_name`）上一致，解析层共用即可；stdout 在 `--target traecli` 下不再嵌套 `hookEventName`，只输出 `{"hookSpecificOutput": {"additionalContext": "..."}}`。
- TraeCLI 没有 Codex 的 `SessionStart`+`matcher=compact` 组合；`post_compact` 是 TraeCLI 上的主召回入口。`polaris hook post-compact --target traecli` 直接渲染 recall 提示到 stdout（不再像 Codex fallback 那样写 pending 状态文件）。`polaris hook session-start --target traecli` 与 `polaris hook post-tool-use --target traecli` 在 TraeCLI 上保留为静默 noop，便于将来扩展。
- 默认值 `--target codex` 保持现有 Codex hook 行为字节级一致，所有现有 `examples/codex-hooks/*.json` 配置无需改动。
- 新增 `examples/traecli-hooks/hooks.json`，注册 `PostCompact` 事件运行 `polaris hook post-compact --target traecli`。
- README 增加 TraeCLI 集成段落，引用新示例与 `--target` 参数。

## Capabilities

### New Capabilities
（无）

### Modified Capabilities
- `polaris-recall`: 新增 `--target` 参数语义；新增 TraeCLI host 的 `post-compact` 压缩恢复 hook 行为；新增 TraeCLI 示例配置。Codex 现有行为不变。

## Impact

- 代码：`src/cli.rs`（hook 子命令增加 `--target`）、`src/hook.rs`（拆分 host-agnostic 渲染层 + host adapter）。
- 测试：`tests/cli.rs` 增加 TraeCLI target 与默认 Codex target 的端到端用例。
- 文档与示例：新增 `examples/traecli-hooks/hooks.json`，更新 `README.md`。
- 向后兼容：`--target codex` 是默认值，现有 Codex 用户的 hook 配置无需改动。
- 不影响：`.polaris/` 存储结构、`polaris remember/forget/recall/clear/note` 等子命令、`.polaris/config.toml` 的 `[hooks].recall_prompt` 与 `{{recall}}` 占位符（被 host adapter 共用）。
