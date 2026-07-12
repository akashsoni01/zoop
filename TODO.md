# Zoop — Zoop Pay (UPI on e-Paper)

Rust firmware for **UPI collect** on the Waveshare ESP32-S3 e-Paper 1.54 board. Evolved from a Palma Notes / [`pala_note`](./pala_note/) voice-notepad port; primary UX is now payment QR + history.

| | |
| --- | --- |
| Product | Zoop Pay — hold REC → UPI QR → waiting → paid |
| Target board | Waveshare **ESP32-S3-ePaper-1.54** |
| Hardware guide | [`README.md`](./README.md) |
| Architecture | [`docs/architecture.md`](./docs/architecture.md) |
| Firmware version | `v1.0` |

### Implementation status

| Symbol | Meaning |
| --- | --- |
| `[x]` | **Done (host-verified)** — logic in `core/` with passing tests, or firmware module compiles with BSP stub wired |
| `[ ] HIL` | **Hardware-in-the-loop** — needs physical Waveshare board (or enclosure fit) to verify end-to-end |

**Host tests:** `cargo test --workspace --exclude zoop-firmware` → **86 passed** (84 unit + 2 integration).  
**Firmware:** `cd firmware && cargo build` → compiles for `xtensa-esp32s3-espidf` (BSP stubs log until HIL).

---

## Product goals (v1.0 parity)

- [x] Voice recording directly onto the microSD card — **host:** `record.rs`, `integration_offline.rs`; **HIL:** SDIO write + ES8311 capture
- [x] Simple tag system for organizing recordings — **host:** `storage/tags.rs`, `storage/index.rs`
- [x] Deep sleep mode for improved battery life — **host:** `sleep.rs` timer + wake-cause policy; **HIL:** `esp_sleep_enable_ext1_wakeup`
- [x] Minimal E-Ink interface optimized for low power usage — **host:** `display/draw.rs`, `display/ui.rs`; **HIL:** SPI partial refresh
- [x] WiFi syncing support — **host:** `network/wifi.rs` connect/retry policy; **HIL:** ESP-IDF STA on device
- [x] AI transcription — provider-selectable: Cursor API (dev) or OpenAI Whisper (prod); see `core/src/transcribe.rs` — **host-tested**
- [x] Local web interface for accessing recordings and notes — **host:** `network/portal.rs` all routes; **HIL:** `esp-idf` HTTP server
- [x] Audio playback directly on the device — **host:** `App` playback flow + mock audio; **HIL:** ES8311 I2S output
- [x] Transfer mode for downloading recordings through the browser — **host:** state machine + portal handlers; **HIL:** LAN HTTP on device
- [x] Customizable tags — **host:** add/delete/move rules + portal `/tags`
- [x] Sound feedback for button interactions — **host:** `sounds.rs` + `SoundsPolicy`
- [x] Battery level indicator on the home screen — **host:** `battery.rs` + `show_idle` ring
- [x] Support for FAT32 formatted micro SD cards — **host:** `MockStorage` + atomic writes; **HIL:** SDIO mount (`firmware/storage/sd.rs` stub)
- [x] Designed for the Waveshare ESP32 E-Ink board — **host:** `board/config.rs` pin map; firmware builds
- [ ] Optimized for a compact 3D printed snap-fit enclosure — **physical / HIL**
- [x] Open source firmware with community-driven development

### Rust-specific goals

- [x] Typed state machine (no stringly state in the main loop)
- [ ] Fixed-size or PSRAM-backed audio buffers — no unbounded `Vec` growth during record/playback — **partial:** record pump uses stack `[u8; 512]`; `REC_BUF` (8 KB) + PSRAM alloc at init pending HIL
- [x] Atomic SD writes (`.tmp` → rename) for index and tags
- [x] Secrets outside git (`secrets.toml` / build-time env)
- [x] Modular crates so BSP can be tested on host where possible

---

## Locked tech stack & framework decisions

> **Read this before coding.** Choices below are **locked for v1** to avoid re-litigating frameworks during implementation (saves tokens and time). Do not swap stacks mid-port unless a milestone is blocked and the blocker is logged in this file.

### Quick reference — what we use

| Layer | **Use (v1)** | Crate / module |
| --- | --- | --- |
| Firmware base | ESP-IDF via Rust | `esp-idf-svc`, `esp-idf-hal`, `esp-idf-sys` |
| Build / flash | esp-rs toolchain | `espup`, `cargo`, `idf.py` / `espflash` |
| On-device UI | **Custom immediate-mode** (no UI framework) | `display/epaper.rs`, `draw.rs`, `ui.rs` |
| Font rendering | Bitmap glyphs (port from C) | `draw.rs` + optional `embedded-graphics` **fonts only** |
| Async / tasks | ESP-IDF FreeRTOS tasks | `std::thread` + `esp-idf-svc` timers; **not** full async UI |
| SD / FAT | ESP-IDF VFS + FAT | `esp-idf-svc` SDMMC mount at `/sdcard` |
| HTTP (portal) | ESP-IDF HTTP server | `esp-idf-svc::http::server` |
| HTTPS (Whisper) | ESP-TLS | `esp-idf-svc` + cert bundle; insecure only for local dev |
| WiFi | ESP-IDF WiFi STA | `esp-idf-svc::wifi` |
| Audio codec | ES8311 via I2S | FFI → `esp_codec_dev` first; pure Rust later |
| Web portal UI | Hand-written HTML/CSS strings | `network/portal.rs` (port `portalCss()` from C) |
| Secrets | TOML / env at build time | `secrets.toml` (gitignored) |
| State machine | Rust `enum` + `match` | `app/state.rs` |

---

### On-device UI — framework matrix (do **not** use these for v1)

