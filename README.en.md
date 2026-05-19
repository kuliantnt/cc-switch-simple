[简体中文](./README.md) | English

---

# cc-switch

> This project focuses on Claude Code configuration switching. Codex already supports `config.toml` profiles — consider using `codex --profile <name>` first.

`cc-switch` is a lightweight Bash tool for WSL/Linux that safely switches Claude Code's global configuration file `~/.claude/settings.json`.

It does one thing:

- Switch the current config between multiple profiles
- Auto-backup before each switch
- Help you manage your local profiles

If you just want to quickly flip between a few `settings.json` files, this tool is a good fit.

## Managed Files

- Active config: `~/.claude/settings.json`
- Profiles: `~/.claude/profiles/*.json`
- Backups: `~/.claude/backups/settings-YYYYmmdd-HHMMSS.json`

## Quick Start

Clone the repo and enter the directory:

```bash
git clone https://github.com/kuliantnt/cc-switch-simple.git
cd cc-switch-simple
```

Then run the install:

```bash
chmod +x cc-switch install.sh
./install.sh
```

Then try it out:

```bash
cc-switch list
cc-switch current
cc-switch next
```

If `cc-switch` is not found, add this line to your shell config:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

## Installation

All commands below assume you are in the repo root.

The install script handles these automatically:

- Install the command to `~/.local/bin/cc-switch`
- Install `zsh` completions to `~/.zsh/completions/_cc-switch`
- Copy `profiles/*.template.json` from the repo to `~/.claude/profiles/`

### Template Files

The repo tracks template files:

- `profiles/official.template.json`
- `profiles/deepseek.template.json`
- `profiles/local-test.template.json`

During install, the `.template` suffix is stripped. For example:

- `deepseek.template.json` -> `deepseek.json`

### Zsh Completions

If you use `zsh`, make sure `~/.zshrc` includes the following before `compinit`:

```bash
fpath=("$HOME/.zsh/completions" $fpath)
autoload -Uz compinit
compinit
```

If you already have `autoload -Uz compinit` and `compinit`, do not duplicate them — just make sure `fpath=...` comes first.

## Uninstall

```bash
./install.sh uninstall
```

Uninstall removes:

- The installed command
- The installed `zsh` completions
- Profiles installed from templates that have not been modified

Uninstall does NOT remove:

- `~/.claude/settings.json`
- Backup files
- Profiles you have manually modified

## Commands

### List Available Profiles

```bash
cc-switch list
```

Lists profiles under `~/.claude/profiles/`, with `*` marking the currently active one.

### Show Current Config

```bash
cc-switch current
cc-switch current --show-token
```

Prints the current `settings.json` and attempts to identify which saved profile it matches.

- `ANTHROPIC_AUTH_TOKEN` is masked by default
- Use `--show-token` to reveal the full value

### Switch to Next Profile

```bash
cc-switch next
```

`next` reads `~/.claude/profiles/*.json` sorted by filename, then:

- If the current config matches a profile, switches to the next one
- If already on the last profile, wraps around to the first
- If the current config cannot be matched, or `settings.json` does not exist, switches directly to the first

### Switch to a Specific Profile

```bash
cc-switch use deepseek
```

On execution:

- Validates that `deepseek.json` is valid JSON
- Backs up the current `settings.json`
- Switches to the target profile

### Create a New Profile

```bash
cc-switch new my-profile
```

Saves the current `settings.json` as a new profile, e.g. `my-profile.json`.

### Edit a Profile

```bash
cc-switch edit deepseek
```

Opens the specified profile with `$EDITOR`. Falls back to `nano` if `$EDITOR` is not set.

### Backup Current Config

```bash
cc-switch backup
```

Creates a timestamped backup of the current config.

### Restore a Backup

```bash
cc-switch restore settings-20260518-142604.json
```

Restores the specified backup as the current `settings.json`. If a current config already exists, it is automatically backed up first.

### Show Help

```bash
cc-switch
cc-switch help
cc-switch -h
cc-switch --help
```

Running `cc-switch` with no arguments shows help and does not modify the current config.

## Suggested Workflow

A smooth workflow typically looks like:

1. Prepare two or more profiles
2. Verify they are all present with `cc-switch list`
3. Check which one is active with `cc-switch current`
4. Switch with `cc-switch next` or `cc-switch use <name>`
5. If the current config is worth keeping, save it as a profile with `cc-switch new <name>`

## Safety & Validation

`cc-switch` operates conservatively on your config by default:

- Refuses to switch if the target profile is not valid JSON
- Prefers `jq` for JSON validation; falls back to `python3 -m json.tool` if `jq` is not installed
- `use` and `restore` both back up the current config before making changes
- Keeps at most `10` recent backups by default (configurable via `BACKUP_KEEP_COUNT`)
- Writes `settings.json` via a temp file in the same directory, then `mv` into place
- Sets `settings.json` permissions to `600` after writing
- Profile names allow only letters, digits, `.`, `_`, `-`, and must not start with `.`
- Template profiles may contain placeholder tokens — a warning is shown before switching

## Local Profile Management

The repo's `profiles/.gitignore` ignores local `profiles/*.json`, so you can keep personal profiles in the repo's `profiles/` directory without accidentally committing them.

## Manual Testing

Prepare at least two profiles with different content, then run:

```bash
cc-switch list
cc-switch current
cc-switch next
cc-switch next
cc-switch use deepseek
cc-switch
```

Key checks:

- `list` output is sorted by filename
- `cc-switch` (no args) consistently shows help without touching the current config
- `next` cycles through profiles in order
- `current` correctly identifies the active profile
- When the current config matches no profile, it shows `Current: unknown` and switches to the first profile

## Zsh Completion Troubleshooting

If `Tab` only completes regular filenames, it is usually a `fpath` ordering issue or stale `zsh` cache. Check in order:

```bash
which cc-switch
cc-switch list
ls ~/.zsh/completions/_cc-switch
head -5 ~/.zsh/completions/_cc-switch
rm -f ~/.zcompdump*
exec zsh
```

Then retest:

```bash
cc-switch <Tab>
cc-switch next <Tab>
cc-switch use <Tab>
cc-switch edit <Tab>
cc-switch restore <Tab>
```

## Design Philosophy

`cc-switch` is intentionally small in scope:

- Handles only local `settings.json` switching
- Pure Bash — easy to audit and modify
- No daemons, dashboards, or extra build steps

If you want "simple, direct, controllable" config switching, this tool fits well. If you need more complete multi-provider management, a broader-scope tool is a better choice.
