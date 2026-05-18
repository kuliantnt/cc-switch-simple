# cc-switch

`cc-switch` 是一个用于在 WSL/Linux 下切换 Claude Code 全局配置的 Bash 小工具。

它管理的路径包括：

- 当前配置：`~/.claude/settings.json`
- 可复用 profiles：`~/.claude/profiles/*.json`
- 备份目录：`~/.claude/backups/settings-YYYYmmdd-HHMMSS.json`

## 文件说明

- `cc-switch`：主命令行脚本
- `install.sh`：安装脚本，会把命令安装到 `~/.local/bin/cc-switch`
- `profiles/`：示例 profile 配置文件

## 安装

在仓库根目录执行：

```bash
chmod +x cc-switch install.sh
./install.sh
```

如果 `~/.local/bin` 还不在 `PATH` 中，把下面这行加入你的 shell 配置：

```bash
export PATH="$HOME/.local/bin:$PATH"
```

## 命令

```bash
cc-switch list
```

列出 `~/.claude/profiles/` 下可用的所有 profile。

```bash
cc-switch use deepseek
```

校验 `deepseek.json` 的 JSON 格式，备份当前 `settings.json`，然后切换到该 profile。

```bash
cc-switch current
```

打印当前 `settings.json` 内容，并尽量识别它对应哪个已保存的 profile。

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

## 校验与安全

- 如果目标 profile 不是合法 JSON，将拒绝切换。
- 优先使用 `jq` 校验；如果系统没有 `jq`，则回退到 `python3 -m json.tool`。
- 不会删除已有 profile。
- 执行 `use` 时会先自动备份当前全局配置。
- 所有文件路径都做了引用处理，能够正确处理包含空格的路径。

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