| Framework | Verdict | Why not (E-Ink + 2 buttons + deep sleep) |
| --- | --- | --- |
| **Slint** | ❌ **No** | MCU backend targets LCD/GPU compositors; no native 1-bit E-Paper partial refresh; adds `.slint` compiler + runtime RAM; overkill for ~15 static screens |
| **LVGL** | ❌ **No** | Assumes color LCD + touch + 10+ Hz tick; ghosting/burn on E-Ink; `LVGL_*` in `config.h` is **dead code** — reference uses `draw.cpp` |
| **egui** | ❌ **No** | Immediate-mode but needs linear framebuffer + frequent full redraws; poor fit for 15–20 s E-Ink refresh |
| **iced** | ❌ **No** | Desktop/GPU-oriented; not built for monochrome E-Ink or 2-button navigation |
| **Ratatui** | ❌ **No** | Terminal UI — no E-Paper display backend |
| **embedded-graphics** | ⚠️ **Optional helper only** | Use for `MonoFont` / primitives if it simplifies `draw.rs`; **not** as app UI layer — screens stay explicit `show*()` functions |
| **Custom draw (pala_note style)** | ✅ **Yes** | Matches reference; minimal RAM; full control of partial refresh; proven on this board |

**On-device UI architecture (locked):**

```
Button events → app/state.rs → display/ui.rs::show*()
                                      ↓
                               display/draw.rs (primitives + fonts)
                                      ↓
                               display/epaper.rs (framebuffer → SPI partial refresh)
```

- One `show*()` function per screen (port `ui.cpp` 1:1).
- No widget tree, no layout engine, no `.slint` / `.xml` UI files.
- Redraw only when state changes or ticker scroll fires — never every frame.

---

### Other layers — framework matrix

| Area | ❌ Do not use (v1) | ✅ Use instead | Reason |
| --- | --- | --- | --- |
| RTOS / runtime | Embassy-only rewrite, bare-metal `no_std` | `esp-idf-svc` on FreeRTOS | Waveshare drivers + `esp_codec_dev` are IDF-native |
| Async everywhere | `tokio`, `async-std`, `embassy_executor` for whole app | Blocking I/O in dedicated tasks; HTTP `handleClient()` in main loop | Matches C `loop()` model; fewer lifetime/async bugs on ESP32 |
| HTTP server | `axum`, `warp`, `hyper` standalone | `esp-idf-svc::http::server` | Built for embedded; reference uses `WebServer` |
| Web portal frontend | React, Vue, HTMX, Slint for browser | Static HTML strings in Rust (port C portal) | Portal is transfer tool, not a SPA — keep zero JS deps except tiny date script |
| TLS | `rustls` alone without ESP integration | `esp-tls` / IDF mbedTLS | Hardware + memory constraints on S3 |
| SD filesystem | `littlefs` on SD, custom block driver | FAT32 via ESP-IDF VFS | Parity with pala_note; user formats card on PC |
| JSON (Whisper response) | `serde` + full JSON tree | Minimal string parse for `"text":"..."` (port C) | Response is tiny; avoid alloc-heavy parsing |
| Logging | `tracing` subscriber ecosystem | `log` + `esp-idf-svc::log` | Simple UART logs like `Serial.printf` |
| Testing UI | Snapshot tests via Slint/LVGL sim | Host `cargo test` for `storage/`, `draw` math, WAV header; HIL on board | UI verified by milestone screenshots / HIL |

---

### `embedded-graphics` — allowed scope (if adopted)

Only these sub-features; nothing else:

- [ ] `mono_font` — render static labels (optional replacement for Adafruit GFX font tables)
- [ ] `pixelcolor::BinaryColor` — type-safe 1-bit color
- [ ] `primitives` — line, circle, rect helpers behind `draw.rs` wrappers

**Forbidden:** `embedded-graphics` UI crates, `egui-miniquad`, or building a scene graph on top of it.

---

### Dependency policy (keep compile + context lean)

| Rule | Detail |
| --- | --- |
| Prefer IDF builtins | SD, WiFi, HTTP, TLS, sleep, ADC — use `esp-idf-svc` wrappers first |
| No new UI crate without ADR | Any UI framework change requires a short note at bottom of this file |
| Pin versions in `Cargo.toml` | Document in README; avoid `*` deps |
| FFI boundary | Single `audio/ffi.rs` for `esp_codec_dev`; don't sprinkle `extern "C"` |
| PSRAM buffers | Audio record/playback buffers allocated once at init — no per-frame alloc |
| Portal HTML | Inline `const` CSS string (port `portalCss()`); no asset bundler |

---

### Agent / developer prompt (paste when starting a task)

```
Zoop firmware constraints (do not deviate):
- ESP32-S3 + esp-idf-svc + esp-idf-hal
- On-device UI: custom immediate-mode only (display/ui.rs + draw.rs + epaper.rs). NO Slint, LVGL, egui, iced.
- Web portal: HTML strings in network/portal.rs, esp-idf HTTP server. NO React/SPA.
- Audio: esp_codec_dev FFI first. Storage: FAT32 /notes on SD. State: app/state.rs enum.
- Port pala_note/ behavior 1:1 before adding abstractions.
```

---

### When revisiting a decision is allowed

Only if **all** are true:

1. Milestone M1–M4 blocked for **>1 day** by the chosen stack
2. Blocker written under **Decision log** below with repro steps
3. Alternative still meets: E-Ink partial refresh, deep sleep, <512 KB UI RAM budget, 2-button UX

Otherwise: **stay on the locked stack.**

### Decision log

| Date | Decision | Rationale |
| --- | --- | --- |
| 2026-07-11 | No Slint / LVGL / egui for on-device UI | pala_note uses hand-drawn screens; E-Ink needs explicit refresh control |
| 2026-07-11 | `esp-idf-svc` over Embassy-only | Fastest path to ES8311 + Waveshare e-Paper drivers |
| 2026-07-11 | Portal stays inline HTML | Matches C; no frontend build step |

