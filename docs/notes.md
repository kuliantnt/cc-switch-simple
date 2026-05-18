# 设计说明、安全规则与排错

## 设计取向

`cc-switch` 的范围刻意很小：

- 只处理本机的 `settings.json` 切换
- 用纯 Bash 实现，便于审计和改动
- 不引入后台服务、面板或额外构建步骤

如果你已经有几份 `settings.json`，只想快速、安全地来回切换，这个项目更合适；如果你需要更重的多 provider 管理能力，应该用范围更大的工具。

## 安全与校验

- 如果目标 profile 不是合法 JSON，会拒绝切换
- 优先使用 `jq` 校验；若系统没有 `jq`，回退到 `python3 -m json.tool`
- `use` 和 `restore` 都会先备份当前配置
- 默认只保留最近 `10` 份备份，可通过 `BACKUP_KEEP_COUNT` 调整
- `current` 默认会 mask `ANTHROPIC_AUTH_TOKEN`
- profile 名只允许字母、数字、`.`、`_`、`-`，且不能以 `.` 开头
- 写入 `settings.json` 时使用同目录临时文件再 `mv`，并将权限设为 `600`
- 模板 profile 里的 token 仍可能是占位值，切换前会给出警告

## 本地 profile 管理

仓库里的 `profiles/.gitignore` 会忽略本地 `profiles/*.json`，所以你可以把个人 profile 放在仓库里的 `profiles/` 下使用，而不会误提交。

## Zsh 补全排错

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
