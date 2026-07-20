#!/usr/bin/env bash
# Fresh Windows laptop → Zoop host + ESP32-S3 toolchain.
# Run inside Git Bash or WSL2 (recommended: WSL2 Ubuntu).
# Kit: ESP32-S3 Camera Board + OLED + INMP441 + MAX98357A + speaker
#
# Usage (Git Bash / WSL):
#   bash scripts/windows-setup.sh
#   bash scripts/windows-setup.sh --host-only
#
# Native PowerShell alternative (run in an elevated PowerShell):
#   winget install Rustlang.Rustup Git.Git
#   then open a new shell and re-run this script from the repo.

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
windows-setup.sh — Zoop setup for Windows (Git Bash or WSL2)

  --host-only   Rust + cargo test / firmware-qr only (no espup)
  -h, --help    Show this help

Recommended path: WSL2 Ubuntu, then:
  sudo apt update && cd /mnt/c/path/to/zoop
  bash scripts/linux-setup.sh

USB serial on native Windows often needs a CH340 / CP210x driver from the
board vendor. In WSL2, USB passthrough requires usbipd-win.
EOF
      exit 0
      ;;
  esac
done

zoop_print_kit
zoop_log "Zoop root: ${ZOOP_ROOT}"

# Detect environment
UNAME="$(uname -s 2>/dev/null || echo unknown)"
IS_WSL=0
if grep -qi microsoft /proc/version 2>/dev/null; then
  IS_WSL=1
fi

if [[ "${IS_WSL}" -eq 1 ]]; then
  zoop_ok "Running under WSL — delegating to linux-setup.sh"
  exec bash "${SCRIPT_DIR}/linux-setup.sh" "$@"
fi

if [[ "${UNAME}" != MINGW* && "${UNAME}" != MSYS* && "${UNAME}" != CYGWIN* ]]; then
  zoop_warn "Not Git Bash/MSYS and not WSL — continuing anyway"
fi

zoop_log "Windows / Git Bash path"
zoop_warn "For firmware flash, WSL2 + usbipd or native Windows espflash is more reliable"

# Ensure curl exists (Git for Windows usually provides it)
if ! command -v curl >/dev/null 2>&1; then
  zoop_die "curl not found — install Git for Windows or use WSL2"
fi

if ! command -v git >/dev/null 2>&1; then
  zoop_warn "git not on PATH — install from https://git-scm.com/download/win"
fi

zoop_install_rust
zoop_ensure_cargo_bin_path

# shellcheck disable=SC1091
if [[ -f "${HOME}/.cargo/env" ]]; then
  source "${HOME}/.cargo/env"
fi

if [[ "${HOST_ONLY}" -eq 0 ]]; then
  zoop_install_esp_tools
  zoop_install_esp_toolchain
  zoop_source_esp_env
else
  zoop_log "Skipping espup (--host-only)"
fi

zoop_verify_host

cat <<EOF

──────────────────────────────────────────────────────────────
Windows notes (DIY AI Voice + Vision kit)
──────────────────────────────────────────────────────────────
• Use a Type-C *data* cable. Device Manager should show a COM port.
• CH340 / CP210x driver may be required for the ESP32-S3-CAM USB bridge.
• Flash from Git Bash after sourcing export-esp.sh:
    . ~/export-esp.sh
    cd firmware && cargo espflash flash --monitor
• Prefer WSL2 for a Linux-like IDF experience:
    wsl --install
    # then: bash scripts/linux-setup.sh
• USB into WSL2: install usbipd-win, then
    usbipd list
    usbipd bind --busid <BUSID>
    usbipd attach --wsl --busid <BUSID>

Kit wiring: see scripts/common.sh kit banner or docs/qr/hardware.md
EOF

zoop_print_next_steps
zoop_log "Windows setup finished."
