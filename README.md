# Zoop

Small **Palma Notes**–style voice notepad on the Waveshare ESP32-S3 e-Paper board.

- **Reference firmware (C/Arduino):** [`pala_note/`](./pala_note/) — v1.0, 2026-05-24  
- **Rust IoT port (in progress):** see [`TODO.md`](./TODO.md)

---

## Features (v1.0 target)

- Voice recording directly onto the microSD card  
- Simple tag system for organizing recordings  
- Deep sleep for better battery life  
- Minimal E-Ink UI optimized for low power  
- Wi‑Fi syncing  
- AI transcription via OpenAI Whisper API  
- Local web interface for recordings and notes  
- Audio playback on the device  
- Transfer mode for downloading recordings in the browser  
- Customizable tags  
- Sound feedback for button presses  
- Battery level on the home screen  
- FAT32 microSD support  
- Designed for the Waveshare ESP32 E-Ink board  
- Optimized for a compact 3D-printed snap-fit enclosure  
- Open source, community-friendly firmware  

---

## Hardware

### Board

| Item | Spec |
| --- | --- |
| Board | Waveshare **ESP32-S3-ePaper-1.54** (`S3_ePaper_1_54`) |
| MCU | ESP32-S3 (dual-core up to 240 MHz), Wi‑Fi + BLE |
| Display | 1.54″ e-Paper, **200 × 200** |
| Audio | ES8311 codec, onboard mic + speaker header |
| Storage | microSD (SDIO 1-bit) |
| Sensors | PCF85063 RTC, SHTC3 temp/humidity |
| Power | Li‑ion charge IC, MX1.25 / GH1.25 battery connector |

Docs: [Waveshare ESP32-S3-ePaper-1.54](https://docs.waveshare.com/ESP32-S3-ePaper-1.54)

### Battery (recommended baseline)

- **3.7 V LiPo 602530, 500 mAh** (≈ 32 × 25 × 6 mm)  
- Link: https://geni.us/Bs2mF (AliExpress / Amazon)

**Notes**

- Some packs ship with a different JST connector; swap to match the board (see Assembly).  
- Use a small piece of foam or double-sided tape so the cell does not rattle in the case.

### SD card

- Any **FAT32**-formatted microSD for recordings and notes.  
- Prefer a known brand with decent random-write performance (see upgrades below).

### Filament

- **PETG** (recommended): https://geni.us/petgorigami  
- PETG is fine for everyday snap-fit use.

---

## What else to buy for better performance

These upgrades stay compatible with the same board and enclosure; pick what matches your budget.

| Upgrade | Why it helps | Suggestion |
| --- | --- | --- |
| Larger LiPo | Longer record / sync sessions between charges | 602530 **1000 mAh** or 603040 **800–1200 mAh** *if it still fits the case* |
| High-endurance / A2 microSD | Fewer stalls while writing WAV; better for always-on SD use | 32–64 GB **A2 / High Endurance** (SanDisk, Samsung, Kingston) |
| Correct MX1.25 / GH1.25 pigtail | Avoid flaky power from adapter hacks | Pre-crimped cable matching the Waveshare battery header |
| Foam / 3M VHB tape | Keeps battery and SD seating solid under pocket use | Thin foam + double-sided tape strip |
| External speaker (MX1.25) | Louder, clearer playback than the tiny onboard driver | 8 Ω 0.5–1 W mini speaker that fits the shell |
| Better PETG (or ASA) | Stronger snap-fit, less warp than cheap PLA | Dry PETG; ASA if you leave it in a hot car |
| USB‑C data cable (known good) | Reliable flashing and serial logs | Short, data-capable cable (not charge-only) |
| Silica packet / dry box for filament | Cleaner prints → better lid fit around the E-Ink | Optional but worth it for snap-fit tolerances |

**Diminishing returns / usually skip**

- Huge SD cards (128 GB+): waste for voice notes; format overhead and power draw rise.  
- Oversized batteries that force case mods: defeats the snap-fit design.  
- External Wi‑Fi antennas: board uses the onboard antenna; enclosure RF cutouts matter more than buying extras.

---

## Complete kit comparison

Pick one row as a shopping list. Prices are relative (**$** low → **$$$** higher); buy links change often — use the board vendor + the geni.us links above as starting points.

### Kit overview

| Kit | Best for | Relative cost | Est. battery life* | Record reliability | Playback | Print durability |
| --- | --- | --- | --- | --- | --- | --- |
| **Starter** | First build / validate firmware | $ | Short day of light use | OK | Onboard / basic | Good |
| **Recommended** | Daily carry (default buy) | $$ | Full day light use | Strong | Clear enough | Strong |
| **Performance** | Heavy recording + sync | $$$ | Multi-day light / long sessions | Best | Loud & clear | Best |
| **Dev / lab** | Flashing & debugging on the desk | $$ | N/A (often USB-powered) | Strong | Optional | Any |

\*Battery life is rough and depends on Wi‑Fi sync frequency, E-Ink refreshes, and sleep behavior.

### Bill of materials by kit

| Part | Starter | Recommended | Performance | Dev / lab |
| --- | --- | --- | --- | --- |
| Waveshare ESP32-S3-ePaper-1.54 | ✓ | ✓ | ✓ | ✓ |
| LiPo | 500 mAh 602530 | 500–800 mAh, correct connector | Largest that fits (≈800–1200 mAh) | Optional / USB only |
| microSD | 16 GB FAT32, any brand | 32 GB A1/A2 name brand | 32–64 GB **A2 / High Endurance** | 32 GB A2 |
| Filament | PETG | Dry PETG | Dry PETG or ASA | PLA OK for jigs |
| Battery tape / foam | Optional | ✓ | ✓ | — |
| Connector fix (if needed) | DIY swap | Pre-made MX1.25 lead | Pre-made MX1.25 lead | — |
| External speaker | — | Optional | ✓ fitted in case | Bench speaker |
| USB‑C data cable | Basic | Known-good | Known-good + spare | ✓ + UART if needed |
| Enclosure STL print | ✓ | ✓ | ✓ (tight tolerances) | Open jig / no case |

### When to choose which

| If you want… | Choose |
| --- | --- |
| Cheapest path to a working note-taker | **Starter** |
| Balanced pocket device matching pala_note design intent | **Recommended** |
| Longer sessions, fewer SD write glitches, louder playback | **Performance** |
| Firmware work without caring about battery fit yet | **Dev / lab** |

---

## Assembly tips

1. Confirm battery connector polarity and pitch (**MX1.25 / GH1.25** on the Waveshare board) before plugging in.  
2. Format the microSD as **FAT32** on a computer before first boot.  
3. Secure the cell with foam/double-sided tape so it cannot press the E-Ink or SD slot.  
4. Print the snap-fit shell in **PETG**; dry the filament if lids feel soft or warped.  
5. Flash over USB‑C; keep the board powered (USB or battery hold) during first bring-up.

---

## Repo layout

```
zoop/
  README.md          ← you are here
  TODO.md            ← Rust IoT port checklist
  pala_note/         ← reference C/Arduino firmware
```

---

## License / credits

Open source firmware direction inspired by the pala_note community build. Hardware by Waveshare. Rust port tracked in `TODO.md`.
