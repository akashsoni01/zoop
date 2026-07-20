#!/usr/bin/env bash
# Fresh macOS laptop → Zoop host + ESP32-S3 toolchain.
# Kit: ESP32-S3 Camera Board + OLED + INMP441 + MAX98357A + speaker
#
# Usage (from anywhere):
#   bash scripts/macos-setup.sh
#   bash scripts/macos-setup.sh --host-only    # skip espup (faster)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=common.sh
source "${SCRIPT_DIR}/common.sh"

HOST_ONLY=0
for arg in "$@"; do
  case "${arg}" in
    --host-only) HOST_ONLY=1 ;;
    -h|--help)
      cat <<'EOF'
macos-setup.sh — install Zoop dev tools on a fresh Mac

  --host-only   Rust + cargo test / firmware-qr only (no espup)
  -h, --help    Show this help
EOF
      exit 0
      ;;
  esac
done

zoop_print_kit
zoop_log "Zoop root: ${ZOOP_ROOT}"

if ! command -v brew >/dev/null 2>&1; then
  zoop_warn "Homebrew not found — installing (optional but recommended)"
  /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)" || true
fi

if command -v brew >/dev/null 2>&1; then
  zoop_log "Installing macOS deps (cmake, ninja, dfu-util, python3, git)"
  brew list cmake >/dev/null 2>&1 || brew install cmake
  brew list ninja >/dev/null 2>&1 || brew install ninja
  brew list dfu-util >/dev/null 2>&1 || brew install dfu-util
  brew list python3 >/dev/null 2>&1 || brew install python3
  brew list git >/dev/null 2>&1 || brew install git
  # libclang for bindgen / esp-idf-sys
  brew list llvm >/dev/null 2>&1 || brew install llvm
  zoop_ok "brew packages ready"
else
  zoop_warn "Continue without Homebrew — ensure cmake/ninja/python3/git are on PATH"
fi

zoop_install_rust
zoop_ensure_cargo_bin_path

if [[ "${HOST_ONLY}" -eq 0 ]]; then
  zoop_install_esp_tools
  zoop_install_esp_toolchain
  zoop_source_esp_env
else
  zoop_log "Skipping espup (--host-only)"
fi

zoop_verify_host
zoop_print_next_steps

zoop_log "macOS setup finished."
zoop_ok "Add to ~/.zshrc:  . ${ZOOP_EXPORT_ESP:-~/export-esp.sh}"
