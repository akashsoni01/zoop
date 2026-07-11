# Zoop — Palma Notes in Rust (IoT)

Port of the C/Arduino [`pala_note`](./pala_note/) firmware to **Rust** on the Waveshare ESP32-S3 e-Paper 1.54 board, using `esp-idf-hal` / `esp-idf-svc` (or Embassy) for low-power voice notes.

Reference firmware: Arduino/C++ in `pala_note/` (v1.0, 2026-05-24). Target board: **Waveshare ESP32-S3-ePaper-1.54** (`S3_ePaper_1_54` in `board_cfg.h`).

---

## Goals

- [ ] Voice notes on device with E-Ink UI, SD storage, deep sleep, Wi‑Fi sync, and Whisper transcription
- [ ] Safe, modular Rust firmware (no heap surprises in audio path; typed state machine)
- [ ] Feature parity with pala_note v1.0, then small quality upgrades

---

## Phase 0 — Project scaffold

- [ ] Create `firmware/` (or `zoop-fw/`) as an `esp-idf` + `esp-rs` / `esp-idf-sys` Cargo project
- [ ] Pin Rust nightly/toolchain for ESP32-S3; document `espup` / `idf.py` setup in README
- [ ] Mirror pin map from [`pala_note/config.h`](./pala_note/config.h) into a Rust `board` module
- [ ] Secrets via `secrets.toml` / env (Wi‑Fi, OpenAI) — never commit keys
- [ ] CI: `cargo check` for `xtensa-esp32s3-espidf` (or equivalent)

---

## Phase 1 — Board bring-up (BSP)

- [ ] GPIO power rails: EPD (`GPIO6`), audio (`GPIO42`), battery hold (`GPIO17`)
- [ ] Battery ADC (`GPIO4`) + percentage mapping (port `battery.cpp`)
- [ ] Buttons: REC (`GPIO0`), PWR (`GPIO18`) — single / long / double (port `buttons.cpp`)
- [ ] I2C bus (`SDA=47`, `SCL=48`): PCF85063 RTC (`0x51`), SHTC3 (`0x70`), ES8311 (`0x18`)
- [ ] E-Ink 200×200 SPI driver (port `epaper_driver_bsp`) — full + partial refresh
- [ ] SD_MMC 1-bit (`CLK=39`, `CMD=41`, `D0=40`) + FAT32 mount
- [ ] ES8311 record/playback via I2S (port `audio_bsp` / codec_board)
- [ ] Deep sleep + wake on `GPIO0` (port `sleep.cpp`)

---

## Phase 2 — Core app (offline)

- [ ] App state machine (`types.h` states → Rust enum)
- [ ] Record WAV @ 16 kHz to `/notes` on SD (`record.cpp`)
- [ ] Note index CSV + tag file (`notes.cpp`, `INDEX_FILE`, `TAG_FILE`)
- [ ] Default tags + customizable tags (max 20)
- [ ] E-Ink screens: idle, recording, tag select, note list/detail, menu, settings, battery
- [ ] Sound feedback for buttons (port `sounds.h`)
- [ ] Battery level on home; low-battery warning thresholds
- [ ] Audio playback of selected note on device

---

## Phase 3 — Connectivity & AI

- [ ] Wi‑Fi STA connect + reconnect policy
- [ ] OpenAI Whisper API upload/transcribe (`network.cpp` → `transcribe`)
- [ ] Batch “sync / transcribe all”
- [ ] Local HTTP portal: list notes, tags CRUD, export, delete
- [ ] Transfer mode: serve recordings for browser download
- [ ] NTP / RTC time sync (honor `LOCAL_TIME_OFFSET_MIN`)

---

## Phase 4 — Polish & enclosure

- [ ] Ultra-sleep after idle (`ULTRA_SLEEP_MS`)
- [ ] Device info / firmware version screen
- [ ] Power-button latch behavior matching pala_note
- [ ] Validate with snap-fit 3D enclosure (PETG) + recommended battery fit
- [ ] Flash instructions + OTA stretch goal

---

## Feature checklist (parity with pala_note v1.0)

| Feature | Status |
| --- | --- |
| Voice recording to microSD | [ ] |
| Tag system | [ ] |
| Deep sleep | [ ] |
| Minimal E-Ink UI | [ ] |
| Wi‑Fi sync | [ ] |
| Whisper transcription | [ ] |
| Local web interface | [ ] |
| On-device playback | [ ] |
| Transfer mode | [ ] |
| Customizable tags | [ ] |
| Button sound feedback | [ ] |
| Battery indicator | [ ] |
| FAT32 microSD | [ ] |
| Waveshare ESP32 E-Ink board | [ ] |
| Compact snap-fit enclosure | [ ] (hardware / STL) |

---

## Suggested Rust crate layout

```
firmware/
  Cargo.toml
  src/
    main.rs
    board/          # pins, power, adc
    display/        # epaper
    audio/          # es8311 + i2s
    storage/        # sd + notes index
    ui/             # screens + input
    network/        # wifi, whisper, portal
    power/          # deep sleep
  secrets.example.toml
```

---

## Reference map (C → Rust)

| pala_note | Responsibility |
| --- | --- |
| `config.h` | Pins, timing, paths |
| `pala_note.ino` | Setup / loop orchestration |
| `src/app/*.cpp` | UI, record, notes, network, sleep, battery, buttons |
| `src/audio/*`, `src/codec_board/*` | Codec + I2S |
| `src/display/*` | E-Paper |
| `src/power/*` | Rail enables |
| `secrets.h` | Wi‑Fi / API keys |

---

## Non-goals (for now)

- Multi-color E-Ink (G variant) UI themes
- On-device Whisper (too heavy for ESP32-S3)
- BLE companion app
