# Zoop setup scripts (fresh laptop)

Install everything needed to develop Zoop on a new machine — **host Rust tests** first, then the **ESP32-S3 firmware toolchain** for flashing the **OceanLabz DIY AI Voice Kit** (ESP32-S3 Camera + OLED + INMP441 + MAX98357 + speaker).

| Kit part | Role in Zoop |
| --- | --- |
| ESP32-S3 Camera Board | MCU + OV camera (QR scan / vision) |
| OLED **1.54″** (SSD1309 / 128×64) | Aiming / decoded string UI |
| INMP441 | I²S mic (cues / future voice) |
| MAX98357 + speaker | I²S amp / “DAC” beeps |

**Hardware source of truth:** [`physical-components/hardware_spec.md`](../physical-components/hardware_spec.md) (OceanLabz DIY AI Voice Kit)

Tracks: [`TODO_qr.md`](../TODO_qr.md) · [`docs/qr/hardware.md`](../docs/qr/hardware.md) · e-Paper collect: [`TODO.md`](../TODO.md)

**Hardware assembly (beginners):** [`physical-components/README.md`](../physical-components/README.md)

---

## Quick start (recommended)

Clone the repo, then run the script for your OS. Scripts are **idempotent** (safe to re-run).

| OS | Command | What it does |
| --- | --- | --- |
| macOS | `bash scripts/macos-setup.sh` | Homebrew deps → Rust → esp-rs → host verify |
| Linux | `bash scripts/linux-setup.sh` | apt/dnf/pacman deps → Rust → esp-rs → host verify |
| Windows | Prefer **WSL2 Ubuntu**, then `bash scripts/linux-setup.sh` | Same as Linux inside WSL |
| Windows (Git Bash) | `bash scripts/windows-setup.sh` | Rust + esp-rs; flash is fussier than WSL |
| Termux | `bash scripts/termux.sh` | Host tests only (no board flash) |

**Host-only** (skip ESP toolchain — faster if you are not flashing yet):

```bash
bash scripts/macos-setup.sh --host-only
bash scripts/linux-setup.sh --host-only
```

Shared helpers + kit wiring banner: [`common.sh`](./common.sh).

---

## What you need, how to install it, and why

Two layers. **Layer A** is enough for Mac/PC unit tests and the QR demo. **Layer B** is required to build/flash firmware onto the ESP32-S3 board.

### Layer A — host development (always)