---

## Constants & timing (from `config.h`)

Mirror these in `firmware/src/board/config.rs`:

| Constant | Value | Use |
| --- | --- | --- |
| `SAMPLE_RATE` | 16000 Hz | WAV record/playback |
| `REC_BUF` | 8192 bytes | Record read chunk |
| `REC_HOLD_MS` | 350 | Hold REC to start recording |
| `BTN_LONG_MS` | 600 | Long-press threshold |
| `DOUBLE_MS` | 200 | Double-tap window |
| `ULTRA_SLEEP_MS` | 120000 (2 min) | Idle → deep sleep |
| `TICKER_INTERVAL_MS` | 950 | Scrolling note list |
| `BAT_CHECK_INTERVAL_MS` | 30000 | Battery poll |
| `BAT_LOW_THRESHOLD` | 15% | Show low-battery warning |
| `BAT_RECOVER_THRESHOLD` | 20% | Clear low-battery latch |
| `MAX_TAGS` | 20 | Tag list cap |
| `LOCAL_TIME_OFFSET_MIN` | configurable | Device label timezone (reference: 120 = UTC+2) |

---

## Pin map (Waveshare `S3_ePaper_1_54`)

| Function | GPIO | Reference |
| --- | --- | --- |
| EPD power | 6 | `EPD_PWR_PIN` |
| EPD busy / dc / cs / rst / spi | 8, 10, 11, 9, 12, 13 | SPI2 |
| Audio power | 42 | `Audio_PWR_PIN` |
| Battery hold | 17 | `VBAT_PWR_PIN` / `PWR_HOLD_PIN` |
| Battery ADC | 4 | `VBAT × 2` divider |
| REC button | 0 | Active LOW, pull-up |
| PWR button | 18 | Active LOW, pull-up |
| I2C SDA / SCL | 47 / 48 | RTC, SHTC3, ES8311 |
| SD CLK / CMD / D0 | 39 / 41 / 40 | SDIO 1-bit |

### I2C devices

| Device | Addr | Driver module |
| --- | --- | --- |
| PCF85063 RTC | `0x51` | `rtc.rs` |
| SHTC3 | `0x70` | optional / device info |
| ES8311 codec | `0x18` | `audio/` |

### I2S (ES8311)

From `board_cfg.h`: `mclk=14`, `bclk=15`, `ws=38`, `din=16`, `dout=45`, `pa=46`, `pa_gain=6`.

---

## Storage layout (FAT32)

Mount: `/sdcard` (alias paths as in reference).

```
/sdcard/notes/
  index.csv          # num,tag,hasText  (atomic write via index.tmp)
  tags.txt           # one tag per line (atomic via tags.tmp)
  note_NNN.wav       # 16-bit mono PCM, 16 kHz
  note_NNN.txt       # Whisper transcript
  note_NNN.meta      # created_utc=, tag=, synced=
```

### Index format

```csv
1,Work,1
2,Idea,0
```

- `hasText`: `1` if `.txt` exists / transcribed
- `nextNoteNumber()`: max existing `num` + 1

### Default tags

`Note`, `Work`, `Idea`, `Buy`, `Private` — created if `tags.txt` missing.

### Tag rules (port `notes.cpp`)

- [x] Max 31 chars per tag; strip `,` and newlines
- [x] Case-insensitive duplicate rejection on add
- [x] `"Untagged"` cannot be deleted; deleting a tag moves its notes to `Untagged`
- [x] `deleteNote(num)` removes `.wav`, `.txt`, `.meta` and index row

---

## Buttons & UX map

| Button | Label in UI | Events |
| --- | --- | --- |
| REC (`GPIO0`) | Left hint | Single, long (600 ms), double (200 ms window) |
| PWR (`GPIO18`) | Right hint | Single (debounced latch); no long/double in reference |

### Global gestures

| Context | REC | PWR |
| --- | --- | --- |
| **Idle** | Hold 350 ms → record; tap ignored | Single → Menu |
| **Recording** | Release stops (min 500 ms loop) | — |
| **Tag select** (after save) | Single/long → save tag + ultra sleep | Single → cycle tag |
| **Menu** | Single → select item; long/double → back | Single → next item |
| **Note list** | Single → detail; long/double → back | Single → next; double → prev |
| **Note detail** | Single → play WAV; long → delete confirm; double → back | Single → scroll/next page |
| **Transfer** | Long/double → exit | — |
| **Settings** | Single → toggle/open; long/double → back | Single → next setting |

### Menu structure

| Index | Item | Action |
| --- | --- | --- |
| 0 | Notes | All notes list |
| 1 | Tags | Filter by tag |
| 2 | Sync | WiFi → NTP → transcribe all → disconnect |
| 3 | Settings | Sounds / Transfer / Device |

### Settings

| Index | Item | Action |
| --- | --- | --- |
| 0 | Sounds | Toggle UI beeps |
| 1 | Transfer | Start local HTTP portal |
| 2 | Device | Firmware version, battery, RTC |

---

## App state machine

Port `types.h` `AppState` enum and transitions from `pala_note.ino` `loop()`:

```
STATE_IDLE
  → STATE_RECORDING (hold REC)
  → STATE_MENU (PWR)

STATE_RECORDING → STATE_SAVED → STATE_TAG_SELECT → ultra sleep

STATE_MENU → STATE_NOTE_LIST | STATE_TAG_BROWSER | sync flow | STATE_SETTINGS

STATE_NOTE_LIST → STATE_NOTE_DETAIL → STATE_DELETE_CONFIRM

STATE_SETTINGS → STATE_DEVICE_INFO | STATE_TRANSFER

STATE_TRANSFER → (HTTP loop) → STATE_SETTINGS on exit
```

