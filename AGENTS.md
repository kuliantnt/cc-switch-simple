# Repository Guidelines

## Project Structure & Module Organization
This repository is a small Rust CLI project. Keep the binary entrypoint in `src/main.rs`, reusable behavior in `src/lib.rs`, command parsing in `src/cli.rs`, path resolution in `src/paths.rs`, and Codex-specific switching logic in `src/codex.rs`. User-facing documentation lives in `README.md`, `README.zh-CN.md`, and `README.en.md`. Product-shipped example Claude profiles stay in `profiles/*.template.json`; Codex examples stay under `codex/<name>/config.toml` and `codex/<name>/auth.json`. Integration tests belong in `tests/`.

## Build, Test, and Development Commands
Run commands from the repository root:

- `cargo fmt --check` verifies formatting.
- `cargo fmt` formats Rust code.
- `cargo clippy --all-targets --all-features -- -D warnings` catches warnings and lints.
- `cargo test` runs the automated test suite.
- `cargo build --release` builds the distributable `cc-switch` binary.

When manually testing profile switching, use throwaway directories or temporary `HOME` / `CODEX_HOME` values rather than production `~/.claude/` or `~/.codex/` files.

## Coding Style & Naming Conventions
Write idiomatic Rust 2024. Keep functions small, return `anyhow::Result` for fallible command paths, and include context on filesystem errors. Use `snake_case` for functions and variables, `PascalCase` for types and enum variants, and concise clap help text for user-facing commands. Keep terminal output short and avoid printing secret values.

## Testing Guidelines
Every behavior change should include focused integration tests under `tests/`. Prefer `tempfile::TempDir` sandboxes and manually constructed `ResolvedPaths` so tests never touch real user configuration. Cover both successful switching and no-op or skip cases, especially around missing profile files, invalid records, backup creation, and sync-back behavior.

## Commit & Pull Request Guidelines
Use short, imperative commit subjects such as `Add before profile switch`. Keep each commit scoped to one behavior change. Pull requests should explain the user-visible effect, list validation commands, and include terminal output snippets when behavior or error messages change.

## Security & Configuration Tips
Never commit real API tokens, populated `settings.json`, or live Codex auth files. Keep sample profiles redacted, and treat anything under `~/.claude/`, `~/.codex/`, or `~/.cc-switch-simple/` as user data that must be backed up rather than overwritten blindly.
