# cc-switch

`cc-switch` 是一个用于在 WSL/Linux 下切换 Claude Code 全局配置的 Bash 小工具。

它管理的路径包括：

- 当前配置：`~/.claude/settings.json`
- 可复用 profiles：`~/.claude/profiles/*.json`
- 备份目录：`~/.claude/backups/settings-YYYYmmdd-HHMMSS.json`

## 和 `ccs` 的区别

截至 `2026-05-18`，GitHub 上较热门的同类项目之一是 [`kaitranntt/ccs`](https://github.com/kaitranntt/ccs)。

这个仓库和它的定位不同：

- `cc-switch` 只做一件事：在本机直接切换 `~/.claude/settings.json`
- `cc-switch` 是纯 Bash 脚本，零构建、零后台服务、零 Web 面板
- `cc-switch` 主要面向“我已经有几份 `settings.json`，只想安全备份后快速切换”这个场景
- `ccs` 更像完整的多 provider / 多 runtime 管理器，覆盖 Claude 账号切换、OAuth provider、代理、面板、远程能力等更重的功能
- 如果你需要的是小而透明、容易审计、方便自己改的脚本，这个项目更合适
- 如果你需要的是跨 provider 的统一入口和更完整的产品化能力，`ccs` 更合适

换句话说，这个项目不是要和 `ccs` 拼功能数量，而是刻意保持“单文件、可读、可改、可直接落地”。

## 文件说明

- `cc-switch`：主命令行脚本
- `install.sh`：安装脚本，会把命令安装到 `~/.local/bin/cc-switch`
- `completions/_cc-switch`：`zsh` 补全脚本
- `profiles/`：示例 profile 配置文件

## 安装

在仓库根目录执行：

```bash
chmod +x cc-switch install.sh
./install.sh
```

卸载时执行：

```bash
./install.sh uninstall
```

如果 `~/.local/bin` 还不在 `PATH` 中，把下面这行加入你的 shell 配置：

```bash
export PATH="$HOME/.local/bin:$PATH"
```

如果你使用 `zsh`，安装脚本还会把补全文件安装到 `~/.local/share/zsh/site-functions/_cc-switch`。
若补全还没有生效，把下面这行加入 `~/.zshrc`，并确保它出现在 `compinit` 之前：

```bash
fpath=("$HOME/.local/share/zsh/site-functions" $fpath)
```

`./install.sh uninstall` 会删除已安装的命令和 `zsh` 补全，并删除仓库自带且未被修改过的示例 profile。
为避免误删用户数据，它不会删除 `~/.claude/settings.json`、备份文件，也不会删除你改过的 profile。

## 命令

```bash
cc-switch list
```

列出 `~/.claude/profiles/` 下可用的所有 profile。

```bash
cc-switch use deepseek
```

校验 `deepseek.json` 的 JSON 格式，备份当前 `settings.json`，然后切换到该 profile。
在 `zsh` 下输入 `cc-switch use deep` 后按 `Tab`，会从 `~/.claude/profiles/*.json` 中补全出 `deepseek`。

```bash
cc-switch current
```

打印当前 `settings.json` 内容，并尽量识别它对应哪个已保存的 profile。如果当前还没有 `settings.json`，会输出提示并返回成功状态。

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
在 `zsh` 下输入 `cc-switch restore set` 后按 `Tab`，会补全 `~/.claude/backups/` 里的备份文件名。

```bash
cc-switch help
```

显示命令帮助，`-h` 和 `--help` 也可用。

## 校验与安全

- 如果目标 profile 不是合法 JSON，将拒绝切换。
- 优先使用 `jq` 校验；如果系统没有 `jq`，则回退到 `python3 -m json.tool`。
- 不会删除已有 profile。
- 执行 `use` 时会先自动备份当前全局配置。
- 执行 `restore` 时也会先备份当前全局配置。
- 备份默认只保留最近 `50` 份；可通过环境变量 `BACKUP_KEEP_COUNT` 调整。
- profile 名只允许字母、数字、`.`、`_`、`-`，避免写出目录穿越路径。
- 所有文件路径都做了引用处理，能够正确处理包含空格的路径。
- `deepseek.json` 中的 token 仍是占位值，切换前会给出警告。

## 示例 Profiles

安装脚本会附带这些示例文件：

- `official.json`
- `deepseek.json`
- `local-test.json`

## 一键安装

把这些文件写入当前目录后，执行：

```bash
chmod +x install.sh && ./install.sh
```
