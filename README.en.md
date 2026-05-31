[简体中文](./README.md) | English

---

# cc-switch

`cc-switch` is a Rust-based cross-platform CLI with two switching modes:

- Claude Code JSON profile switching
- Codex `config.toml` preset switching

The tool stays intentionally small:

- overwrite the target config from local presets
- create a backup before every overwrite
- avoid printing sensitive values
- ship as a single executable on Windows, macOS, and Linux

## Commands

Claude Code:

```text
cc-switch list
cc-switch current
cc-switch use <name>
cc-switch next
cc-switch doctor
```

Codex:

```text
cc-switch cx list
cc-switch cx current
cc-switch cx use <name>
cc-switch cx next
```

Behavior:

- Claude profiles are sorted by filename and matched by canonical JSON content
- Codex profiles are sorted by directory name and tracked via `~/.cc-switch-simple/codex/current`
- `use` and `next` always back up the current target file before overwrite
- `next` falls back to the first profile when the current selection is missing or unknown

## Runtime Layout

Claude Code runtime directory:

- Linux/macOS: `~/.cc-switch-simple/`
- Windows: `cc-switch-simple/` under the user's config directory

Inside it:

- `profiles/` stores Claude JSON profiles
- `backups/` stores Claude backups
- `config.toml` is optional

Default Claude Code target path:

- `~/.claude/settings.json`

You can override it in `config.toml`:

```toml
[claude]
settings_path = "~/.claude/settings.json"

[backups]
max_files = 5
```

Notes:

- `[backups].max_files` defaults to `5`
- `max_files` must be greater than `0`
- it applies to both Claude and Codex backup retention; for Codex, `config.toml` and `auth.json` each keep up to `max_files` backups
- relative `settings_path` values are resolved from the runtime config directory

Codex runtime paths are fixed:

- preset config: `~/.cc-switch-simple/codex/<name>/config.toml`
- preset auth: `~/.cc-switch-simple/codex/<name>/auth.json`
- current selection record: `~/.cc-switch-simple/codex/current`
- backup directory: `~/.cc-switch-simple/backups/codex/`
- active config: `${CODEX_HOME:-$HOME/.codex}/config.toml`
- active auth: `${CODEX_HOME:-$HOME/.codex}/auth.json`

Codex mode switches both files together:

- the selected preset must contain both `config.toml` and `auth.json`
- existing target files are backed up before overwrite
- `cc-switch` does not print API keys or token values

## Claude Profile Setup

The repo still ships example templates in `profiles/`:

- `profiles/official.template.json`
- `profiles/deepseek.template.json`
- `profiles/local-test.template.json`

Copy them into the runtime directory and drop the `.template` suffix:

```bash
mkdir -p ~/.cc-switch-simple/profiles
cp profiles/official.template.json ~/.cc-switch-simple/profiles/official.json
cp profiles/deepseek.template.json ~/.cc-switch-simple/profiles/deepseek.json
cp profiles/local-test.template.json ~/.cc-switch-simple/profiles/local-test.json
```

## Codex Preset Setup

Create two example presets:

```bash
mkdir -p ~/.cc-switch-simple/codex/openai
mkdir -p ~/.cc-switch-simple/codex/xxxcom
```

`~/.cc-switch-simple/codex/openai/config.toml`:

```toml
model = "gpt-5"
model_provider = "openai"
approval_policy = "on-request"
sandbox_mode = "workspace-write"
```

`~/.cc-switch-simple/codex/openai/auth.json`:

```json
{
  "OPENAI_API_KEY": "<redacted>"
}
```

`~/.cc-switch-simple/codex/xxxcom/config.toml`:

```toml
model = "gpt-5"
model_provider = "xxxcom"
approval_policy = "on-request"
sandbox_mode = "workspace-write"
```

`~/.cc-switch-simple/codex/xxxcom/auth.json`:

```json
{
  "XXXCOM_API_KEY": "<redacted>"
}
```

When switching, `cc-switch` backs up and overwrites both `${CODEX_HOME:-$HOME/.codex}/config.toml` and `${CODEX_HOME:-$HOME/.codex}/auth.json`.

## Usage

Claude Code:

```bash
cc-switch list
cc-switch current
cc-switch use deepseek
cc-switch next
cc-switch doctor
```

Codex:

```bash
cc-switch cx list
cc-switch cx current
cc-switch cx use openai
cc-switch cx next
```

## Build And Verify

Run from the repository root:

```bash
cargo build --release
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

The single executable will be generated at:

- Linux/macOS: `target/release/cc-switch`
- Windows: `target\\release\\cc-switch.exe`

## Constraints

- no Python, Node, Bash, or Zsh dependency
- single-binary distribution
- uses `clap`, `serde`, `toml`, `directories`, and `anyhow`

## Community

Questions, suggestions, or want to help out? Join the conversation at **[linux.do](https://linux.do/t/topic/2279788)**.
