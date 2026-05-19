简体中文 | [English](./README.en.md)

---

# cc-switch

> 本项目专注 Claude Code 配置切换。Codex 已支持 `config.toml` profiles，建议优先使用 `codex --profile <name>`。

`cc-switch` 是一个运行在 WSL/Linux 上的 Bash 小工具，用来安全切换 Claude Code 的全局配置文件 `~/.claude/settings.json`。

它只做一件事：

- 在多个 profile 之间切换当前配置
- 在切换前自动备份
- 帮你管理常用的本地 profile

如果你只是想在几份 `settings.json` 之间快速来回切换，这个工具会比较顺手。

## 管理哪些文件

- 当前配置：`~/.claude/settings.json`
- profile：`~/.claude/profiles/*.json`
- 备份：`~/.claude/backups/settings-YYYYmmdd-HHMMSS.json`

## 快速开始

先获取仓库并进入目录：

```bash
git clone https://github.com/kuliantnt/cc-switch-simple.git
cd cc-switch-simple
```

然后执行安装：

```bash
chmod +x cc-switch install.sh
./install.sh
```

然后试一下：

```bash
cc-switch list
cc-switch current
cc-switch next
```

如果提示找不到 `cc-switch`，把下面这行加入你的 shell 配置：

```bash
export PATH="$HOME/.local/bin:$PATH"
```

## 安装说明

以下命令都默认在仓库根目录执行。

安装脚本会自动完成这些事：

- 把命令安装到 `~/.local/bin/cc-switch`
- 把 `zsh` 补全安装到 `~/.zsh/completions/_cc-switch`
- 把仓库中的 `profiles/*.template.json` 复制到 `~/.claude/profiles/`

### 模板文件

仓库跟踪的是模板文件：

- `profiles/official.template.json`
- `profiles/deepseek.template.json`
- `profiles/local-test.template.json`

安装时会自动去掉 `.template` 后缀。例如：

- `deepseek.template.json` -> `deepseek.json`

### Zsh 补全

如果你使用 `zsh`，请确认 `~/.zshrc` 在 `compinit` 之前包含：

```bash
fpath=("$HOME/.zsh/completions" $fpath)
autoload -Uz compinit
compinit
```

如果你已经有 `autoload -Uz compinit` 和 `compinit`，不要重复添加；只需要保证 `fpath=...` 在前面。

## 卸载

```bash
./install.sh uninstall
```

卸载会删除：

- 已安装的命令
- 已安装的 `zsh` 补全
- 由模板安装且尚未修改的 profile

卸载不会删除：

- `~/.claude/settings.json`
- 备份文件
- 你手动修改过的 profile

## 常用命令

### 查看可用 profile

```bash
cc-switch list
```

列出 `~/.claude/profiles/` 下的 profile，并用 `*` 标记当前正在使用的配置。

### 查看当前配置

```bash
cc-switch current
cc-switch current --show-token
```

显示当前 `settings.json` 内容，并尽量识别它对应哪个已保存的 profile。

- 默认会隐藏 `ANTHROPIC_AUTH_TOKEN`
- 只有 `--show-token` 才会显示完整值

### 切换到下一个 profile

```bash
cc-switch next
```

`next` 会按文件名排序读取 `~/.claude/profiles/*.json`，然后：

- 如果当前配置能匹配某个 profile，就切到下一个
- 如果当前已经是最后一个，就循环回第一个
- 如果当前配置无法识别，或 `settings.json` 还不存在，就直接切到第一个

### 切换到指定 profile

```bash
cc-switch use deepseek
```

执行时会：

- 校验 `deepseek.json` 是否是合法 JSON
- 备份当前 `settings.json`
- 再切换到目标 profile

### 新建 profile

```bash
cc-switch new my-profile
```

把当前 `settings.json` 保存成一个新的 profile，例如 `my-profile.json`。

### 编辑 profile

```bash
cc-switch edit deepseek
```

使用 `$EDITOR` 打开指定 profile；如果没有设置 `$EDITOR`，默认使用 `nano`。

### 备份当前配置

```bash
cc-switch backup
```

为当前配置创建一个带时间戳的备份文件。

### 恢复备份

```bash
cc-switch restore settings-20260518-142604.json
```

把指定备份恢复成当前 `settings.json`。如果当前已有配置，会先自动再备份一次。

### 显示帮助

```bash
cc-switch
cc-switch help
cc-switch -h
cc-switch --help
```

无参数执行 `cc-switch` 时，只会显示帮助，不会修改当前配置。

## 建议这样使用

一个很顺手的流程通常是：

1. 先准备两份或以上 profile
2. 用 `cc-switch list` 确认它们都在
3. 用 `cc-switch current` 看当前是哪一份
4. 用 `cc-switch next` 或 `cc-switch use <name>` 切换
5. 如果当前配置值得保留，用 `cc-switch new <name>` 存成 profile

## 安全与校验

`cc-switch` 默认会尽量保守地操作你的配置：

- 目标 profile 不是合法 JSON 时，会拒绝切换
- 优先使用 `jq` 校验 JSON；若系统没有 `jq`，会回退到 `python3 -m json.tool`
- `use` 和 `restore` 都会先备份当前配置
- 默认只保留最近 `10` 份备份，可通过 `BACKUP_KEEP_COUNT` 调整
- 写入 `settings.json` 时会先写同目录临时文件，再 `mv` 替换
- 写入后的 `settings.json` 权限会设为 `600`
- profile 名只允许字母、数字、`.`、`_`、`-`，且不能以 `.` 开头
- 模板 profile 中的 token 可能仍是占位值，切换前会给出警告

## 本地 profile 管理

仓库中的 `profiles/.gitignore` 会忽略本地 `profiles/*.json`，所以你可以把个人 profile 放在仓库里的 `profiles/` 目录中使用，而不会误提交。

## 手动自测

建议至少准备两份内容不同的 profile，然后执行：

```bash
cc-switch list
cc-switch current
cc-switch next
cc-switch next
cc-switch use deepseek
cc-switch
```

重点检查：

- `list` 是否按文件名排序
- `cc-switch` 无参数时是否稳定显示帮助且不改写当前配置
- `next` 是否会按顺序循环切换
- `current` 识别出的 profile 是否正确
- 当前配置不匹配任何 profile 时，是否显示 `Current: unknown` 并切到第一个 profile

## Zsh 补全排错

如果按 `Tab` 只补全普通文件名，通常是 `fpath` 顺序不对，或者 `zsh` 仍在使用旧缓存。可以依次检查：

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

## 设计取向

`cc-switch` 的范围刻意保持很小：

- 只处理本机的 `settings.json` 切换
- 用纯 Bash 实现，便于审计和修改
- 不引入后台服务、图形面板或额外构建步骤

如果你要的是“简单、直接、可控”的配置切换，它比较合适；如果你需要更完整的多 provider 管理能力，就更适合使用范围更大的工具。