| Install | How | Why |
| --- | --- | --- |
| **Git** | macOS: `brew install git` · Linux: `apt install git` · Win: [git-scm.com](https://git-scm.com) | Clone the repo; version control |
| **curl** | Usually preinstalled; Linux: `apt install curl` | Download rustup / Homebrew installers |
| **Rust + Cargo** via **rustup** | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` then restart shell / `source ~/.cargo/env` | Zoop is Rust. `cargo test` / `cargo run` build `zoop-core` and `zoop-firmware-qr` on your laptop **without** a board |
| **C compiler toolchain** | macOS: Xcode CLT (`xcode-select --install`) · Linux: `build-essential` / `base-devel` | Native deps and crates that compile C bindings |
| **Python 3** | macOS: `brew install python3` · Linux: `python3` | ESP-IDF / espup tooling expects Python; some build scripts use it |

After Layer A you can:

```bash
cargo test -p zoop-core
cargo test -p zoop-core qr::
cargo test -p zoop-core oled
cargo run -p zoop-firmware-qr
```

### Layer B — ESP32-S3 firmware (flash the kit)

| Install | How | Why |
| --- | --- | --- |
| **cmake** | `brew install cmake` · `apt install cmake` | ESP-IDF / `esp-idf-sys` build system |
| **ninja** | `brew install ninja` · `apt install ninja-build` | Fast parallel builds used by IDF |
| **pkg-config** | Linux package managers | Find OpenSSL / system libs at link time |
| **clang / LLVM + libclang** | macOS: `brew install llvm` · Linux: `clang libclang-dev` | **bindgen** generates Rust FFI for ESP-IDF C headers (`esp-idf-sys`) |
| **OpenSSL headers** | Linux: `libssl-dev` / `openssl-devel` | HTTPS / crypto crates and some IDF host tools |
| **libudev** (Linux) | `libudev-dev` | USB serial device access for flashing |
| **dfu-util** | `brew` / `apt` / `dnf` | Alternate USB DFU flashing path (handy for some Espressif boards) |
| **espup** | `cargo install espup` | Installs the Espressif **Xtensa** Rust toolchain (`esp` channel) — stock rustup cannot target ESP32-S3 Xtensa alone |
| **espflash** | `cargo install espflash` | Standalone flasher CLI for ESP chips over UART/USB |
| **cargo-espflash** | `cargo install cargo-espflash` | `cargo espflash flash` subcommand (the `espflash` binary alone is **not** enough for `cargo espflash`) |
| **ldproxy** | `cargo install ldproxy` | Linker wrapper required by esp-rs; without it you get `linker ldproxy not found` |
| **esp toolchain** | `espup install` → creates `~/export-esp.sh` | Downloads Xtensa GCC, clang bits, and sets `LIBCLANG_PATH` etc. |

Every new terminal before firmware work:

```bash
. ~/export-esp.sh
```

Add that line to `~/.zshrc` / `~/.bashrc` so you do not forget.

Then:

```bash
cd firmware
cp -n secrets.example.toml secrets.toml   # Wi‑Fi / API keys — never commit
cargo build
cargo espflash flash --monitor
```

QR / camera track firmware will live under `firmware-qr/` (host demo today; ESP-IDF flash in Phase 3 — see `TODO_qr.md`).

### Layer C — OS / USB specifics

| Item | How | Why |
| --- | --- | --- |
| **Homebrew** (macOS) | [brew.sh](https://brew.sh) | Convenient install of cmake, ninja, llvm, python |
| **dialout group** (Linux) | `sudo usermod -aG dialout $USER` then **re-login** | Permission to open `/dev/ttyUSB*` / `/dev/ttyACM*` for `espflash` |
| **Type-C data cable** | Not charge-only | Board enumerates as a serial port for flash + UART logs (115200) |
| **USB serial driver** (Windows native) | CH340 / CP210x from board vendor | Without it, no COM port in Device Manager |
| **usbipd-win** (WSL2) | [dorssel/usbipd-win](https://github.com/dorssel/usbipd-win) | Attach the board’s USB device into WSL so `espflash` can see it |
| **BOOT + RST** dance | Hold BOOT (IO0), tap RST, release BOOT | Puts ESP32-S3-CAM into download mode if auto-reset fails |

---

## What each setup script installs

| Script | System packages | Rust | esp-rs tools | espup toolchain | Host verify |
| --- | --- | --- | --- | --- | --- |
| [`macos-setup.sh`](./macos-setup.sh) | cmake, ninja, dfu-util, python3, git, llvm (via brew) | yes | yes* | yes* | `qr` / `oled` / `firmware-qr` |
| [`linux-setup.sh`](./linux-setup.sh) | build tools, cmake, ninja, clang, openssl, udev, dfu-util + dialout | yes | yes* | yes* | same |
| [`windows-setup.sh`](./windows-setup.sh) | expects Git Bash; **or** hands off to Linux script in WSL | yes | yes* | yes* | same |
| [`termux.sh`](./termux.sh) | Termux `pkg`: clang, cmake, rust, openssl, git | yes (pkg or rustup) | **no** | **no** | host only |

\*Skipped with `--host-only`.

---

## Hardware wiring (why these GPIOs)

**Do not edit pins here.** Canonical table: [`physical-components/hardware_spec.md`](../physical-components/hardware_spec.md) (OceanLabz DIY AI Voice Kit).

| Peripheral | Pins (draft) | Why |
| --- | --- | --- |
| INMP441 | WS=39, SCK=40, SD=41 | I²S mic (OceanLabz camera-board ref) |
| MAX98357 | LRC=21, DIN=47, BCLK=48 | I²S amp → speaker |
| OLED | SDA=8, SCL=9 | I²C UI (draft; avoid mic SD=41) |
| Camera | Onboard FPC | `esp_camera` frames for QR |
| REC / PWR | GPIO 0 / 14 | Confirm / cancel (draft) |

Constants: [`firmware-qr/src/board/pins.rs`](../firmware-qr/src/board/pins.rs).

---

## Verify installation

```bash
# Layer A
rustc --version
cargo --version
cargo test -p zoop-core qr::
cargo run -p zoop-firmware-qr

# Layer B (after . ~/export-esp.sh)
which espflash ldproxy
cargo espflash --version
cd firmware && cargo build
```

| Symptom | Likely fix |
| --- | --- |
| `custom toolchain 'esp' is not installed` | `espup install` and `. ~/export-esp.sh` |
| `no such command espflash` / cargo subcommand missing | `cargo install cargo-espflash` |
| `linker ldproxy not found` | `cargo install ldproxy` |
| Permission denied on `/dev/tty*` (Linux) | dialout group + re-login |
| No COM port (Windows) | data cable + CH340/CP210x driver |
| Flash timeout | BOOT+RST download mode; try lower baud |

---

## Daily workflow after setup

1. `. ~/export-esp.sh` (firmware shells)
2. Change logic in `core/` → `cargo test -p zoop-core`
3. QR demo → `cargo run -p zoop-firmware-qr`
4. Collect firmware → `cd firmware && cargo build && cargo espflash flash --monitor`
5. Camera / QR HIL → follow [`TODO_qr.md`](../TODO_qr.md) Phase 3

---

## Script map

| File | Role |
| --- | --- |
| [`common.sh`](./common.sh) | Shared install helpers, kit banner, verify + next-steps text |
| [`macos-setup.sh`](./macos-setup.sh) | Fresh Mac |
| [`linux-setup.sh`](./linux-setup.sh) | Fresh Linux / WSL |
| [`windows-setup.sh`](./windows-setup.sh) | Git Bash or redirect to WSL |
| [`termux.sh`](./termux.sh) | Android host-only |