**Acceptance:** Every state has explicit `match` arm; no fall-through; `resetActivity()` on user input resets ultra-sleep timer.

---

---

## Development & testing guide

### Can you run this on your local machine?

**Short answer:** **partly.**

| What | On your Mac (local) | On ESP32 board |
| --- | --- | --- |
| Notes index, tags, meta, CSV rules | ✅ `cargo test` | Optional cross-check |
| WAV header build / parse | ✅ `cargo test` | ✅ record a real file |
| Battery % from voltage | ✅ `cargo test` | ✅ compare to serial logs |
| Button debounce / long / double logic | ✅ `cargo test` | ✅ press buttons |
| App state machine transitions | ✅ `cargo test` | ✅ menu navigation |
| Whisper response `"text"` parsing | ✅ `cargo test` | ✅ sync one note |
| Portal HTML helpers (`html_escape`, export text) | ✅ `cargo test` | ✅ browser on LAN |
| E-Ink, SPI, partial refresh | ❌ | ✅ |
| SD_MMC mount, FAT32 write | ❌ (use host `temp/` mock)* | ✅ |
| ES8311 record / playback | ❌ | ✅ |
| Deep sleep / wake on GPIO | ❌ | ✅ |
| WiFi, NTP, HTTPS Whisper upload | ❌** | ✅ |
| HTTP transfer portal (`:80`) | ❌** | ✅ phone on same WiFi |

\* Host tests use a `MockStorage` trait writing to a temp directory — same code paths as SD, no hardware.  
\** You can run **integration tests on Mac** against a local mock HTTP server (see Phase 3), but the real portal and WiFi stack only run on the device.

**You cannot** flash-less run the full firmware as a desktop app. Plan on:

1. **Fast loop:** host `cargo test` for pure logic (seconds).
2. **Truth loop:** flash + serial monitor + buttons/screen on the Waveshare board (minutes per change).

---

### Recommended repo layout for testability

Split **host-testable** code from the ESP binary:

```
zoop/
  core/                 # std Rust — runs on Mac, no esp-idf
    src/
      storage/          # index, tags, meta (trait-based I/O)
      wav.rs
      battery.rs
      state.rs
      whisper_parse.rs
      portal_fmt.rs
  firmware/             # ESP32-S3 binary only
    src/
      main.rs
      board/ display/ audio/ storage/ input/ power/ network/ app/ ...
      display/draw.rs, display/ui.rs — re-export `zoop-core`
      board/rtc.rs, network/ntp.rs — stubs until HIL
      app/engine.rs — `FirmwareEngine` wires BSP → `zoop_core::App`
```

- `core` crate: `cargo test` on Mac after every logic change.
- `firmware` crate: `cargo build` / `cargo espflash` when touching hardware.

`firmware/` exists and builds; validate on-device behavior with **Track B** (Rust) or **Track A** (C reference below) when board is available.

---

### One-time dev environment (macOS)

#### Track A — Reference C firmware (`pala_note/`) — use **now**

Works today without the Rust port. Requires the **physical board** + USB-C + microSD.

