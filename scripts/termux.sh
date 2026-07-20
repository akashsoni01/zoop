#!/usr/bin/env bash
# Termux (Android) — host-side Zoop development (QR decode / OLED UI tests).
# Full esp-idf / espflash on Termux is fragile; use a laptop for board flash.
# Kit reference: ESP32-S3 Camera + OLED + INMP441 + MAX98357A (develop elsewhere).
#
# Usage in Termux:
#   pkg install git curl
#   git clone <your-zoop-url> && cd zoop
#   bash scripts/termux.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=common.sh
source "${SCRIPT_DIR}/common.sh"

for arg in "$@"; do
  case "${arg}" in
    -h|--help)
      cat <<'EOF'
termux.sh — Zoop host tools inside Termux (Android)

Installs Rust + runs cargo tests for zoop-core / firmware-qr.
Does NOT install espup / flash toolchain (use macOS/Linux/Windows scripts).

Prereqs (Termux):
  pkg update && pkg install git curl build-essential openssl
EOF
      exit 0
      ;;
  esac
done

zoop_print_kit
zoop_log "Zoop root: ${ZOOP_ROOT}"

if [[ -z "${PREFIX:-}" ]] || [[ ! -d "/data/data/com.termux" && ! -d "${PREFIX}" ]]; then
  zoop_warn "Does not look like Termux — continuing anyway"
fi

zoop_log "Installing Termux packages"
pkg update -y
pkg install -y \
  git curl wget \
  clang make cmake ninja \
  rust \
  openssl \
  which \
  2>/dev/null || {
    zoop_warn "pkg install failed partly — try: pkg install rust clang make git curl"
  }

# Prefer Termux rust if present; else rustup
if command -v rustc >/dev/null 2>&1 && command -v cargo >/dev/null 2>&1; then
  zoop_ok "Rust available ($(rustc --version))"
else
  zoop_install_rust
fi

zoop_ensure_cargo_bin_path

# Host-only verify — skip esp toolchain
zoop_log "Termux mode: host-only (no espup)"
zoop_verify_host

cat <<EOF

──────────────────────────────────────────────────────────────
Termux limits
──────────────────────────────────────────────────────────────
• Host tests + zoop-firmware-qr demo work here.
• ESP32 flash / esp-idf builds: use a laptop
    macOS:  bash scripts/macos-setup.sh
    Linux:  bash scripts/linux-setup.sh
    Win:    bash scripts/windows-setup.sh   (or WSL2)

• Wire the DIY AI Voice+Vision kit on a desk, flash from PC,
  then use Termux only for editing / reviewing host logic if needed.

EOF

zoop_log "Termux host setup finished."
