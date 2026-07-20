#!/usr/bin/env bash
# Fresh Linux laptop/desktop → Zoop host + ESP32-S3 toolchain.
# Kit: ESP32-S3 Camera Board + OLED + INMP441 + MAX98357A + speaker
#
# Usage:
#   bash scripts/linux-setup.sh
#   bash scripts/linux-setup.sh --host-only

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
linux-setup.sh — install Zoop dev tools on a fresh Linux machine

  --host-only   Rust + cargo test / firmware-qr only (no espup)
  -h, --help    Show this help

Requires sudo for apt/dnf/pacman packages and dialout group.
EOF
      exit 0
      ;;
  esac
done

zoop_print_kit
zoop_log "Zoop root: ${ZOOP_ROOT}"

install_pkgs_debian() {
  zoop_log "Installing apt packages"
  sudo apt-get update
  sudo DEBIAN_FRONTEND=noninteractive apt-get install -y \
    build-essential cmake ninja-build pkg-config git curl wget \
    python3 python3-pip python3-venv \
    libudev-dev libssl-dev \
    clang libclang-dev \
    dfu-util
}

install_pkgs_fedora() {
  zoop_log "Installing dnf packages"
  sudo dnf install -y \
    gcc gcc-c++ cmake ninja-build pkgconf-pkg-config git curl wget \
    python3 python3-pip \
    openssl-devel systemd-devel \
    clang clang-devel \
    dfu-util
}

install_pkgs_arch() {
  zoop_log "Installing pacman packages"
  sudo pacman -Syu --needed --noconfirm \
    base-devel cmake ninja pkgconf git curl wget \
    python python-pip \
    openssl \
    clang \
    dfu-util
}

if command -v apt-get >/dev/null 2>&1; then
  install_pkgs_debian
elif command -v dnf >/dev/null 2>&1; then
  install_pkgs_fedora
elif command -v pacman >/dev/null 2>&1; then
  install_pkgs_arch
else
  zoop_warn "Unknown package manager — install cmake, ninja, clang, python3, git manually"
fi

if getent group dialout >/dev/null 2>&1; then
  if id -nG "${USER}" | grep -qw dialout; then
    zoop_ok "User already in dialout (serial flash OK)"
  else
    zoop_log "Adding ${USER} to dialout for USB serial"
    sudo usermod -aG dialout "${USER}"
    zoop_warn "Log out and back in (or reboot) before cargo espflash"
  fi
fi

# udev rule hint for Espressif USB (optional)
UDEV_HINT="/etc/udev/rules.d/99-esp-usb.rules"
if [[ ! -f "${UDEV_HINT}" ]] && command -v sudo >/dev/null 2>&1; then
  zoop_log "Optional: install Espressif USB udev rules? (skipped by default)"
  zoop_warn "If flash fails with permission denied, see https://docs.espressif.com/projects/esp-idf/en/latest/esp32/get-started/linux-setup.html"
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

zoop_log "Linux setup finished."
zoop_ok "Add to ~/.bashrc:  . ${ZOOP_EXPORT_ESP:-~/export-esp.sh}"
