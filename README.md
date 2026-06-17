简体中文 | [English](./README.en.md)

---

# cc-switch

[toc]

`cc-switch` 是一个 Rust 编写的跨平台 CLI，现在同时支持两种切换模式：

- Claude Code JSON profile 切换
- Codex `config.toml` 预设切换

工具保持小而直接：

- 使用本地预设覆盖目标配置
- 每次覆盖前自动备份
- 不输出敏感值
- 在 Windows / macOS / Linux 上以单文件可执行程序运行

## 命令

Claude Code：

```text
cc-switch list
cc-switch current
cc-switch use <name>
cc-switch next
cc-switch before
cc-switch doctor
```

Codex：

```text
cc-switch cx list
cc-switch cx current
cc-switch cx use <name>
cc-switch cx next
cc-switch cx before
```

行为规则：

- Claude profile 按文件名排序，通过规范化 JSON 内容识别当前项
- Codex profile 按目录名排序，通过 `~/.cc-switch-simple/codex/current` 记录当前项
- `use`、`next` 和 `before` 真正切换前都会先备份现有目标文件
- `next` 在当前项缺失或无法识别时，会回退到第一个 profile
- `before` 使用最近一次成功切换前的 profile，不按名称排序
- `before` 在没有历史记录、历史记录非法或 profile 已删除时会提示并跳过

## 运行时目录

默认运行时根目录：

- Linux/macOS: `~/.cc-switch-simple/`
- Windows: 用户配置目录下的 `cc-switch-simple/`

Claude Code 相关文件：

- `profiles/`：Claude JSON profile
- `current`：当前选择记录
- `before`：最近一次成功切换前的 profile 记录
- `backups/`：Claude 自动备份
- `config.toml`：可选配置

Claude Code 默认目标配置路径：

- `~/.claude/settings.json`

可通过 `config.toml` 覆盖：

```toml
[claude]
settings_path = "~/.claude/settings.json"

[backups]
max_files = 5
```

说明：

- `[backups].max_files` 默认是 `5`
- `max_files` 必须大于 `0`
- 同时作用于 Claude 和 Codex 的自动备份保留数量；对于 Codex，会对 `config.toml` 和 `auth.json` 分别保留 `max_files` 个备份
- 如果 `settings_path` 是相对路径，会相对 `config.toml` 所在目录解析

Codex 相关文件：

- 预设根目录：`~/.cc-switch-simple/codex/`
- 预设配置：`~/.cc-switch-simple/codex/<name>/config.toml`
- 预设认证：`~/.cc-switch-simple/codex/<name>/auth.json`
- 当前选择记录：`~/.cc-switch-simple/codex/current`
- 上一次切换记录：`~/.cc-switch-simple/codex/before`
- 备份目录：`~/.cc-switch-simple/backups/codex/`
- 当前生效配置：`${CODEX_HOME:-$HOME/.codex}/config.toml`
- 当前生效认证：`${CODEX_HOME:-$HOME/.codex}/auth.json`

Codex 模式会一起切换这两个文件：

- 选中的预设目录必须同时包含 `config.toml` 和 `auth.json`
- 覆盖前会分别备份当前目标文件
- 切换离开当前 Codex 预设前，如果 `${CODEX_HOME:-$HOME/.codex}/auth.json` 有变化，会自动保存回当前预设；ChatGPT Plus 登录状态会随 profile 自动更新
- `cc-switch` 不会输出 API Key 或 token 内容

自动创建规则：

- Claude 相关命令会自动创建 `~/.cc-switch-simple/`、`profiles/`、`backups/`
- `cc-switch cx use <name>` 会自动创建 `~/.cc-switch-simple/codex/`、`~/.cc-switch-simple/backups/codex/`，以及 `${CODEX_HOME:-$HOME/.codex}/`
- `~/.cc-switch-simple/codex/<name>/` 和其中的 `config.toml`、`auth.json` 不会自动生成，仍需手动准备
- `cc-switch cx list` 和 `cc-switch cx current` 只读取现有文件，不会初始化预设目录