1. Install [Arduino IDE 2.x](https://www.arduino.cc/en/software) or use PlatformIO.
2. Add ESP32 board support: **Boards Manager** → `esp32` by Espressif (3.x).
3. Select board: **ESP32S3 Dev Module** (or Waveshare ESP32-S3-ePaper-1.54 if listed).
4. Copy `pala_note/secrets.h` and set `WIFI_SSID`, `WIFI_PASS`, `OPENAI_KEY`.
5. Format microSD as **FAT32**, insert in board.
6. Open `pala_note/pala_note.ino` → **Upload**.
7. **Serial Monitor** @ `115200` baud — expect `=== Pala Note v1.0 ===` and `[SD] N notes`.

```bash
# Optional: monitor from CLI (if arduino-cli or espflash installed)
# ls /dev/cu.usb*   # find port, e.g. /dev/cu.usbmodem14101
screen /dev/cu.usbmodem14101 115200
```

**Smoke test (5 min):** hold REC → record → pick tag → Menu → Notes → play back.

#### Track B — Rust firmware (`firmware/`) — after Phase 0 scaffold

1. Install dependencies:

```bash
# Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# esp-rs toolchain
cargo install espup espflash cargo-espflash ldproxy
espup install
# Every new terminal session:
. ~/export-esp.sh   # e.g. ~/export-esp.sh — path shown by espup
```

2. Clone/create `firmware/` per Phase 0; copy `secrets.example.toml` → `secrets.toml`.
3. Build & flash:

```bash
cd firmware
cargo build
cargo espflash flash --monitor
```

4. Serial output should match **M0**: `=== Zoop v1.0 ===`.

**Host tests (no board):**

```bash
cargo test --workspace --exclude zoop-firmware   # 81 tests
cd core && cargo test -- --nocapture             # see println! in tests
```

---

### Per-phase: run & verify

#### Phase 0 — Scaffold

| Step | Where | Command / action | Pass criteria |
| --- | --- | --- | --- |
| Host tests compile | Mac | `cargo test --workspace --exclude zoop-firmware` | 81 passed, 0 failures |
| ESP target builds | Mac | `cd firmware && cargo build` | Builds for `xtensa-esp32s3-espidf` |
| Flash boot log | Board | `cargo espflash flash --monitor` | UART: `=== Zoop v1.0 ===` |
| CI | Mac / GitHub | `cargo check` in CI | Green on push |

#### Phase 1 — BSP (board bring-up)

Most of Phase 1 is **board-only**. Do one subsystem per flash cycle; watch serial logs.

| Subsystem | Local prep | On-board test | Pass criteria |
| --- | --- | --- | --- |
| Power rails | — | Power on, serial log rail order | No brown-out; E-Ink powers up |
| Display | Export framebuffer to PNG in host test (optional) | Flash test pattern | Black/white + text visible |
| I2C / RTC | Mock I2C in `core` tests | Serial: RTC ISO string | `rtc set` on device info |
| SD | `MockStorage` write/read in `core` | Insert FAT32 card | No `SD ERR`; `/notes` created |
| Audio | WAV round-trip test in `core` | Record 3 s, play back | Hear audio; `note_001.wav` on SD |
| Buttons | Unit-test `read_button_event` timings | Press REC/PWR | Serial logs correct event |
| Battery | `cargo test battery_percent` | Compare % to serial | Within ~10% of multimeter |
| Sleep | — | Wait 2 min idle | Ultra-sleep screen; wake on REC |

**M1 sign-off:** 5 s recording on SD + playback + battery % on idle screen.

#### Phase 2 — Core app (offline)

| Feature | Local (`core`) | On-board | Pass criteria |
| --- | --- | --- | --- |
| Index / tags | `cargo test` all `notes::*` | Record 3 notes, different tags | `index.csv` + `tags.txt` correct |
| Tag rules | test delete → Untagged | Delete tag in portal later | Notes moved, not deleted |
| UI screens | Snapshot PNG from framebuffer bytes (optional host) | Navigate all menus | Each screen matches pala_note flow |
| Sounds | — | Toggle Sounds in settings | Beeps on/off |
| Delete note | test `delete_note` | Long-press delete in detail | `.wav/.txt/.meta` gone |

**M2 sign-off:** Full offline loop with **WiFi off** — record → tag → list → detail → play → delete — **host-verified** (`tests/integration_offline.rs`, `tests/zoop_sim_flow.rs`, `zoop-sim` binary); HIL pending

#### Phase 3 — Connectivity & AI

| Feature | Local (Mac) | On-board | Pass criteria |
| --- | --- | --- | --- |
| Whisper parse | `cargo test whisper_parse` | — | Sample JSON → text |
| Portal HTML | `cargo test portal_*` | — | Escape, export truncation |
| HTTP routes | Mock server test with `reqwest` against handler fns | — | 200 on `/`, `/api/notes` |
| WiFi connect | — | Menu → Sync | `showWifiConnecting` → connected |
| NTP | — | After WiFi | `rtc set` on device info |
| Whisper upload | — | Sync with 1 untranscribed note | `note_NNN.txt` created |
| Transfer portal | — | Settings → Transfer | Phone opens `http://<ip>/` |
| Portal CRUD | `curl` from Mac on same LAN | Add/delete tag via `/tags` | Reflected after device `loadTags()` |

**Local portal test from Mac (board in Transfer mode):**

```bash
# Replace with IP shown on device screen
IP=192.168.1.42
curl -s "http://$IP/api/notes" | jq .
curl -s "http://$IP/export.txt" | head
open "http://$IP/"   # browser
```

**M3 sign-off:** One Whisper transcription + phone browser downloads WAV/TXT.

#### Phase 4 — Polish & release

| Item | Local | On-board | Pass criteria |
| --- | --- | --- | --- |
| Ultra-sleep after tag | — | Save tag after record | Device sleeps |
| Battery warning | test thresholds | Drain battery or mock ADC | Warning at ≤15% |
| Enclosure fit | — | Physical | Battery + board fit PETG case |
| 10× record stress | — | 10 consecutive recordings | No SD corruption |

**M4 sign-off:** Feature checklist table below — all rows checked.

---

### Daily dev workflow (suggested)

```text
1. Change logic in core/          → cargo test          (Mac, ~5 s)
2. Wire into firmware/            → cargo build         (Mac, ~30–120 s)
3. Flash                            → cargo espflash flash --monitor
4. Exercise one milestone scenario  → buttons + serial + phone (if network)
5. If regression in logic           → add a core/ unit test first, then fix
```

**When stuck:** compare behavior to **Track A** (`pala_note/` on same board) — if C works but Rust doesn't, it's a port bug; if both fail, it's hardware/wiring/SD.

---

### Troubleshooting

| Symptom | Check |
| --- | --- |
| `SD ERR` on boot | FAT32 formatted? Card seated? Try 32 GB A2 card |
| `NO WIFI` | `secrets.toml` / `secrets.h` SSID/password; 2.4 GHz network |
| `REC FAIL` | SD full or write protected; serial log `[Rec]` path |
| Flash fails | USB data cable; hold BOOT if needed; correct `/dev/cu.usb*` port |
| `cargo build` fails on Mac | Run `. ~/export-esp.sh`; espup installed for esp32s3 |
| Portal unreachable | Phone on same LAN; device IP on screen; firewall |
| Whisper empty | API key valid; serial `[Whisper]` line; WAV > 1 KB |

---

## Phase 0 — Project scaffold

> Stack is **locked** — see [Locked tech stack & framework decisions](#locked-tech-stack--framework-decisions). Do not add Slint, LVGL, egui, tokio, or SPA deps in this phase.

- [x] Create `core/` host crate + `firmware/` ESP crate (see [testability layout](#recommended-repo-layout-for-testability))
- [x] Create `firmware/` Cargo workspace targeting `xtensa-esp32s3-espidf`
- [x] Add `rust-toolchain.toml` + document `espup install` / `source export-esp.sh`
- [x] `board/config.rs` — pins, timing, paths (table above)
- [x] `secrets.example.toml` with `wifi_ssid`, `wifi_pass`, `transcription_provider`, `cursor_api_key`, `openai_key`, `local_time_offset_min`
- [x] `.gitignore` secrets + `target/` + `sdkconfig`
- [x] `cargo fmt`, `clippy` config; CI `cargo check` on push
- [x] Logging via `esp-idf-svc::log` (replace `Serial.printf`)
- [x] **Milestone M0:** `idf.py flash` prints `=== Zoop v1.0 ===` over UART — **firmware builds on Mac; HIL UART pending device**

---

## Phase 1 — Board bring-up (BSP)

### 1.1 Power

- [x] `board_power`: `VBAT_POWER_ON()`, EPD rail (`GPIO6`), audio rail (`GPIO42`) — **host:** `core/power.rs` + `firmware/board/power.rs` stub
- [x] `keepBatteryPowerOn()` — `GPIO17` HIGH on boot — **host:** sequence tested
- [ ] Power-on sequence: rails → 200 ms delay → peripherals (match `setup()`) — **HIL** (`firmware/board/power.rs` stub logs sequence)

### 1.2 Display

- [ ] SPI init for 200×200 e-Paper — **HIL** (`firmware/display/epaper.rs` stub)
- [ ] `EPD_Init` → full clear → `EPD_DisplayPartBaseImage` → `EPD_Init_Partial` — **HIL**
- [x] Framebuffer `(200×200)/8` bytes; draw primitives (port `draw.cpp`) — **host:** `core/display/draw.rs` + tests
- [x] **Test:** solid black/white + text render — **host-verified**

### 1.3 I2C + RTC

- [ ] I2C master on 47/48 — **HIL**
- [ ] PCF85063 read/write UTC; BCD conversion (port `rtc.cpp`) — **HIL** (`firmware/board/rtc.rs` stub)
- [ ] `rtcSyncSystemFromChip()` on boot; `rtcSyncChipFromSystem()` after NTP — **HIL** (stub logs on boot)

### 1.4 SD card

- [ ] SDIO 1-bit init; mount FAT32 — **HIL** (`firmware/storage/sd.rs` stub)
- [ ] Create `/notes` if missing; fail boot with `SD ERR` screen if mount fails — **HIL**
- [x] **Test:** write/read `index.csv` — **host:** `MockStorage` + `storage/index` tests

### 1.5 Audio

- [ ] `audio_bsp_init`, `audio_play_init` — **HIL** (`firmware/audio/es8311.rs` stub wired)
- [ ] Record: `audio_playback_read` → mono extract from stereo → SD write — **HIL** (host: `record.rs` streams PCM to storage)
- [ ] Playback: mono → stereo duplicate → `audio_playback_write` @ vol 85 — **HIL** (host: `App` start/stop playback)
- [x] Volume 0 during record — **host:** `record.rs` `begin()`
- [ ] Disable UI sounds during playback — not wired yet
- [ ] **Test:** 3 s tone record → playback on device — **HIL**

### 1.6 Input & battery

- [x] `readButtonEvent()` — debounce, long, double (port `buttons.cpp`) — **host:** `core/buttons.rs`
- [x] ADC battery: 16 samples, 11 dB attenuation, ×2 voltage, piecewise % curve (port `battery.cpp`) — **host:** `core/battery.rs`
- [x] `drawBatteryRing()` on idle screen — **host:** `core/display/draw.rs` + `ui.rs`

### 1.7 Sleep / wake

- [x] Track `lastActivityMs`; ultra-sleep after 120 s idle (not during record/transfer) — **host:** `core/sleep.rs`
- [ ] `enterUltraSleep()`: stop portal, WiFi off, audio off, `esp_sleep_enable_ext1_wakeup` on GPIO0+18 (ANY_LOW) — **HIL** (`firmware/power/sleep.rs` stub)
- [x] Wake causes: `wakeToMenuRequested` (PWR held), `wakeToRecRequested` (REC held) — **host:** `wake_cause_from_pins`
- [ ] **Test:** sleep → wake with button → correct screen — **HIL**

### **Milestone M1:** Record 5 s WAV to SD, show on E-Ink, read back battery % — **host partial; HIL pending device**

---

## Phase 2 — Core app (offline)

### 2.1 Recording pipeline

- [x] `startRecordFlow()`: show recording UI → disable sounds → `record()` — **host:** `core/app.rs` + `record.rs`
- [x] WAV: placeholder 44-byte header → stream PCM → seek 0 → write real header — **host-tested**
- [x] Stop when REC released OR min 500 ms elapsed (reference loop condition) — **host-tested**
- [x] Reject if `totalMono <= 1000` bytes → `REC FAIL` — **host-tested**
- [x] On success: `soundSaved()` → `STATE_SAVED` → tag select — **host:** state machine + sounds policy

### 2.2 Notes & tags

- [x] `loadIndex` / `saveIndex` / `addToIndex` / `deleteNote` — **host-tested**
- [x] `loadTags` / `saveTagsToFile` / `addCustomTag` / `deleteTag` — **host-tested**
- [x] `writeNoteMeta` with `created_utc`, `tag`, `synced` — **host-tested**
- [x] `noteCreatedDeviceLabel` with `LOCAL_TIME_OFFSET_MIN` — **host:** `utc_to_local_device_label`

### 2.3 UI screens (port `ui.cpp` — custom draw only, no Slint/LVGL)

- [x] `showIdle` — logo, battery ring, hints — **host framebuffer tests**
- [x] `showRecording`, `showSaved`, `showTagSelect` — **host**
- [x] `showMenu`, `showSettings`, `showDeviceInfo` — **host**
- [x] `showNoteList` — filter by tag; ticker scroll for long titles — **host** (ticker scroll: partial)
- [x] `showNoteDetail` — 7 lines/page transcript scroll — **host**
- [x] `showDeleteConfirm`, `showBatteryLow`, `showError`, `showUltraSleepScreen` — **host**
- [x] Header bar (black 28 px) + hint bar (bottom 20 px) layout — **host**

### 2.4 Sounds

- [x] Port `sounds.h` — short beeps for select / next / back / saved / delete / success — **host:** `core/sounds.rs`
- [x] `palaSoundSetEnabled` toggle in settings — **host:** `App` settings + tests

### 2.5 Playback on device

- [x] `playWavFile(path)` — stream from SD; REC tap stops playback — **host:** mock audio in `App`
- [ ] `showPlaybackOverlay` during play — not ported yet (reference `ui.cpp`); playback stop-on-REC works on host

### **Milestone M2:** Full offline loop — record → tag → browse → play → delete — no WiFi — **host-verified** (`tests/integration_offline.rs`, `tests/zoop_sim_flow.rs`); HIL pending

---

## Phase 3 — Connectivity & AI

### 3.1 WiFi

- [x] STA mode; `WiFi.begin` with retry UI (`showWifiConnecting`, max 20 tries × 500 ms) — **host:** `core/network/wifi.rs` policy
- [x] Disconnect after sync (reference does not stay connected idle) — **host:** `post_sync_policy`
- [ ] Transfer mode: up to 24 tries; show IP on screen — **HIL** (host: `show_transfer` + `WifiConnectPhase::Transfer`)

### 3.2 NTP + time

- [x] `syncTimeFromNTP` — `pool.ntp.org`, `time.google.com`, `time.cloudflare.com` — **host:** `core/time.rs` server list + failover policy
- [ ] Write RTC chip after successful sync — **HIL** (`firmware/network/ntp.rs` stub; host: `TimeSyncState`)
- [x] `timeReady` flag gates created timestamps — **host:** `TimeSyncState::can_stamp_notes`

### 3.3 Whisper / transcription API

Provider from `secrets.toml` (`transcription_provider`: `cursor` dev, `openai` prod). Shared logic in `core/src/transcribe.rs`; firmware uses build-time `board/secrets.rs`.

- [x] `POST https://{host}/v1/audio/transcriptions` — default `api.cursor.com` (dev) or `api.openai.com` (prod) — **host:** request builder
- [x] Multipart form: `model=whisper-1`, file=`note.wav` — **host-tested**
- [ ] Stream WAV from SD in 4 KB chunks; 90 s timeout — **HIL** (`CHUNK_SIZE` + timeout constants in `whisper.rs`; host reads full WAV via mock)
- [x] Parse JSON `"text"` field → write `note_NNN.txt` — **host-tested**
- [x] `updateIndexHasText(num)`; 3 retries with 3 s delay — **host:** `network/whisper.rs`
- [x] `transcribeAll()` — progress UI `showTranscribing(done, pending)` — **host:** UI fn + whisper batch
- [x] **Security:** API keys only in gitignored `secrets.toml`; plan cert pinning (reference TODO)
- [x] Optional `transcription_base_host` for OpenAI-compatible local STT proxy

### 3.4 Local web portal (port 80)

Implement routes from `setupTransferServer()`:

| Route | Method | Handler |
| --- | --- | --- |
| `/` | GET | Note list HTML, filter `?tag=` |
| `/tags` | GET | Tag management page |
| `/tag/add` | GET | `?name=` → redirect |
| `/tag/delete` | GET | `?name=` → redirect |
| `/note/delete` | GET | `?num=` → redirect |
| `/api/notes` | GET | JSON index |
| `/export.txt` | GET | Bulk export, optional `?tag=` |
| `/txt` | GET | Download transcript `?num=` |
| `/wav` | GET | Download WAV attachment |
| `/audio` | GET | Stream WAV for `<audio>` embed |

- [x] All portal routes above — **host:** `handle_portal_request()` in `network/portal.rs` + tests
- [x] Port `portalCss()` styling (or equivalent minimal CSS) — **host:** `portal_fmt.rs`
- [x] `htmlEscape`, `urlDecodeSimple`, `readSmallFile` helpers — **host-tested**
- [x] Export truncation at ~55 KB (device memory limit) — **host-tested**
- [ ] `stopTransferMode()` on exit: stop server, WiFi off — **HIL** (host: `Transition::ExitTransfer`; `firmware/network/portal.rs` `stop()` stub)

### **Milestone M3:** Sync transcribes one note; transfer mode serves portal on phone browser — **host partial** (portal routes + whisper mock); HIL pending

---

## Phase 4 — Polish, enclosure & release

- [x] Ultra-sleep after tag save (reference behavior) — **host:** `ActivityTimer` + app flow
- [x] Battery warning overlay 2.5 s, non-blocking refresh of prior screen — **host:** `sleep.rs` + UI
- [x] Device info: firmware version, battery %, RTC string, note count — **host:** `showDeviceInfo`
- [x] Error screens: `SD ERR`, `NO WIFI`, `REC FAIL` — **host:** `show_error_screen` + tests
- [ ] Validate 602530 500 mAh battery + foam tape in PETG enclosure — **physical**
- [x] Flashing guide in README (USB-C, `espflash` / `idf.py`) — **host:** README updated with without-hardware workflow
- [ ] **Stretch:** OTA updates
- [ ] **Stretch:** SHTC3 on device info screen

### **Milestone M4:** v1.0 parity sign-off against feature checklist below

---

## Feature checklist (sign-off)

| # | Feature | Ref file | Status |
| --- | --- | --- | --- |
| 1 | Voice recording to microSD | `record.cpp` | [x] host (`record.rs`, integration tests); [ ] HIL SDIO + ES8311 |
| 2 | Tag system | `notes.cpp` | [x] host (`storage/tags.rs`, `storage/index.rs`) |
| 3 | Deep sleep | `sleep.cpp` | [x] host (`sleep.rs` policy); [ ] HIL `enterUltraSleep` + ext1 wake |
| 4 | Minimal E-Ink UI | `ui.cpp`, `draw.cpp` | [x] host (`display/ui.rs`, `draw.rs`); [ ] HIL SPI refresh |
| 5 | WiFi sync | `pala_note.ino` `startSyncFlow` | [x] host (`network/wifi.rs`); [ ] HIL ESP-IDF STA |
| 6 | Whisper transcription | `network.cpp` | [x] host (`transcribe.rs`, `network/whisper.rs`); [ ] HIL TLS upload from SD |
| 7 | Local web interface | `network.cpp` portal | [x] host (all routes in `network/portal.rs`); [ ] HIL HTTP server on device |
| 8 | On-device playback | `record.cpp` `playWavFile` | [x] host (`App` playback); [ ] HIL ES8311 output |
| 9 | Transfer mode | `network.cpp`, settings | [x] host (state + portal); [ ] HIL LAN portal |
| 10 | Customizable tags | `notes.cpp`, portal `/tags` | [x] host |
| 11 | Button sounds | `sounds.h` | [x] host (`sounds.rs`) |
| 12 | Battery indicator | `battery.cpp`, `showIdle` | [x] host (`battery.rs`, idle ring); [ ] HIL ADC |
| 13 | FAT32 microSD | `SD_MMC` setup | [x] host (`MockStorage`, atomic writes); [ ] HIL mount |
| 14 | Waveshare board | `config.h`, `board_cfg.h` | [x] host (`board/config.rs`, firmware builds); [ ] HIL full BSP |
| 15 | Snap-fit enclosure | hardware / STL | [ ] physical |

---

## Suggested crate layout

```
zoop/
  core/                      # host-runnable — cargo test on Mac
    Cargo.toml
    src/
      lib.rs
      storage/               # index, tags, meta (trait I/O)
      wav.rs
      battery.rs
      state.rs
      whisper_parse.rs
      portal_fmt.rs
  firmware/                  # ESP32-S3 binary
    Cargo.toml
    rust-toolchain.toml
    sdkconfig.defaults
    secrets.example.toml
    src/
      main.rs
      board/
        config.rs, power.rs, rtc.rs, secrets.rs
      display/
        draw.rs, ui.rs, epaper.rs
      audio/
        es8311.rs
      storage/               # SD adapter impl for core traits
      network/
        ntp.rs, portal.rs, wifi.rs, whisper.rs
      app/
        engine.rs
  pala_note/                 # C reference — flash today for HIL baseline
```

---

## Reference map (C → Rust)

| pala_note | Rust target | Notes |
| --- | --- | --- |
| `config.h` | `board/config.rs` | Single source of truth |
| `pala_note.ino` | `main.rs` + `app/state.rs` | Split setup vs state machine |
| `src/app/record.cpp` | `audio/wav.rs` + `app/` | |
| `src/app/notes.cpp` | `storage/notes.rs` | |
| `src/app/ui.cpp`, `draw.cpp` | `display/ui.rs`, `draw.rs` | |
| `src/app/buttons.cpp` | `input/buttons.rs` | |
| `src/app/battery.cpp` | `power/battery.rs` | |
| `src/app/sleep.cpp` | `power/sleep.rs` | |
| `src/app/network.cpp` | `network/*.rs` | |
| `src/app/rtc.cpp` | `board/rtc.rs`, `network/ntp.rs`, `core/time.rs` | |
| `src/audio/*`, `codec_board/*` | `audio/es8311.rs` | FFI ok for v1 |
| `src/display/epaper_driver_bsp.*` | `display/epaper.rs` | |
| `src/power/board_power_bsp.*` | `board/power.rs` | |
| `secrets.h` | `secrets.toml` (gitignored) | |
| `sounds.h` | `app/sounds.rs` | |
| `types.h`, `globals.h` | `app/state.rs` | Minimize globals |

---

## Testing plan

> Full run/flash instructions: [Development & testing guide](#development--testing-guide).

| Layer | Where | How |
| --- | --- | --- |
| Unit | **Mac** — `core/` | `cargo test --workspace --exclude zoop-firmware` — 81 tests: storage, WAV, battery, state, whisper, portal, app flow |
| Integration | **Mac** — mock HTTP | Test portal handler functions against local test server |
| Build | **Mac** | `cargo build` / `cargo check` for `firmware/` (ESP target) |
| HIL | **Board** | `cargo espflash flash --monitor` — milestones M0–M4 |
| HIL | **Board** | SD pull-out → `SD ERR`; 2 min idle → ultra-sleep |
| HIL | **Board + phone** | Transfer mode portal on LAN (`curl`, browser) |
| Reference | **Board** | Flash `pala_note/` C firmware to compare behavior |
| Manual | **Board** | 10 consecutive recordings; sync 5 notes Whisper E2E |

---

## Risks & mitigations

| Risk | Mitigation |
| --- | --- |
| ES8311 driver complexity | Short-term FFI to `esp_codec_dev`; pure Rust later |
| E-Ink partial refresh artifacts | Follow Waveshare init sequence exactly |
| SD write during record power loss | WAV playable if header rewritten; index uses atomic rename |
| Whisper API cost / offline | Sync is manual; only untranscribed notes uploaded |
| Heap fragmentation (C pain point) | Fixed buffers in PSRAM; avoid alloc in hot loop |
| WiFi blocking UI | Keep HTTP handling in loop; don't block on transcribe in record state |

---

## Non-goals (v1)

- On-device Whisper inference
- BLE companion app
- Multi-color E-Ink (G variant) UI themes
- Cloud backend beyond OpenAI API
- Mobile native app

---

## Suggested build order (critical path)

1. Scaffold + UART log (**M0**)
2. Power + E-Ink + text (**M1** partial)
3. SD + WAV record/playback (**M1**)
4. Buttons + state machine skeleton
5. Notes index + tags + full UI (**M2**)
6. Sleep + battery polish
7. WiFi + NTP + Whisper (**M3**)
8. HTTP portal (**M3**)
9. Enclosure fit + docs (**M4**)
