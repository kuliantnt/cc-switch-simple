[简体中文](./README.md) | English

---

# cc-switch

[toc]

`cc-switch` is a Rust-based cross-platform CLI with two switching modes:

- Claude Code JSON profile switching
- Codex `config.toml` / `auth.json` preset switching with optional `models_catalog.json`

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
cc-switch before
cc-switch doctor
```

Codex:

```text
cc-switch cx list
cc-switch cx current
cc-switch cx use <name>
cc-switch cx next
cc-switch cx before
```

You can also use the standalone Codex command:

```text
cx-switch list
cx-switch current
cx-switch use <name>
cx-switch next
cx-switch before
```

Behavior:

- Claude profiles are sorted by filename and matched by canonical JSON content
- Codex profiles are sorted by directory name and tracked via `~/.cc-switch-simple/codex/current`
- `use`, `next`, and `before` back up existing target files before real switches
- `next` falls back to the first profile when the current selection is missing or unknown
- `before` uses the profile from before the most recent successful switch, not name order
- `before` prints a skip message and exits successfully when history is missing, invalid, or deleted

## Runtime Layout

Default runtime root:

- Linux/macOS: `~/.cc-switch-simple/`
- Windows: `cc-switch-simple/` under the user's config directory

Claude Code files:

- `profiles/` stores Claude JSON profiles
- `current` stores the current selection record
- `before` stores the profile from before the most recent successful switch
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
- it applies to both Claude and Codex backup retention; for Codex, `config.toml`, `auth.json`, and any present `models_catalog.json` each keep up to `max_files` backups
- relative `settings_path` values are resolved from the runtime config directory

Codex files:

- preset root: `~/.cc-switch-simple/codex/`
- preset config: `~/.cc-switch-simple/codex/<name>/config.toml`
- preset auth: `~/.cc-switch-simple/codex/<name>/auth.json`
- current selection record: `~/.cc-switch-simple/codex/current`
- previous switch record: `~/.cc-switch-simple/codex/before`
- backup directory: `~/.cc-switch-simple/backups/codex/`
- active config: `${CODEX_HOME:-$HOME/.codex}/config.toml`
- active auth: `${CODEX_HOME:-$HOME/.codex}/auth.json`
- active model catalog (optional): `${CODEX_HOME:-$HOME/.codex}/models_catalog.json`

Codex mode switches the config and auth files together and handles the optional model catalog:

- the selected preset must contain both `config.toml` and `auth.json`
- if a preset contains `models_catalog.json`, it is written during the switch; if it does not, an existing active catalog is backed up and removed
- existing target files are backed up before overwrite or removal
- before switching away from the current Codex preset, changed `${CODEX_HOME:-$HOME/.codex}/auth.json` is saved back to that preset automatically; ChatGPT Plus login state is updated with the profile
- `cc-switch` and `cx-switch` do not print API keys or token values

Auto-creation rules:

- Claude-related commands create `~/.cc-switch-simple/`, `profiles/`, and `backups/`
- `cc-switch cx use <name>` and `cx-switch use <name>` create `~/.cc-switch-simple/codex/`, `~/.cc-switch-simple/backups/codex/`, and `${CODEX_HOME:-$HOME/.codex}/`
- `~/.cc-switch-simple/codex/<name>/` and its `config.toml` / `auth.json` / optional `models_catalog.json` are not generated automatically and must still be prepared manually
- `cc-switch cx list` and `cc-switch cx current` only read existing files and do not initialize presets

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

The repo also ships copy-ready Codex preset examples in `codex/`:

- `codex/openai/config.toml`
- `codex/openai/auth.json`
- `codex/deepseek/config.toml`
- `codex/deepseek/auth.json`
- `codex/deepseek/models_catalog.json`
- `codex/xxxcom/config.toml`
- `codex/xxxcom/auth.json`

When using DeepSeek's Moon Bridge setup, place its generated `models_catalog.json` in the corresponding preset directory. It contains Codex model capability metadata, not an API key.

The bundled `codex/deepseek/` preset provides a Moon Bridge configuration without a key. Start Moon Bridge and configure its DeepSeek API key as described in the DeepSeek guide, copy this preset, then run `cx-switch use deepseek` or `cc-switch cx use deepseek`.

Create the preset directories, then copy the examples over:

```bash
mkdir -p ~/.cc-switch-simple/codex/openai
mkdir -p ~/.cc-switch-simple/codex/deepseek
mkdir -p ~/.cc-switch-simple/codex/xxxcom
cp codex/openai/config.toml ~/.cc-switch-simple/codex/openai/config.toml
cp codex/openai/auth.json ~/.cc-switch-simple/codex/openai/auth.json
cp codex/deepseek/config.toml ~/.cc-switch-simple/codex/deepseek/config.toml
cp codex/deepseek/auth.json ~/.cc-switch-simple/codex/deepseek/auth.json
cp codex/deepseek/models_catalog.json ~/.cc-switch-simple/codex/deepseek/models_catalog.json
cp codex/xxxcom/config.toml ~/.cc-switch-simple/codex/xxxcom/config.toml
cp codex/xxxcom/auth.json ~/.cc-switch-simple/codex/xxxcom/auth.json
```

After copying, edit the files as needed. For example, `~/.cc-switch-simple/codex/openai/config.toml`:

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

`~/.cc-switch-simple/codex/xxxcom/auth.json`:

```json
{
  "XXXCOM_API_KEY": "<redacted>"
}
```

When switching, `cc-switch` or `cx-switch` backs up and overwrites `${CODEX_HOME:-$HOME/.codex}/config.toml` and `${CODEX_HOME:-$HOME/.codex}/auth.json`, and writes or removes `models_catalog.json` according to the selected preset. If Codex or ChatGPT Plus login refreshes the active `auth.json`, the next switch away from that preset automatically writes it back to `~/.cc-switch-simple/codex/<name>/auth.json`; no manual sync is required.

## Usage

Claude Code:

```bash
cc-switch list
cc-switch current
cc-switch use deepseek
cc-switch next
cc-switch before
cc-switch doctor
```

Codex:

```bash
cc-switch cx list
cc-switch cx current
cc-switch cx use openai
cc-switch cx next
cc-switch cx before
```

Or:

```bash
cx-switch list
cx-switch current
cx-switch use openai
cx-switch next
cx-switch before
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
- standalone Codex entrypoint: `target/release/cx-switch` (`cx-switch.exe` on Windows)

## Constraints

- no Python, Node, Bash, or Zsh dependency
- single-file binary distribution
- uses `clap`, `serde`, `toml`, `directories`, and `anyhow`

## Community

Questions, suggestions, or want to help out? Join the conversation at **[linux.do](https://linux.do/t/topic/2279788)**.
