[简体中文](./README.md) | English

---

# cc-switch

`cc-switch` is now a Rust-based cross-platform CLI for switching Claude Code configuration profiles.

The scope is intentionally small:

- manage JSON profiles
- write a selected profile into Claude Code's target settings file
- back up the current settings before every switch
- ship as a single executable on Windows, macOS, and Linux

## Commands

```text
cc-switch list
cc-switch current
cc-switch use <name>
cc-switch next
cc-switch doctor
```

Behavior:

- profiles are sorted by filename
- `list` marks the matched active profile with `*`
- `current` matches the target settings file against saved profiles by canonical JSON content
- `use` and `next` always back up the current target settings before writing
- `next` falls back to the first profile when the current settings cannot be matched

## Runtime Layout

Runtime config directory:

- Linux/macOS: `~/.cc-switch-simple/`
- Windows: `cc-switch-simple/` under the user's config directory

Inside that directory:

- `profiles/` stores profile JSON files
- `backups/` stores automatic backups
- `config.toml` is optional

Default Claude Code target path:

- `~/.claude/settings.json`

You can override it in `config.toml`:

```toml
[claude]
settings_path = "~/.claude/settings.json"
```

Relative `settings_path` values are resolved from the runtime config directory.

## Build

Run from the repository root:

```bash
cargo build --release
```

The single executable will be generated at:

- Linux/macOS: `target/release/cc-switch`
- Windows: `target\\release\\cc-switch.exe`

Copy that file into your `PATH` if you want a global install.

On macOS/Linux, you can also symlink it into a common user `bin` directory:

```bash
mkdir -p ~/.local/bin
ln -sf "$(pwd)/target/release/cc-switch" ~/.local/bin/cc-switch
```

Make sure `~/.local/bin` is already in your `PATH`. On Windows, keep using the `cc-switch.exe` copy-to-`PATH` approach.

## Bootstrapping Profiles

The repo ships three example templates under `profiles/`:

| Template | Purpose |
|---|---|
| `profiles/official.template.json` | Official Anthropic API (empty profile, no env override) |
| `profiles/deepseek.template.json` | DeepSeek proxy — overrides model and base URL |
| `profiles/local-test.template.json` | Local/offline testing — disables non-essential network requests |

### Setup

Copy the templates you need into the runtime `profiles/` directory, **removing the `.template` suffix** from each filename. They will be recognized by `cc-switch list` immediately.

**Linux / macOS:**

```bash
# ensure the directory exists (first-time setup)
mkdir -p ~/.cc-switch-simple/profiles

# copy templates (drop .template)
cp profiles/official.template.json ~/.cc-switch-simple/profiles/official.json
cp profiles/deepseek.template.json ~/.cc-switch-simple/profiles/deepseek.json
cp profiles/local-test.template.json ~/.cc-switch-simple/profiles/local-test.json
```

**Windows (PowerShell):**

```powershell
# runtime path: %APPDATA%\cc-switch-simple\profiles
# resolves to C:\Users\<user>\AppData\Roaming\cc-switch-simple\profiles

mkdir -Force "$env:APPDATA\cc-switch-simple\profiles"

Copy-Item profiles\official.template.json "$env:APPDATA\cc-switch-simple\profiles\official.json"
Copy-Item profiles\deepseek.template.json "$env:APPDATA\cc-switch-simple\profiles\deepseek.json"
```

> Note: `deepseek.template.json` contains a placeholder `ANTHROPIC_AUTH_TOKEN` value (`sk-填这里`). Replace it with your actual API key before use.

## Usage

List available profiles:

```bash
cc-switch list
```

Show the currently matched profile:

```bash
cc-switch current
```

Switch to a named profile:

```bash
cc-switch use deepseek
```

Rotate to the next profile:

```bash
cc-switch next
```

Check directories, config paths, and JSON validity:

```bash
cc-switch doctor
```

## Tests

```bash
cargo test
```

Current baseline coverage includes:

- profile listing order
- `next` rotation behavior
- backup filename generation
- path resolution and `config.toml` parsing

## Constraints

- no Python, Node, Bash, or Zsh dependency
- single-binary distribution
- uses `clap`, `serde`, `toml`, `directories`, and `anyhow`

## Community

Questions, suggestions, or want to help out? Join the conversation at **[linux.do](https://linux.do/t/topic/2279788)**.

## Notes

This version intentionally drops the old Bash installer and shell-completion installation flow. The priority is a stable core CLI first; packaging and richer UX can be added later if needed.
