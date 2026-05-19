[简体中文](./README.md) | English

---

# cc-switch

> This project focuses on Claude Code configuration switching. Codex already supports config.toml profiles — consider using `codex --profile <name>` directly.

`cc-switch` is a lightweight Bash tool for switching Claude Code global configurations on WSL/Linux.

It does one thing: safely switch `~/.claude/settings.json` while managing profiles and backups.

Managed paths:

- Active config: `~/.claude/settings.json`
- Profiles: `~/.claude/profiles/*.json`
- Backups: `~/.claude/backups/settings-YYYYmmdd-HHMMSS.json`

## Install & Uninstall

### Install

Run from the repo root:

```bash
chmod +x cc-switch install.sh
./install.sh
```

The install script will:

- Install the command to `~/.local/bin/cc-switch`
- Install `zsh` completions to `~/.zsh/completions/_cc-switch`
- Copy `profiles/*.template.json` from the repo into `~/.claude/profiles/`

If `~/.local/bin` is not on your `PATH`, add this line to your shell config:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

### Zsh Completions

If you use `zsh`, make sure `~/.zshrc` includes the following before `compinit`:

```bash
fpath=("$HOME/.zsh/completions" $fpath)
autoload -Uz compinit
compinit
```

If `autoload -Uz compinit` and `compinit` already exist, do not duplicate them — just ensure `fpath=...` comes first.

### Uninstall

```bash
./install.sh uninstall
```

Uninstall removes:

- The installed command
- The installed `zsh` completions
- Profiles that were installed from templates and have not been modified

Uninstall does NOT remove:

- `~/.claude/settings.json`
- Backup files
- Profiles you have modified

### Template Files

The repo tracks template files:

- `profiles/official.template.json`
- `profiles/deepseek.template.json`
- `profiles/local-test.template.json`

During install the `.template` suffix is stripped — e.g. `deepseek.template.json` becomes `deepseek.json`.

## Commands

### Default Behavior

```bash
cc-switch
```

Running without arguments shows help and does not modify the current config. To switch to the next profile, explicitly run:

```bash
cc-switch next
```

`next` reads `~/.claude/profiles/*.json` sorted by filename, checks whether the current `~/.claude/settings.json` matches a saved profile, then switches to the next one. If already on the last profile it wraps around to the first. If the current config cannot be matched or `settings.json` does not exist, it switches directly to the first profile.

### Command Reference

```bash
cc-switch list
```

List available profiles under `~/.claude/profiles/`, with `*` marking the active one.

```bash
cc-switch next
```

Explicitly switch to the next profile. The command prints both the before and after profile. If there is only one profile, it still switches and prints a note.

```bash
cc-switch use deepseek
```

Validate `deepseek.json` as valid JSON, back up the current `settings.json`, then switch to that profile.

```bash
cc-switch current
cc-switch current --show-token
```

Print the current `settings.json` and attempt to identify which saved profile it matches. The `ANTHROPIC_AUTH_TOKEN` value is masked by default; use `--show-token` to reveal it.

```bash
cc-switch backup
```

Create a timestamped backup of the current config.

```bash
cc-switch edit deepseek
```

Open a profile in `$EDITOR`. Falls back to `nano` if `$EDITOR` is not set.

```bash
cc-switch new my-profile
```

Save the current `settings.json` as a new profile, e.g. `my-profile.json`.

```bash
cc-switch restore settings-20260518-142604.json
```

Restore a backup file as the current `settings.json`. If a current config already exists, it is backed up first automatically.

```bash
cc-switch help
cc-switch -h
cc-switch --help
```

Show help. Running `cc-switch` with no arguments does the same.

### Manual Testing

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
- `current` correctly identifies the profile you just switched to
- When the current config matches no profile, it shows `Current: unknown` and switches to the first profile

## Design, Safety & Troubleshooting

### Design Philosophy

`cc-switch` is intentionally small in scope:

- Handles only local `settings.json` switching
- Pure Bash — easy to audit and modify
- No daemons, dashboards, or extra build steps

If you have a handful of `settings.json` files and want fast, safe switching between them, this tool fits well. If you need heavier multi-provider management, a broader-scope tool is a better choice.

### Safety & Validation

- Refuses to switch if the target profile is not valid JSON
- Prefers `jq` for validation; falls back to `python3 -m json.tool` if `jq` is not installed
- `use` and `restore` both back up the current config before making changes
- Keeps at most `10` recent backups by default (configurable via `BACKUP_KEEP_COUNT`)
- `current` masks `ANTHROPIC_AUTH_TOKEN` by default
- Profile names allow only letters, digits, `.`, `_`, `-`, and must not start with `.`
- Writes `settings.json` via a temp-file-in-same-directory + `mv`, with permissions set to `600`
- Template profiles may contain placeholder tokens — a warning is shown before switching

### Local Profile Management

The repo's `profiles/.gitignore` ignores local `profiles/*.json`, so you can keep personal profiles in the repo's `profiles/` directory without accidentally committing them.

### Zsh Completion Troubleshooting

If `Tab` only completes regular filenames, it's usually a `fpath` ordering issue or stale `zsh` cache. Check in order:

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
