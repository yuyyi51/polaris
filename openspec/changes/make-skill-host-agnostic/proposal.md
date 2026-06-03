## Why

Polaris 当前的 skill 仅放在 `skills/codex/polaris/`，spec capability 名 `polaris-codex-skill` 也按 host 命名。但 `SKILL.md` 的内容（除两处 "Codex" 字眼外）完全 host-agnostic，frontmatter 形式（`name` + `description`）也是 Codex / TraeCLI(Coco) / Claude 等 markdown skill host 共用的标准。继续按 host 分目录会强制每接入一个新 host 就拷一份相同内容，与 oh-my-code 单源 skill 的做法不一致，也违背 Polaris 已经做了的 hook host-agnostic 改造（`--target` 模型）。把 skill 改成 host-agnostic 单源，可与 hook 层的多 host 适配保持架构一致，并降低后续维护成本。

## What Changes

- **BREAKING**：把 `skills/codex/polaris/` 重命名为 `skills/polaris/`，capability 从 `polaris-codex-skill` 重命名为 `polaris-skill`。
- 修改 `skills/polaris/SKILL.md`：
  - frontmatter `description` 中的 "long-running Codex tasks" 改为 host-agnostic 表述（如 "long-running agent tasks"）。
  - 正文首段 "remind Codex to reload" 改为 "remind the agent to reload"。
- 更新 `README.md` "Codex Skill" 段落：
  - 段落标题改为 host-agnostic（如 "Polaris Skill"）。
  - 安装命令同时给出 Codex 与 TraeCLI/Coco 两个目标路径示例。
- 不修改任何代码（`src/`、`tests/`）；不影响 hook 行为；不影响 `.polaris/` 存储。
- 不保留 `skills/codex/polaris/` 旧路径（仓库直接重命名；用户拷贝路径在 README 已说明）。

## Capabilities

### New Capabilities
- `polaris-skill`: Host-agnostic Polaris skill bundle and its on-disk shape, frontmatter, and content guidance.

### Modified Capabilities
（无）—— 通过 `RENAMED Requirements` / 重命名 capability 完成；旧 capability `polaris-codex-skill` 在归档时被新 capability 取代。

## Impact

- 文件：`skills/codex/polaris/SKILL.md` 重命名/移动到 `skills/polaris/SKILL.md`，并修改两处文案。
- 文档：`README.md` "Codex Skill" 段落重写为 host-agnostic。
- Spec：归档此 change 时，新建 `openspec/specs/polaris-skill/spec.md`，废弃 `openspec/specs/polaris-codex-skill/spec.md`（capability 重命名）。
- 向后兼容：用户已安装的 Codex skill 仍能用（拷贝过去的内容仍合法），但仓库源目录路径变了；README 提示新路径。
- 不影响：hook 行为、`--target` 参数、CLI 子命令、cargo 测试、`.polaris/` 存储。
