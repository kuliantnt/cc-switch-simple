# cc-switch

`cc-switch` 是一个在 WSL/Linux 下切换 Claude Code 全局配置的 Bash 小工具。

它只做一件事：在本机安全地切换 `~/.claude/settings.json`，并管理常用的 profile 和备份。

管理路径：

- 当前配置：`~/.claude/settings.json`
- profile：`~/.claude/profiles/*.json`
- 备份：`~/.claude/backups/settings-YYYYmmdd-HHMMSS.json`

## 快速安装

```bash
chmod +x cc-switch install.sh
./install.sh
```

卸载：

```bash
./install.sh uninstall
```

如果 `~/.local/bin` 还不在 `PATH` 中：

```bash
export PATH="$HOME/.local/bin:$PATH"
```

## 快速使用

```bash
cc-switch
```

无参数时默认执行 `next`，会按文件名排序切换到下一个 profile。

```bash
cc-switch list
cc-switch use deepseek
cc-switch current
cc-switch backup
cc-switch restore settings-20260518-142604.json
```

可用命令：

- `cc-switch` / `cc-switch next`：切到下一个 profile
- `cc-switch list`：列出 profile
- `cc-switch use <profile>`：切到指定 profile
- `cc-switch current [--show-token]`：查看当前配置
- `cc-switch backup`：手动创建备份
- `cc-switch edit <profile>`：编辑 profile
- `cc-switch new <profile>`：把当前配置保存成新 profile
- `cc-switch restore <backup-file>`：恢复备份
- `cc-switch help`：显示帮助

## 文档

- [安装与卸载](docs/install.md)
- [命令说明与手动测试](docs/usage.md)
- [设计说明、安全规则与排错](docs/notes.md)
