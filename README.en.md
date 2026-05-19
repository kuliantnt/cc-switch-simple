[简体中文](./README.zh-CN.md) | English

---

# cc-switch

> This project focuses on Claude Code configuration switching. Codex already supports config.toml profiles — consider using `codex --profile <name>` directly.

`cc-switch` is a lightweight Bash tool for switching Claude Code global configurations on WSL/Linux.

It does one thing: safely switch `~/.claude/settings.json` while managing profiles and backups.

Managed paths:

- Active config: `~/.claude/settings.json`
- Profiles: `~/.claude/profiles/*.json`
- Backups: `~/.claude/backups/settings-YYYYmmdd-HHMMSS.json`

## Quick Install

```bash
chmod +x cc-switch install.sh
./install.sh
```

Uninstall:

```bash
./install.sh uninstall
```

If `~/.local/bin` is not on your `PATH`:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

## Quick Start

```bash
cc-switch help
```

Running without arguments shows help. To switch to the next profile, use `cc-switch next`.

```bash
cc-switch list
cc-switch use deepseek
cc-switch current
cc-switch backup
cc-switch restore settings-20260518-142604.json
```

Available commands:

- `cc-switch` — show help
- `cc-switch next` — switch to the next profile
- `cc-switch list` — list profiles with `*` marking the active one
- `cc-switch use <profile>` — switch to a specific profile
- `cc-switch current [--show-token]` — show current config
- `cc-switch backup` — create a manual backup
- `cc-switch edit <profile>` — edit a profile
- `cc-switch new <profile>` — save current config as a new profile
- `cc-switch restore <backup-file>` — restore from a backup
- `cc-switch help` — show help

## Docs

- [Install & Uninstall](docs/install.md)
- [Commands & Manual Testing](docs/usage.md)
- [Design Notes, Safety Rules & Troubleshooting](docs/notes.md)
