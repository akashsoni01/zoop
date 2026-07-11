# Zoop — Palma Notes in Rust (IoT)

Port of the C/Arduino [`pala_note`](./pala_note/) firmware to **Rust** on the Waveshare ESP32-S3 e-Paper 1.54 board.

| | |
| --- | --- |
| Reference | `pala_note/` — v1.0 (2026-05-24) |
| Target board | Waveshare **ESP32-S3-ePaper-1.54** (`S3_ePaper_1_54` in `board_cfg.h`) |
| Hardware guide | [`README.md`](./README.md) — BOM, kit combos, upgrades |
| Firmware version | `v1.0` (match reference until Rust port diverges) |

---

## Product goals (v1.0 parity)

- [ ] Voice recording directly onto the microSD card
- [ ] Simple tag system for organizing recordings
- [ ] Deep sleep mode for improved battery life
- [ ] Minimal E-Ink interface optimized for low power usage
- [ ] WiFi syncing support
- [ ] AI transcription using the OpenAI Whisper API
- [ ] Local web interface for accessing recordings and notes
- [ ] Audio playback directly on the device
- [ ] Transfer mode for downloading recordings through the browser
- [ ] Customizable tags
- [ ] Sound feedback for button interactions
- [ ] Battery level indicator on the home screen
- [ ] Support for FAT32 formatted micro SD cards
- [ ] Designed for the Waveshare ESP32 E-Ink board
- [ ] Optimized for a compact 3D printed snap-fit enclosure
- [ ] Open source firmware with community-driven development

### Rust-specific goals

- [ ] Typed state machine (no stringly state in the main loop)
- [ ] Fixed-size or PSRAM-backed audio buffers — no unbounded `Vec` growth during record/playback
- [ ] Atomic SD writes (`.tmp` → rename) for index and tags
- [ ] Secrets outside git (`secrets.toml` / build-time env)
- [ ] Modular crates so BSP can be tested on host where possible

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

- [ ] Max 31 chars per tag; strip `,` and newlines
- [ ] Case-insensitive duplicate rejection on add
- [ ] `"Untagged"` cannot be deleted; deleting a tag moves its notes to `Untagged`
- [ ] `deleteNote(num)` removes `.wav`, `.txt`, `.meta` and index row

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

## Phase 0 — Project scaffold

