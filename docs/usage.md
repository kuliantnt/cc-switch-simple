# 命令说明

## 默认行为

```bash
cc-switch
```

无参数时等价于：

```bash
cc-switch next
```

它会按文件名排序读取 `~/.claude/profiles/*.json`，判断当前 `~/.claude/settings.json` 是否和某个 profile 完全一致，然后切到下一个；如果已经是最后一个则循环回第一个；如果当前配置无法识别或 `settings.json` 还不存在，则直接切到第一个 profile。

## 命令

```bash
cc-switch list
```

列出 `~/.claude/profiles/` 下可用的 profile。

```bash
cc-switch next
```

显式切换到下一个 profile。若只有一个 profile，仍会切换到该唯一 profile，并给出提示。

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

显示帮助。由于 `cc-switch` 默认执行 `next`，如果只是想看帮助，请显式使用上面的形式。

## 手动测试

建议至少准备两个内容不同的 profile，然后执行：

```bash
cc-switch list
cc-switch current
cc-switch
cc-switch
cc-switch next
cc-switch use deepseek
cc-switch
```

重点确认：

- `list` 的顺序是否按文件名排序
- `cc-switch` 和 `next` 是否按顺序循环切换
- `current` 识别出的 profile 是否与刚切换到的目标一致
- 当前配置不匹配任何 profile 时，是否显示 `Current: unknown` 并切到第一个 profile
