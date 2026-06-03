## Context

Polaris 现有 hook 实现假定 host 是 Codex：`session_start_output` 验证 `hook_event_name == "SessionStart"` 且 `source == "compact"`，并输出形如 `{"hookSpecificOutput": {"hookEventName": "SessionStart", "additionalContext": "..."}}` 的 JSON（`src/hook.rs:18-29, 41-58`）。`post_compact` 与 `post_tool_use_output` 也按 Codex schema 解析。`src/cli.rs:84-92` 通过 `HookCommand` 子命令枚举把每个事件分发到对应函数；hook 配置文件示例只覆盖 Codex（`examples/codex-hooks/`）。

新加的 host TraeCLI 在压缩恢复方向上有自己的事件模型：`post_compact` 是独立事件且会带 `compact_summary`；TraeCLI 的 stdout 不依赖 `hookEventName` 包装，`hookSpecificOutput.additionalContext` 字段直接被 host 注入下一轮模型上下文（来自 TraeCLI hooks 文档）。stdin 公共字段 `cwd`、`hook_event_name` 与 Codex 一致，因此解析层差异最小。

约束：
- `AGENTS.md:28` 要求 hook 不能内联 stored memory 内容，只在 `recall_prompt` 显式包含 `{{recall}}` 时才允许；这是 host-agnostic 的核心约束。
- `AGENTS.md:21` 要求 CLI 小而显式，倾向于增加聚焦的命令/参数而不是引入广义抽象。
- `AGENTS.md:25-26` 要求复用现有 `anyhow::Result` 错误风格、机器输出 JSON。

## Goals / Non-Goals

**Goals:**
- 同一个 `polaris` 二进制能同时服务 Codex 与 TraeCLI 两个 host，不破坏现有 Codex 行为。
- TraeCLI 用户能在 `post_compact` 事件上获得 Polaris 召回提示（与 Codex 的 `SessionStart`+`matcher=compact` 等价）。
- host 适配层结构清晰，未来增加第三个 host 只需增加一个 enum 分支与对应的 parse/render，不需要改 CLI 入口。

**Non-Goals:**
- 不利用 TraeCLI 的 `compact_summary` 字段（保持 host-agnostic 的最小化推进；用户已确认 Q2=A）。
- 不实现 TraeCLI 上的 fallback 机制（pending 状态文件 + post_tool_use 二次触发）。TraeCLI 在 `post_compact` 上即可一次性输出召回，没有 Codex 那样的 fallback 必要。
- 不修改存储层 `.polaris/state.json` / `.polaris/memories.jsonl` 结构。
- 不改变 `[hooks].recall_prompt` 与 `{{recall}}` 占位符的语义。
- 不引入 `POLARIS_HOOK_TARGET` 环境变量；`--target` 是显式 CLI 参数。
- 不修改 Codex hook 的 stdin/stdout 行为（默认 `--target codex` 字节级保持现状）。

## Decisions

### Decision 1: `--target` 放在 hook 子命令上，不放顶层

**选择**：在 `polaris hook session-start | post-compact | post-tool-use` 三个子命令各自加 `--target <host>` 参数，默认 `codex`。

**理由**：
- 顶层参数会污染所有子命令（`init`、`remember`、`recall` 都得重新解析），不符合 `AGENTS.md:21` "small and explicit" 的指导。
- 子命令级参数定位清晰，仅 hook 子命令需要 host 适配。
- 与 oh-my-code 仓库 `--target` 设计一致（参考方案：放在 `hook` / `mcp` 子命令上）。

**备选**：
- 顶层 `polaris --target X hook ...`：被否，理由如上。
- 新增 `hook traecli-post-compact` 等子命令（每个 host 一组）：被否，扩展成本随 host 数线性增长，且子命令名暗含 host，不如显式参数清晰。

### Decision 2: host 适配层用 `enum HookTarget` + match，不引入 trait object

**选择**：在 `src/hook.rs` 内增加 `enum HookTarget { Codex, Traecli }`（实现 `clap::ValueEnum` 与 `Default`），用 match 在 `session_start_output` / `post_compact` / `post_tool_use_output` 内部分发解析与渲染。