> Stack is **locked** — see [Locked tech stack & framework decisions](#locked-tech-stack--framework-decisions). Do not add Slint, LVGL, egui, tokio, or SPA deps in this phase.

- [ ] Create `firmware/` Cargo workspace targeting `xtensa-esp32s3-espidf`
- [ ] Add `rust-toolchain.toml` + document `espup install` / `source export-esp.sh`
- [ ] `board/config.rs` — pins, timing, paths (table above)
- [ ] `secrets.example.toml` with `wifi_ssid`, `wifi_pass`, `openai_key`, `local_time_offset_min`
- [ ] `.gitignore` secrets + `target/` + `sdkconfig`
- [ ] `cargo fmt`, `clippy` config; CI `cargo check` on push
- [ ] Logging via `esp-idf-svc::log` (replace `Serial.printf`)
- [ ] **Milestone M0:** `idf.py flash` prints `=== Zoop v1.0 ===` over UART

---

## Phase 1 — Board bring-up (BSP)

### 1.1 Power

- [ ] `board_power`: `VBAT_POWER_ON()`, EPD rail (`GPIO6`), audio rail (`GPIO42`)
- [ ] `keepBatteryPowerOn()` — `GPIO17` HIGH on boot
- [ ] Power-on sequence: rails → 200 ms delay → peripherals (match `setup()`)

### 1.2 Display

- [ ] SPI init for 200×200 e-Paper
- [ ] `EPD_Init` → full clear → `EPD_DisplayPartBaseImage` → `EPD_Init_Partial`
- [ ] Framebuffer `(200×200)/8` bytes; draw primitives (port `draw.cpp`)
- [ ] **Test:** solid black/white + text render

### 1.3 I2C + RTC

- [ ] I2C master on 47/48
- [ ] PCF85063 read/write UTC; BCD conversion (port `rtc.cpp`)
- [ ] `rtcSyncSystemFromChip()` on boot; `rtcSyncChipFromSystem()` after NTP

### 1.4 SD card

- [ ] SDIO 1-bit init; mount FAT32
- [ ] Create `/notes` if missing; fail boot with `SD ERR` screen if mount fails
- [ ] **Test:** write/read `index.csv`

### 1.5 Audio

- [ ] `audio_bsp_init`, `audio_play_init`
- [ ] Record: `audio_playback_read` → mono extract from stereo → SD write
- [ ] Playback: mono → stereo duplicate → `audio_playback_write` @ vol 85
- [ ] Volume 0 during record; disable UI sounds during playback
- [ ] **Test:** 3 s tone record → playback on device

### 1.6 Input & battery

- [ ] `readButtonEvent()` — debounce, long, double (port `buttons.cpp`)
- [ ] ADC battery: 16 samples, 11 dB attenuation, ×2 voltage, piecewise % curve (port `battery.cpp`)
- [ ] `drawBatteryRing()` on idle screen

### 1.7 Sleep / wake

- [ ] Track `lastActivityMs`; ultra-sleep after 120 s idle (not during record/transfer)
- [ ] `enterUltraSleep()`: stop portal, WiFi off, audio off, `esp_sleep_enable_ext1_wakeup` on GPIO0+18 (ANY_LOW)
- [ ] Wake causes: `wakeToMenuRequested` (PWR held), `wakeToRecRequested` (REC held)
- [ ] **Test:** sleep → wake with button → correct screen

### **Milestone M1:** Record 5 s WAV to SD, show on E-Ink, read back battery %

---

## Phase 2 — Core app (offline)

### 2.1 Recording pipeline

- [ ] `startRecordFlow()`: show recording UI → disable sounds → `record()`
- [ ] WAV: placeholder 44-byte header → stream PCM → seek 0 → write real header
- [ ] Stop when REC released OR min 500 ms elapsed (reference loop condition)
- [ ] Reject if `totalMono <= 1000` bytes → `REC FAIL`
- [ ] On success: `soundSaved()` → `STATE_SAVED` → tag select

### 2.2 Notes & tags

- [ ] `loadIndex` / `saveIndex` / `addToIndex` / `deleteNote`
- [ ] `loadTags` / `saveTagsToFile` / `addCustomTag` / `deleteTag`
- [ ] `writeNoteMeta` with `created_utc`, `tag`, `synced`
- [ ] `noteCreatedDeviceLabel` with `LOCAL_TIME_OFFSET_MIN`

### 2.3 UI screens (port `ui.cpp` — custom draw only, no Slint/LVGL)

- [ ] `showIdle` — logo, battery ring, hints
- [ ] `showRecording`, `showSaved`, `showTagSelect`
- [ ] `showMenu`, `showSettings`, `showDeviceInfo`
- [ ] `showNoteList` — filter by tag; ticker scroll for long titles
- [ ] `showNoteDetail` — 7 lines/page transcript scroll
- [ ] `showDeleteConfirm`, `showBatteryLow`, `showError`, `showUltraSleepScreen`
- [ ] Header bar (black 28 px) + hint bar (bottom 20 px) layout

### 2.4 Sounds

- [ ] Port `sounds.h` — short beeps for select / next / back / saved / delete / success
- [ ] `palaSoundSetEnabled` toggle in settings

### 2.5 Playback on device

- [ ] `playWavFile(path)` — stream from SD; REC tap stops playback
- [ ] `showPlaybackOverlay` during play

### **Milestone M2:** Full offline loop — record → tag → browse → play → delete — no WiFi

---

## Phase 3 — Connectivity & AI

### 3.1 WiFi

- [ ] STA mode; `WiFi.begin` with retry UI (`showWifiConnecting`, max 20 tries × 500 ms)
- [ ] Disconnect after sync (reference does not stay connected idle)
- [ ] Transfer mode: up to 24 tries; show IP on screen

### 3.2 NTP + time

- [ ] `syncTimeFromNTP` — `pool.ntp.org`, `time.google.com`, `time.cloudflare.com`
- [ ] Write RTC chip after successful sync
- [ ] `timeReady` flag gates created timestamps

### 3.3 Whisper API

- [ ] `POST https://api.openai.com/v1/audio/transcriptions`
- [ ] Multipart form: `model=whisper-1`, file=`note.wav`
- [ ] Stream WAV from SD in 4 KB chunks; 90 s timeout
- [ ] Parse JSON `"text"` field → write `note_NNN.txt`
- [ ] `updateIndexHasText(num)`; 3 retries with 3 s delay
- [ ] `transcribeAll()` — progress UI `showTranscribing(done, pending)`
- [ ] **Security:** move API key to secrets; plan cert pinning (reference TODO)

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

- [ ] Port `portalCss()` styling (or equivalent minimal CSS)
- [ ] `htmlEscape`, `urlDecodeSimple`, `readSmallFile` helpers
- [ ] Export truncation at ~55 KB (device memory limit)
- [ ] `stopTransferMode()` on exit: stop server, WiFi off

### **Milestone M3:** Sync transcribes one note; transfer mode serves portal on phone browser

---

## Phase 4 — Polish, enclosure & release

- [ ] Ultra-sleep after tag save (reference behavior)
- [ ] Battery warning overlay 2.5 s, non-blocking refresh of prior screen
- [ ] Device info: firmware version, battery %, RTC string, note count
- [ ] Error screens: `SD ERR`, `NO WIFI`, `REC FAIL`
- [ ] Validate 602530 500 mAh battery + foam tape in PETG enclosure
- [ ] Flashing guide in README (USB-C, `espflash` / `idf.py`)
- [ ] **Stretch:** OTA updates
- [ ] **Stretch:** SHTC3 on device info screen

### **Milestone M4:** v1.0 parity sign-off against feature checklist below

---

## Feature checklist (sign-off)

| # | Feature | Ref file | Status |
| --- | --- | --- | --- |
| 1 | Voice recording to microSD | `record.cpp` | [ ] |
| 2 | Tag system | `notes.cpp` | [ ] |
| 3 | Deep sleep | `sleep.cpp` | [ ] |
| 4 | Minimal E-Ink UI | `ui.cpp`, `draw.cpp` | [ ] |
| 5 | WiFi sync | `pala_note.ino` `startSyncFlow` | [ ] |
| 6 | Whisper transcription | `network.cpp` | [ ] |
| 7 | Local web interface | `network.cpp` portal | [ ] |
| 8 | On-device playback | `record.cpp` `playWavFile` | [ ] |
| 9 | Transfer mode | `network.cpp`, settings | [ ] |
| 10 | Customizable tags | `notes.cpp`, portal `/tags` | [ ] |
| 11 | Button sounds | `sounds.h` | [ ] |
| 12 | Battery indicator | `battery.cpp`, `showIdle` | [ ] |
| 13 | FAT32 microSD | `SD_MMC` setup | [ ] |
| 14 | Waveshare board | `config.h`, `board_cfg.h` | [ ] |
| 15 | Snap-fit enclosure | hardware / STL | [ ] |

---

## Suggested crate layout

```
firmware/
  Cargo.toml
  rust-toolchain.toml
  sdkconfig.defaults
  secrets.example.toml
  src/
    main.rs              # setup, loop, state dispatch
    board/
      mod.rs
      config.rs          # pins, timing
      power.rs
      pins.rs
    display/
      mod.rs
      epaper.rs
      draw.rs            # primitives, fonts
      ui.rs              # screens
    audio/
      mod.rs
      es8311.rs
      wav.rs             # header read/write
    storage/
      mod.rs
      sd.rs
      notes.rs           # index, tags, meta
    input/
      mod.rs
      buttons.rs
    power/
      mod.rs
      battery.rs
      sleep.rs
    time/
      mod.rs
      rtc.rs
      ntp.rs
    network/
      mod.rs
      wifi.rs
      whisper.rs
      portal.rs
    app/
      mod.rs
      state.rs           # AppState enum + transitions
      sounds.rs
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
| `src/app/rtc.cpp` | `time/rtc.rs`, `ntp.rs` | |
| `src/audio/*`, `codec_board/*` | `audio/es8311.rs` | FFI ok for v1 |
| `src/display/epaper_driver_bsp.*` | `display/epaper.rs` | |
| `src/power/board_power_bsp.*` | `board/power.rs` | |
| `secrets.h` | `secrets.toml` (gitignored) | |
| `sounds.h` | `app/sounds.rs` | |
| `types.h`, `globals.h` | `app/state.rs` | Minimize globals |

---

## Testing plan

| Layer | How |
| --- | --- |
| Unit | `storage/notes.rs` parse/write CSV, tag rules — host `cargo test` |
| Unit | WAV header builder — host test |
| Unit | `batteryPercentFromVoltage` curve — host test |
| HIL | UART log milestones M0–M4 on real board |
| HIL | SD pull-out recovery (graceful error screen) |
| HIL | 2 min idle → sleep → wake gesture |
| HIL | Portal from phone on same LAN |
| Manual | 10 consecutive recordings without SD corruption |
| Manual | Sync 5 notes Whisper end-to-end |

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
