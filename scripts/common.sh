#!/usr/bin/env bash
# Shared helpers for Zoop fresh-laptop setup scripts.
# Target kit: ESP32-S3 Camera Board + OLED + INMP441 + MAX98357A + speaker
# (DIY AI Voice / Vision kit — Keyestudio KS5028-class wiring as draft).

set -euo pipefail

ZOOP_SCRIPTS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ZOOP_ROOT="$(cd "${ZOOP_SCRIPTS_DIR}/.." && pwd)"

# Prefer espup's export file; fall back to common locations.
ZOOP_EXPORT_ESP="${ZOOP_EXPORT_ESP:-}"
if [[ -z "${ZOOP_EXPORT_ESP}" ]]; then
  for candidate in \
    "${HOME}/export-esp.sh" \
    "${HOME}/.espressif/export-esp.sh" \
    "${HOME}/.cargo/export-esp.sh"; do
    if [[ -f "${candidate}" ]]; then
      ZOOP_EXPORT_ESP="${candidate}"
      break
    fi
  done
fi

zoop_log()  { printf '==> %s\n' "$*"; }
zoop_ok()   { printf '    ✓ %s\n' "$*"; }
zoop_warn() { printf '    ! %s\n' "$*" >&2; }
zoop_die()  { printf 'ERROR: %s\n' "$*" >&2; exit 1; }

zoop_need_cmd() {
  command -v "$1" >/dev/null 2>&1 || zoop_die "missing command: $1 — install it and re-run"
}

zoop_ensure_cargo_bin_path() {
  local cargo_bin="${CARGO_HOME:-${HOME}/.cargo}/bin"
  case ":${PATH}:" in
    *":${cargo_bin}:"*) ;;
    *) export PATH="${cargo_bin}:${PATH}" ;;
  esac
}

zoop_print_kit() {
  cat <<'EOF'

┌─────────────────────────────────────────────────────────────┐
│  Zoop target hardware (QR / voice+vision track)             │
│  ESP32-S3 Camera Board · OLED · INMP441 · MAX98357A · Spk │
└─────────────────────────────────────────────────────────────┘

Draft wiring (KS5028-class ESP32-S3-CAM — verify silkscreen on your board):

  INMP441 mic (I²S RX)
    WS  → GPIO 1
    SCK → GPIO 2
    SD  → GPIO 42
    VDD → 3V3    GND → GND (+ L/R shorted to GND for left)

  MAX98357A amp (I²S TX) → speaker
    DIN  → GPIO 39
    BCLK → GPIO 40
    LRC  → GPIO 41
    Vin  → 3V3 (SD often tied to Vin)    GND → GND

  OLED SSD1306 128×64 (I²C) — draft in docs/qr/hardware.md
    SDA → GPIO 8    SCL → GPIO 9   (freeze after HIL)

  Camera: onboard 24-pin FPC (OV series) — no Dupont needed
  USB: Type-C data cable (not charge-only) · UART 115200

Docs: docs/qr/hardware.md · TODO_qr.md · TODO_camera.md

EOF
}

zoop_install_rust() {
  if command -v rustc >/dev/null 2>&1 && command -v cargo >/dev/null 2>&1; then
    zoop_ok "Rust already installed ($(rustc --version))"
    return
  fi
  zoop_log "Installing Rust via rustup (default toolchain)"
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  # shellcheck disable=SC1091
  source "${HOME}/.cargo/env"
  zoop_ok "Rust installed ($(rustc --version))"
}

zoop_install_esp_tools() {
  zoop_ensure_cargo_bin_path
  zoop_log "Installing esp-rs host tools (espup, espflash, cargo-espflash, ldproxy)"
  cargo install espup espflash cargo-espflash ldproxy --locked 2>/dev/null \
    || cargo install espup espflash cargo-espflash ldproxy
  zoop_ok "esp-rs Cargo tools installed"
}

zoop_install_esp_toolchain() {
  zoop_ensure_cargo_bin_path
  zoop_need_cmd espup
  zoop_log "Installing ESP Xtensa / RISC-V toolchain via espup (may take several minutes)"
  espup install
  if [[ -z "${ZOOP_EXPORT_ESP}" || ! -f "${ZOOP_EXPORT_ESP}" ]]; then
    if [[ -f "${HOME}/export-esp.sh" ]]; then
      ZOOP_EXPORT_ESP="${HOME}/export-esp.sh"
    fi
  fi
  if [[ -n "${ZOOP_EXPORT_ESP}" && -f "${ZOOP_EXPORT_ESP}" ]]; then
    zoop_ok "espup export file: ${ZOOP_EXPORT_ESP}"
  else
    zoop_warn "Could not find export-esp.sh — check espup install output for the path"
  fi
}

zoop_source_esp_env() {
  zoop_ensure_cargo_bin_path
  if [[ -n "${ZOOP_EXPORT_ESP}" && -f "${ZOOP_EXPORT_ESP}" ]]; then
    # shellcheck disable=SC1090
    source "${ZOOP_EXPORT_ESP}"
    zoop_ok "Sourced ${ZOOP_EXPORT_ESP}"
  else
    zoop_warn "Skip sourcing esp env (export-esp.sh not found yet)"
  fi
}

zoop_verify_host() {
  zoop_ensure_cargo_bin_path
  cd "${ZOOP_ROOT}"
  zoop_log "Host verify: zoop-core QR + OLED tests"
  cargo test -p zoop-core qr:: --quiet
  cargo test -p zoop-core state_qr --quiet
  cargo test -p zoop-core oled --quiet
  zoop_log "Host demo: zoop-firmware-qr"
  cargo run -q -p zoop-firmware-qr
  zoop_ok "Host track OK"
}

zoop_print_next_steps() {
  local export_hint="${ZOOP_EXPORT_ESP:-~/export-esp.sh}"
  cat <<EOF

──────────────────────────────────────────────────────────────
Next steps
──────────────────────────────────────────────────────────────
1. Every new shell (firmware builds):
     . ${export_hint}

2. Host-only (no board):
     cd ${ZOOP_ROOT}
     cargo test -p zoop-core
     cargo run -p zoop-firmware-qr

3. E-Paper collect firmware (Waveshare board):
     cd ${ZOOP_ROOT}/firmware
     cp -n secrets.example.toml secrets.toml   # edit Wi-Fi / keys
     cargo build
     cargo espflash flash --monitor

4. Camera / QR kit (this DIY AI Voice+Vision board):
     See docs/qr/hardware.md and TODO_qr.md Phase 3 (esp_camera HIL).
     Use a Type-C data cable; hold BOOT + tap RST if the port won't open.

5. Serial permissions (Linux): add yourself to dialout, then re-login:
     sudo usermod -aG dialout "\$USER"

EOF
}
