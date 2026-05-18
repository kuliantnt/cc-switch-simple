#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CLAUDE_DIR="${HOME}/.claude"
PROFILES_DIR="${CLAUDE_DIR}/profiles"
BACKUPS_DIR="${CLAUDE_DIR}/backups"
BIN_DIR="${HOME}/.local/bin"
TARGET_BIN="${BIN_DIR}/cc-switch"
ZSH_COMPLETIONS_DIR="${HOME}/.local/share/zsh/site-functions"
TARGET_ZSH_COMPLETION="${ZSH_COMPLETIONS_DIR}/_cc-switch"

usage() {
  cat <<'EOF'
Usage:
  ./install.sh
  ./install.sh install
  ./install.sh uninstall

Notes:
  - install: install command, zsh completion, and bundled sample profiles
  - uninstall: remove installed command/completion and remove only unmodified bundled sample profiles
EOF
}

install_profile_if_missing() {
  local source_file="$1"
  local target_file="${PROFILES_DIR}/$(basename "$source_file")"

  if [[ -e "$target_file" ]]; then
    printf 'Skip existing profile: %s\n' "$target_file"
    return
  fi

  cp "$source_file" "$target_file"
  printf 'Installed profile: %s\n' "$target_file"
}

remove_file_if_exists() {
  local target_file="$1"
  local label="$2"

  if [[ -e "$target_file" || -L "$target_file" ]]; then
    rm -f -- "$target_file"
    printf 'Removed %s: %s\n' "$label" "$target_file"
    return
  fi

  printf 'Skip missing %s: %s\n' "$label" "$target_file"
}

remove_sample_profile_if_unmodified() {
  local source_file="$1"
  local target_file="${PROFILES_DIR}/$(basename "$source_file")"

  if [[ ! -e "$target_file" ]]; then
    printf 'Skip missing sample profile: %s\n' "$target_file"
    return
  fi

  if cmp -s "$source_file" "$target_file"; then
    rm -f -- "$target_file"
    printf 'Removed sample profile: %s\n' "$target_file"
    return
  fi

  printf 'Keep modified profile: %s\n' "$target_file"
}

remove_dir_if_empty() {
  local dir="$1"

  if [[ -d "$dir" ]] && rmdir "$dir" 2>/dev/null; then
    printf 'Removed empty directory: %s\n' "$dir"
  fi
}

install_main() {
  local -a profile_files=()
  local profile_file

  mkdir -p "$PROFILES_DIR" "$BACKUPS_DIR" "$BIN_DIR" "$ZSH_COMPLETIONS_DIR"

  cp "${SCRIPT_DIR}/cc-switch" "$TARGET_BIN"
  chmod +x "$TARGET_BIN"
  printf 'Installed command: %s\n' "$TARGET_BIN"

  cp "${SCRIPT_DIR}/completions/_cc-switch" "$TARGET_ZSH_COMPLETION"
  printf 'Installed zsh completion: %s\n' "$TARGET_ZSH_COMPLETION"

  shopt -s nullglob
  profile_files=("${SCRIPT_DIR}/profiles/"*.json)
  shopt -u nullglob

  for profile_file in "${profile_files[@]}"; do
    install_profile_if_missing "$profile_file"
  done

  case ":${PATH}:" in
    *":${BIN_DIR}:"*)
      printf 'PATH already contains %s\n' "$BIN_DIR"
      ;;
    *)
      printf '\nAdd this to your shell config if needed:\n'
      printf 'export PATH="$HOME/.local/bin:$PATH"\n'
      ;;
  esac

  printf '\nIf zsh completion is not active yet, add this to ~/.zshrc before compinit:\n'
  printf 'fpath=("$HOME/.local/share/zsh/site-functions" $fpath)\n'

  printf '\nTry these commands:\n'
  printf '  cc-switch list\n'
  printf '  cc-switch use official\n'
  printf '  cc-switch current\n'
  printf '  cc-switch help\n'
}

uninstall_main() {
  local -a profile_files=()
  local profile_file

  remove_file_if_exists "$TARGET_BIN" "command"
  remove_file_if_exists "$TARGET_ZSH_COMPLETION" "zsh completion"

  shopt -s nullglob
  profile_files=("${SCRIPT_DIR}/profiles/"*.json)
  shopt -u nullglob

  for profile_file in "${profile_files[@]}"; do
    remove_sample_profile_if_unmodified "$profile_file"
  done

  remove_dir_if_empty "$PROFILES_DIR"
  remove_dir_if_empty "$BACKUPS_DIR"
  remove_dir_if_empty "$CLAUDE_DIR"
  remove_dir_if_empty "$BIN_DIR"
  remove_dir_if_empty "$ZSH_COMPLETIONS_DIR"
  remove_dir_if_empty "${HOME}/.local/share/zsh"
  remove_dir_if_empty "${HOME}/.local/share"
  remove_dir_if_empty "${HOME}/.local"

  printf '\nUninstall finished.\n'
  printf 'Kept user data such as settings.json, backups, and modified profiles.\n'
}

main() {
  local action="${1:-install}"

  case "$action" in
    install)
      install_main
      ;;
    uninstall)
      uninstall_main
      ;;
    help|-h|--help)
      usage
      ;;
    *)
      usage
      printf '\nError: Unknown action: %s\n' "$action" >&2
      exit 1
      ;;
  esac
}

main "$@"
