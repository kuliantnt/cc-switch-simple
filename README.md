简体中文 | [English](./README.en.md)

---

# cc-switch

`cc-switch` 现在是一个 Rust 编写的跨平台 CLI，用来切换 Claude Code 的配置 profile。

这个版本故意保持很小：

- 管理 JSON profile
- 把选中的 profile 写入 Claude Code 目标配置文件
- 每次切换前自动备份当前配置
- 在 Windows / macOS / Linux 上以单文件可执行程序运行

## 命令

```text
cc-switch list
cc-switch current
cc-switch use <name>
cc-switch next
cc-switch doctor
```

行为规则：

- profile 按文件名排序
- `list` 会用 `*` 标记当前匹配到的 profile
- `current` 会按规范化后的 JSON 内容匹配当前配置
- `use` 和 `next` 都会先备份，再写入
- `next` 在当前配置无法识别时，会回退到第一个 profile

## 运行时目录

运行时配置目录：

- Linux/macOS: `~/.cc-switch-simple/`
- Windows: 用户配置目录下的 `cc-switch-simple/`

目录内容：

- `profiles/`：profile JSON
- `backups/`：自动备份
- `config.toml`：可选配置

Claude Code 默认目标配置路径：

- `~/.claude/settings.json`

可以在 `config.toml` 中覆盖：

```toml
[claude]
settings_path = "~/.claude/settings.json"
```

如果 `settings_path` 是相对路径，会相对 `config.toml` 所在目录解析。

## 构建与安装

在仓库根目录执行：

```bash
cargo build --release
```

生成的单文件可执行程序位于：

- Linux/macOS: `target/release/cc-switch`
- Windows: `target\\release\\cc-switch.exe`

如果要全局使用，把该文件复制到你的 `PATH` 即可。

在 macOS/Linux 上，也可以用符号链接放到常见的用户 `bin` 目录，例如：

```bash
mkdir -p ~/.local/bin
ln -sf "$(pwd)/target/release/cc-switch" ~/.local/bin/cc-switch
```

确认 `~/.local/bin` 已在 `PATH` 中即可。Windows 请继续使用复制 `cc-switch.exe` 到 `PATH` 目录的方式。

## 初始化 profile

仓库仍保留示例模板：

- `profiles/official.template.json`
- `profiles/deepseek.template.json`
- `profiles/local-test.template.json`

可以手动复制到运行时目录的 `profiles/`，并去掉 `.template`：

- `official.template.json` -> `official.json`
- `deepseek.template.json` -> `deepseek.json`

## 使用

列出 profile：

```bash
cc-switch list
```

查看当前配置匹配到的 profile：

```bash
cc-switch current
```

切换到指定 profile：

```bash
cc-switch use deepseek
```

切换到下一个 profile：

```bash
cc-switch next
```

检查目录、配置路径和 JSON 有效性：

```bash
cc-switch doctor
```

## 测试

```bash
cargo test
```

当前基础测试覆盖：

- profile 列表排序
- `next` 轮换逻辑
- 备份文件名生成
- 路径解析与 `config.toml` 读取

## 约束

- 不依赖 Python、Node、Bash、Zsh
- 单一二进制分发
- 使用 `clap`、`serde`、`toml`、`directories`、`anyhow`

## 备注

这个版本不再保留原来的 Bash 安装脚本和 shell 补全安装逻辑。先把核心 CLI 稳定下来，后续再决定是否补安装器、打包或更复杂交互。
