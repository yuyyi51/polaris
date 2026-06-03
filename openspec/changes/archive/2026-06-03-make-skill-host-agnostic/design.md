## Context

Polaris 仓库当前结构：`skills/codex/polaris/SKILL.md` 是给 Codex 拷贝安装的 skill 包；spec 名 `polaris-codex-skill`。SKILL.md 的 frontmatter 用 `name` + `description`（CommonMark + YAML 风格），是 Codex / TraeCLI(Coco) / Claude 等 markdown 风格 skill host 的共用格式。SKILL.md 全文除 frontmatter `description` 中 "long-running Codex tasks" 与正文首段 "remind Codex to reload" 两处之外，其余都不带 host 字样，直接对 Codex / TraeCLI 都成立。

参考资料：
- oh-my-code 仓库（`liuhaoyang.qz/oh-my-code`）的 skill 包是单一源，多 host 共用，host 区分集中在 hook adapter 与安装路径。
- 当前 Polaris hook 已经走 `--target` 单二进制 + host-agnostic 核心（参见已归档的 `add-traecli-hook-target`），skill 层应保持同样的架构姿态。
- `AGENTS.md:13` 描述 `skills/codex/polaris/SKILL.md` 是"the copyable Codex skill bundled by this repo"——文件移动后这一行也需要更新。
- `AGENTS.md:21-22` 要求 CLI 小而显式、不存敏感数据；本 change 不触碰这些约束。

## Goals / Non-Goals

**Goals:**
- 把 Polaris skill 包从 `skills/codex/polaris/` 改为 host-agnostic 的 `skills/polaris/`，单一源服务所有 markdown-skill host。
- 把 SKILL.md 内容里仅有的两处 "Codex" 字眼改为 host-agnostic 表述，使 frontmatter description 与正文首段都不绑定具体 host。
- README 同时给出 Codex 与 TraeCLI/Coco 的安装目标路径，明确"一份源、按 host 拷"。
- spec capability 从 `polaris-codex-skill` 重命名为 `polaris-skill`，并把其中"Codex-only"的语言改为 host-agnostic。

**Non-Goals:**
- 不改 SKILL.md 的整体结构、命令示例、安全约束（除两处文案外其它不动）。
- 不改 hook 行为、`src/` 任何代码、`.polaris/` 存储格式。
- 不引入新的 host 适配机制（没有 `skills/<host>/`、没有模板系统）。
- 不删除其它 host 安装时已有的工作流；用户从 `skills/polaris/` 拷贝到任何 host 的 skill 目录都应直接生效。
- 不改 OpenSpec change `add-traecli-hook-target` 的归档结果。
- 不修改 `AGENTS.md` 之外的文档（如果 `AGENTS.md:13` 路径过期，仅同步该行；不做更大重写）。

## Decisions

### Decision 1: 单源 host-agnostic 目录 `skills/polaris/`，不保留旧路径

**选择**：用 `git mv skills/codex/polaris skills/polaris`，删掉 `skills/codex/` 这一层（它现在不再有意义）。

**理由**：
- Polaris 是单 skill 项目，加 host 前缀只是 oh-my-code 多产物仓库的副作用，对 Polaris 是冗余。
- 保留 `skills/codex/` + 新建 `skills/polaris/` 会让仓库存在两份易漂移的同名内容。
- README 是用户唯一的安装入口，已经显式说明拷贝命令；改一行命令路径即可，没有"用户记忆中的旧路径"问题。

**备选**：
- 保留 `skills/codex/polaris/` 不动，新增 `skills/traecli/polaris/`：被否，与"单源"目标矛盾。
- 用 symlink `skills/codex/polaris -> ../polaris` 兼容旧路径：被否，YAGNI 且增加 Windows 拷贝困难。

### Decision 2: capability rename 而不是 add+remove

**选择**：spec delta 用 `## RENAMED Requirements` + `## MODIFIED Requirements` 组合：把 capability `polaris-codex-skill` 重命名为 `polaris-skill`，再 MODIFY 涉及"Codex"措辞或 `skills/codex/polaris/` 路径的具体 Requirement / Scenario 到 host-agnostic 写法。

**理由**：
- 整个 capability 的语义没变（依然是"项目自带 Polaris skill 包"），只是不再绑定 Codex；rename 比"先删 polaris-codex-skill 再加 polaris-skill"语义更准。
- 让历史 archive (`add-polaris-codex-skill`) 继续有意义：它是当年加入此 capability 的来源；rename 不是否定它。
- OpenSpec 支持 RENAMED Requirements，这是 spec-driven 的常规演化方式。

**备选**：
- 直接 ADDED `polaris-skill` + REMOVED `polaris-codex-skill`：被否，语义模糊（不是新能力，是改名）。
- 不改 capability 名，只 MODIFY 内部 Scenario：被否，capability 名仍然误导（`polaris-codex-skill` 不再 Codex-only）。

### Decision 3: SKILL.md 改动最小化

**选择**：仅修改 frontmatter `description` 与正文首段两处带 "Codex" 字样的句子，其它字段（`name: polaris`、其它段落、命令示例）保持不变。

**理由**：
- `name: polaris` 已是 host-agnostic。
- 命令示例 `polaris recall` / `polaris init` / `polaris remember` 等都是 host-agnostic。
- 改动越少，归档时合并风险越低。

### Decision 4: README "Codex Skill" 段落改名而非新增

**选择**：把现有 "Codex Skill" 标题改为 "Polaris Skill"（或类似 host-agnostic 名），保留段落结构；安装命令块从单条改成两条（Codex + TraeCLI），其它表述按 host-agnostic 改写。

**理由**：
- 不增加新段落，保持 README 紧凑。
- 与已合入的 "TraeCLI Integration" 段落（hook 部分）形成呼应：hook 是 host-aware（`--target`），skill 是 host-agnostic（一份源 + 不同安装路径）。

## Risks / Trade-offs

- **[Risk] 用户已 fork 或脚本化引用 `skills/codex/polaris/` 路径** → Mitigation：README 明确写出新路径并简短说明此次重命名；CHANGELOG 类信息通过 commit message 体现。Polaris 还在早期，目录稳定性不是强承诺。
- **[Risk] capability rename 时 `openspec validate` 报漂移** → Mitigation：tasks.md 包含 `openspec validate make-skill-host-agnostic --strict` 步骤；`openspec` 工具支持 RENAMED Requirements，前一 change 的归档已经验证流程可行。
- **[Trade-off] AGENTS.md 第 13 行路径过期** → 计划在 tasks 中同步该行（仅一行），不做更大文档重写。
- **[Trade-off] 旧 capability `polaris-codex-skill` 在归档前仍存在于 `openspec/specs/`** → 这是 spec-driven 流程的正常状态；归档时由 RENAMED 操作处理，archive 后旧 capability 文件被替换为新 capability 文件。
