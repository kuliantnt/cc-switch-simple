# Repository Guidelines

## Project Structure & Module Organization
This repository is currently flat and centered on a small Bash CLI. The root currently contains [`agents.md`](/home/lianlian/project/cc-switch-simple/agents.md), which records the original product requirements. Keep executable entrypoints at the top level, for example `cc-switch` and `install.sh`, and keep user-facing documentation in `README.md`. If sample JSON profiles or fixtures are added, place them in a dedicated directory such as `examples/` or `testdata/` instead of mixing them with scripts.

## Build, Test, and Development Commands
No build step is required. Use shell-native checks during development:

- `bash -n cc-switch install.sh` validates shell syntax.
- `shellcheck cc-switch install.sh` catches quoting, path, and portability issues.
- `shfmt -w cc-switch install.sh` normalizes formatting if `shfmt` is installed.
- `./install.sh` installs the tool into `~/.local/bin` for manual verification.

Run commands from the repository root. When testing profile switching, use throwaway files under `~/.claude/profiles/` rather than editing production settings first.

## Coding Style & Naming Conventions
Write Bash for WSL/Linux with predictable, defensive defaults. Use `#!/usr/bin/env bash`, quote every path, and prefer small functions over long command blocks. Indent with 2 spaces. Use `snake_case` for function names, uppercase for exported environment variables, and lowercase kebab-case for executable filenames such as `cc-switch`. Keep terminal output concise and actionable.

## Testing Guidelines
This project does not yet include an automated test suite, so every change should include syntax validation plus a manual smoke test. Test each command path at least once: `list`, `use`, `current`, `backup`, `edit`, and `new`. If you add automated tests later, place them in `tests/` and name them after the command being verified, for example `tests/use.bats`.

## Commit & Pull Request Guidelines
Git history is not available in this workspace snapshot, so use short, imperative commit subjects such as `Add backup before profile switch`. Keep each commit scoped to one behavior change. Pull requests should explain the user-visible effect, list manual test commands, and include terminal output snippets when behavior changes or error messages are updated.

## Security & Configuration Tips
Never commit real API tokens or populated `settings.json` files. Keep sample profiles redacted, and treat anything under `~/.claude/` as user data that must be backed up, not overwritten blindly.
