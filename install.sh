#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CLAUDE_DIR="${HOME}/.claude"
PROFILES_DIR="${CLAUDE_DIR}/profiles"
BACKUPS_DIR="${CLAUDE_DIR}/backups"
BIN_DIR="${HOME}/.local/bin"
TARGET_BIN="${BIN_DIR}/cc-switch"

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

main() {
  local -a profile_files=()

  mkdir -p "$PROFILES_DIR" "$BACKUPS_DIR" "$BIN_DIR"

  cp "${SCRIPT_DIR}/cc-switch" "$TARGET_BIN"
  chmod +x "$TARGET_BIN"
  printf 'Installed command: %s\n' "$TARGET_BIN"

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

  printf '\nTry these commands:\n'
  printf '  cc-switch list\n'
  printf '  cc-switch use official\n'
  printf '  cc-switch current\n'
  printf '  cc-switch help\n'
}

main "$@"