## Claude Profile 初始化

仓库 `profiles/` 目录仍提供示例模板：

- `profiles/official.template.json`
- `profiles/deepseek.template.json`
- `profiles/local-test.template.json`

复制到运行时目录并去掉 `.template` 后缀即可：

```bash
mkdir -p ~/.cc-switch-simple/profiles
cp profiles/official.template.json ~/.cc-switch-simple/profiles/official.json
cp profiles/deepseek.template.json ~/.cc-switch-simple/profiles/deepseek.json
cp profiles/local-test.template.json ~/.cc-switch-simple/profiles/local-test.json
```

## Codex 预设初始化

仓库 `codex/` 目录也提供可直接复制的 Codex 预设示例：

- `codex/openai/config.toml`
- `codex/openai/auth.json`
- `codex/xxxcom/config.toml`
- `codex/xxxcom/auth.json`

先创建预设目录，再把示例复制过去：

```bash
mkdir -p ~/.cc-switch-simple/codex/openai
mkdir -p ~/.cc-switch-simple/codex/xxxcom
cp codex/openai/config.toml ~/.cc-switch-simple/codex/openai/config.toml
cp codex/openai/auth.json ~/.cc-switch-simple/codex/openai/auth.json
cp codex/xxxcom/config.toml ~/.cc-switch-simple/codex/xxxcom/config.toml
cp codex/xxxcom/auth.json ~/.cc-switch-simple/codex/xxxcom/auth.json
```

复制后可按需编辑。例如 `~/.cc-switch-simple/codex/openai/config.toml`：

```toml
model = "gpt-5"
model_provider = "openai"
approval_policy = "on-request"
sandbox_mode = "workspace-write"
```

`~/.cc-switch-simple/codex/openai/auth.json`：

```json
{
  "OPENAI_API_KEY": "<redacted>"
}
```

`~/.cc-switch-simple/codex/xxxcom/config.toml`：

```toml
disable_response_storage = true
model = "gpt-5.5"
model_reasoning_effort = "high"
model_provider = "xxxcom"
model_context_window = 1000000
model_auto_compact_token_limit = 900000

[model_providers.xxxcom]
name = "xxxcom"
base_url = "https://xxxcom.net/v1"
requires_openai_auth = true
wire_api = "responses"
```

`~/.cc-switch-simple/codex/xxxcom/auth.json`：

```json
{
  "XXXCOM_API_KEY": "<redacted>"
}
```

切换时，`cc-switch` 会先备份，再覆盖 `${CODEX_HOME:-$HOME/.codex}/config.toml` 和 `${CODEX_HOME:-$HOME/.codex}/auth.json`。如果 Codex 或 ChatGPT Plus 登录流程更新了当前 `auth.json`，下次切换离开该预设时会自动写回 `~/.cc-switch-simple/codex/<name>/auth.json`，无需手动同步。

## 使用

Claude Code：

```bash
cc-switch list
cc-switch current
cc-switch use deepseek
cc-switch next
cc-switch before
cc-switch doctor
```

Codex：

```bash
cc-switch cx list
cc-switch cx current
cc-switch cx use openai
cc-switch cx next
cc-switch cx before
```

## 构建与验证

在仓库根目录执行：

```bash
cargo build --release
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

生成的单文件可执行程序位于：

- Linux/macOS: `target/release/cc-switch`
- Windows: `target\\release\\cc-switch.exe`

## 约束

- 不依赖 Python、Node、Bash、Zsh
- 单一二进制分发
- 使用 `clap`、`serde`、`toml`、`directories`、`anyhow`

## 社区

有问题、建议，或想一起折腾？欢迎来 **[linux.do](https://linux.do/t/topic/2279788)** 社区交流反馈。
