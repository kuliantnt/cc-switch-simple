简体中文 | [English](./README.en.md)

---

# cc-switch

> 本项目专注 Claude Code 配置切换。Codex 已支持 config.toml profiles，建议直接使用 `codex --profile <name>`。

`cc-switch` 是一个在 WSL/Linux 下切换 Claude Code 全局配置的 Bash 小工具。

它只做一件事：在本机安全地切换 `~/.claude/settings.json`，并管理常用的 profile 和备份。

管理路径：

- 当前配置：`~/.claude/settings.json`
- profile：`~/.claude/profiles/*.json`
- 备份：`~/.claude/backups/settings-YYYYmmdd-HHMMSS.json`

## 安装与卸载

### 安装

在仓库根目录执行：

```bash
chmod +x cc-switch install.sh
./install.sh
```

安装脚本会执行这些动作：

- 把命令安装到 `~/.local/bin/cc-switch`
- 把 `zsh` 补全安装到 `~/.zsh/completions/_cc-switch`
- 把仓库内的 `profiles/*.template.json` 复制到 `~/.claude/profiles/`

如果 `~/.local/bin` 不在 `PATH` 中，把下面这行加入 shell 配置：

```bash
export PATH="$HOME/.local/bin:$PATH"
```

### Zsh 补全

如果你使用 `zsh`，确认 `~/.zshrc` 在 `compinit` 之前包含：

```bash
fpath=("$HOME/.zsh/completions" $fpath)
autoload -Uz compinit
compinit
```

如果已经有 `autoload -Uz compinit` 和 `compinit`，不要重复添加，只要确保 `fpath=...` 在前面。

### 卸载

```bash
./install.sh uninstall
```

卸载会：

- 删除已安装的命令
- 删除已安装的 `zsh` 补全
- 删除由模板安装且未被修改过的 profile

卸载不会删除：

- `~/.claude/settings.json`
- 备份文件
- 你修改过的 profile

### 模板文件

仓库跟踪的是模板文件：

- `profiles/official.template.json`
- `profiles/deepseek.template.json`
- `profiles/local-test.template.json`

安装时会自动去掉 `.template` 后缀，例如 `deepseek.template.json` 会安装成 `deepseek.json`。

## 命令说明

### 默认行为

```bash
cc-switch
```

无参数时显示帮助，不会修改当前配置。切换到下一个 profile 请显式使用：

```bash
cc-switch next
```

`next` 会按文件名排序读取 `~/.claude/profiles/*.json`，判断当前 `~/.claude/settings.json` 是否和某个 profile 内容一致，然后切到下一个；如果已经是最后一个则循环回第一个；如果当前配置无法识别或 `settings.json` 还不存在，则直接切到第一个 profile。

### 命令列表

```bash
cc-switch list
```

列出 `~/.claude/profiles/` 下可用的 profile，并用 `*` 标记当前使用中的配置。

```bash
cc-switch next
```

显式切换到下一个 profile。命令会先打印修改前的 profile，再打印修改后的 profile。若只有一个 profile，仍会切换到该唯一 profile，并给出提示。

```bash
cc-switch use deepseek
```

校验 `deepseek.json` 的 JSON 格式，备份当前 `settings.json`，然后切换到该 profile。

```bash
cc-switch current
cc-switch current --show-token
```

打印当前 `settings.json` 内容，并尽量识别它对应哪个已保存的 profile。默认会 mask `ANTHROPIC_AUTH_TOKEN`；只有 `--show-token` 才显示完整值。

```bash
cc-switch backup
```

为当前配置创建一个带时间戳的备份文件。

```bash
cc-switch edit deepseek
```

用 `$EDITOR` 打开指定 profile；如果没有设置 `$EDITOR`，默认使用 `nano`。

```bash
cc-switch new my-profile
```

把当前 `settings.json` 复制为一个新的 profile，例如 `my-profile.json`。

```bash
cc-switch restore settings-20260518-142604.json
```

把指定备份文件恢复为当前 `settings.json`。如果当前已有配置，会先自动再备份一次。

```bash
cc-switch help
cc-switch -h
cc-switch --help
```

显示帮助。无参数执行 `cc-switch` 时也会显示同样的帮助。

### 手动测试

建议至少准备两个内容不同的 profile，然后执行：

```bash
cc-switch list
cc-switch current
cc-switch next
cc-switch next
cc-switch use deepseek
cc-switch
```

重点确认：

- `list` 的顺序是否按文件名排序
- `cc-switch` 是否稳定输出帮助且不改写当前配置
- `next` 是否按顺序循环切换
- `current` 识别出的 profile 是否与刚切换到的目标一致
- 当前配置不匹配任何 profile 时，是否显示 `Current: unknown` 并切到第一个 profile

## 设计说明、安全规则与排错

### 设计取向

`cc-switch` 的范围刻意很小：

- 只处理本机的 `settings.json` 切换
- 用纯 Bash 实现，便于审计和改动
- 不引入后台服务、面板或额外构建步骤

如果你已经有几份 `settings.json`，只想快速、安全地来回切换，这个项目更合适；如果你需要更重的多 provider 管理能力，应该用范围更大的工具。

### 安全与校验

- 如果目标 profile 不是合法 JSON，会拒绝切换
- 优先使用 `jq` 校验；若系统没有 `jq`，回退到 `python3 -m json.tool`
- `use` 和 `restore` 都会先备份当前配置
- 默认只保留最近 `10` 份备份，可通过 `BACKUP_KEEP_COUNT` 调整
- `current` 默认会 mask `ANTHROPIC_AUTH_TOKEN`
- profile 名只允许字母、数字、`.`、`_`、`-`，且不能以 `.` 开头
- 写入 `settings.json` 时使用同目录临时文件再 `mv`，并将权限设为 `600`
- 模板 profile 里的 token 仍可能是占位值，切换前会给出警告

### 本地 profile 管理

仓库里的 `profiles/.gitignore` 会忽略本地 `profiles/*.json`，所以你可以把个人 profile 放在仓库里的 `profiles/` 下使用，而不会误提交。

### Zsh 补全排错

如果 `Tab` 只补全普通文件名，通常是 `fpath` 加载顺序不对，或者 `zsh` 还在使用旧缓存。可按顺序检查：

```bash
which cc-switch
cc-switch list
ls ~/.zsh/completions/_cc-switch
head -5 ~/.zsh/completions/_cc-switch
rm -f ~/.zcompdump*
exec zsh
```

然后重新测试：

```bash
cc-switch <Tab>
cc-switch next <Tab>
cc-switch use <Tab>
cc-switch edit <Tab>
cc-switch restore <Tab>
```
