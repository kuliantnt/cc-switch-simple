# 安装与卸载

## 安装

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

## Zsh 补全

如果你使用 `zsh`，确认 `~/.zshrc` 在 `compinit` 之前包含：

```bash
fpath=("$HOME/.zsh/completions" $fpath)
autoload -Uz compinit
compinit
```

如果已经有 `autoload -Uz compinit` 和 `compinit`，不要重复添加，只要确保 `fpath=...` 在前面。

## 卸载

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

## 模板文件

仓库跟踪的是模板文件：

- `profiles/official.template.json`
- `profiles/deepseek.template.json`
- `profiles/local-test.template.json`

安装时会自动去掉 `.template` 后缀，例如 `deepseek.template.json` 会安装成 `deepseek.json`。