**理由**：
- host 数量稳定（短期内只有 codex/traecli），enum + match 零开销且可被编译器穷尽性检查。
- Polaris 现有错误风格 `anyhow::Result`、模块层次扁平，引入 trait + Box<dyn> 会让单元测试与显式 host 分支推理变难。
- oh-my-code 选 trait 是因为它是 TS 项目且 host 已经有 4 个并要继续扩展；Rust 项目在 host 数 ≤ 3 时 enum 更合适。

**备选**：
- `trait HookTarget` + 静态注册 `HashMap<&'static str, Box<dyn HookTarget>>`：被否，过度抽象。
- 完全独立的两组函数（`codex_session_start_output` / `traecli_session_start_output` …）：被否，重复代码且容易让安全约束（不内联 memory）漂移。

### Decision 3: TraeCLI 的 stdout 形态

**选择**：`--target traecli` 输出 `{"hookSpecificOutput": {"additionalContext": "..."}}`，**不**带 `hookEventName` 字段。Codex 现有输出（带 `hookEventName`）保持不变。

**理由**：
- TraeCLI hooks 文档（飞书 wiki Nqe4w36Aoi361gkUXyFcyE4GnNf）显示 `hookSpecificOutput.additionalContext` 是 TraeCLI 注入下一轮模型上下文的标准字段；`hookEventName` 是 Codex 的独有要求。
- 拆分为两种 `HookOutput` 变体后，复用 `recall_text` host-agnostic 的渲染（保持 `{{recall}}` 与不内联 memory 的安全约束在共享代码里）。

**实现细节**：将 `HookOutput` 改成 enum 或者两个变体；在 `cli.rs` 里按 target 选择序列化路径。

### Decision 4: TraeCLI 的事件映射

**选择**：

| Polaris 子命令 | `--target codex` | `--target traecli` |
|---|---|---|
| `hook session-start` | 现状（`SessionStart` + `source=compact`） | noop（静默退出，便于将来扩展） |
| `hook post-compact` | 现状（写 pending 状态文件） | 立即从 stdin `hook_event_name=PostCompact` 触发，**直接输出召回 JSON 到 stdout**（无 pending 状态） |
| `hook post-tool-use` | 现状（消费 pending 状态后输出召回） | noop |

**理由**：
- TraeCLI 的 `post_compact` 是独立事件，可以直接在事件触发时输出 `additionalContext`，不需要 Codex 那种"先记 pending 状态，等下一次 PostToolUse 再注入"的两段式 fallback。
- 让 TraeCLI 路径只用 1 个 hook 命令即可工作，配置最简单（`examples/traecli-hooks/hooks.json` 只挂 `PostCompact`）。
- Codex `post_compact` 写 pending 状态的语义跟 TraeCLI 不同，所以即使共享 `polaris hook post-compact` 子命令名，内部行为按 target 分支处理。
- TraeCLI 的 `session_start` / `post_tool_use` 暂保留 noop，以便未来需要时再扩展（向后兼容）。

### Decision 5: `--target` 错误处理

**选择**：`clap::ValueEnum` 自动校验未知值；缺值由 clap 报错；不接受环境变量覆盖。

**理由**：
- 与现有 `polaris` 其它 clap 解析行为一致。
- 显式参数避免环境变量造成的诊断困扰。

## Risks / Trade-offs

- **[Risk] 用户在 TraeCLI 上误用 `--target codex`** → Mitigation：示例 `examples/traecli-hooks/hooks.json` 显式带 `--target traecli`；README 在 TraeCLI 段落首行强调必须传 `--target traecli`。
- **[Risk] 未来 TraeCLI 修改 stdout schema** → Mitigation：host 渲染层是单一文件 `src/hook.rs` 内的一段函数，定位明确；测试断言 stdout 形态后可一次发现破坏。
- **[Risk] `--target` 命名歧义（用户可能期望 `--host`）** → Mitigation：与 oh-my-code 仓库术语一致（用户主动提到的参考），README 段落介绍。
- **[Trade-off] TraeCLI 路径不利用 `compact_summary`** → 短期内信息有损，但保持核心层 host-agnostic；用户已主动选择 Q2=A，未来如需要可单独提 change。
- **[Trade-off] enum + match 不支持运行时动态注册 host** → Polaris 不需要插件机制；host 列表是编译期已知。
