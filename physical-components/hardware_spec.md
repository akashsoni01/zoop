# Hardware spec — OceanLabz DIY AI Voice Kit

**This file is the single source of truth** for the physical kit Zoop’s QR / voice+vision track targets.  
When wiring, choosing GPIOs, or writing board code, **prefer this file** over scattered KS5028 / generic notes.

| Field | Value |
| --- | --- |
| Product title | DIY AI Voice Kit with ESP32-S3 Camera Board \| OLED, INMP441 Microphone, MAX98357 DAC, Speaker & More! |
| Brand | **OceanLabz** |
| Model name | Starter Kit |
| Model number | ESP32 Basic ST Kit |
| MPN | DIY AI Voice Kit |
| ASIN | [B0G26QNQLD](https://www.amazon.in/dp/B0G26QNQLD) (marketplace listing) |
| Vendor page | [oceanlabz.in — DIY AI Voice Kit](https://www.oceanlabz.in/product/diy-ai-voice-kit-with-esp32-s3-camera-board-oled-inmp441-microphone-max98357-dac-speaker-more-smart-voice-vision-development-kit-for-ai-projects/) |
| Packer / manufacturer | OceanLabz · Country of origin: India |
| Unit count | 13 (kit pieces) |
| Item weight | ~500 g (listing) |
| Package size (listing) | ~10 × 10 × 5 cm |

Beginner assembly: [`README.md`](./README.md) · Software setup: [`../scripts/README.md`](../scripts/README.md) · Pin constants in code: [`../firmware-qr/src/board/pins.rs`](../firmware-qr/src/board/pins.rs)

---

## 1. What Zoop assumes you have

Smart **voice + vision** development kit for AI / IoT projects:

| # | Component | Role in Zoop |
| --- | --- | --- |
| 1 | **ESP32-S3 Camera Board** | MCU, Wi‑Fi / BLE, camera interface, USB program/serial |
| 2 | **Camera module** (OV-series on FPC) | Live frames → QR decode |
| 3 | **OLED** (typically 0.96″ I²C 128×64) | Aiming / decoded string UI |
| 4 | **INMP441** | I²S MEMS microphone |
| 5 | **MAX98357** (MAX98357A) | I²S Class‑D amp (“DAC” in marketing) |
| 6 | **Speaker** | Beeps / prompts |
| 7+ | Breadboard, jumpers, USB cable, extras | Prototyping (kit “13 count”) |

Listing **box contents** (marketing): *DAC, ESP32-S3 Camera Board, Microphone, OLED* — expect jumpers / breadboard / speaker in the full kit count.

---

## 2. Specs from the product listing

Recorded from the marketplace page. **Some marketplace fields are wrong for ESP32-S3** — see corrections below.

| Listing field | Listed value | Zoop interpretation / correction |
| --- | --- | --- |
| RAM memory installed | **8 MB** | Treat as **8 MB PSRAM** (external), not internal SRAM alone |
| RAM technology (listing) | SRAM | Misleading; S3 has internal SRAM **plus** external PSRAM |
| Memory storage capacity | **16 MB** | Treat as **16 MB flash** (typical N16R8-class module) |
| Processor brand | Espressif | Correct |
| CPU / SoC | ESP32-S3 (dual-core — see About) | Use Espressif ESP32-S3 |
| Processor speed (listing) | **2.5 MHz** | **Ignore** — marketplace error. ESP32-S3 runs up to **240 MHz** per core |
| Processor count (listing) | 1 | **Ignore** — ESP32-S3 is **dual-core** Xtensa LX7 |
| Network | Bluetooth, GPIO, I2C, USB, Wi‑Fi | Wi‑Fi **2.4 GHz** + Bluetooth LE; GPIO / I²C / USB present |
| Wireless | 2.4 GHz RF, Bluetooth | Correct class |
| USB ports | 1 | Type‑C data port for flash + UART |
| OS (listing) | Android, FreeRTOS, Linux, Windows | Host PC OS for tooling; on-device stack is **FreeRTOS** via ESP‑IDF (Zoop) |
| Compatible devices | IDE, MicroPython, ESP, RPI | Zoop uses **Rust + esp-idf** on this kit |
| Smart home | Listed compatible | Optional; Zoop v1 is local scan → string / pay tracks |

### About-this-item (vendor bullets)

- ESP32-S3 AI kit with **camera, mic, DAC, speaker**
- Dual-core ESP32-S3 with **Wi‑Fi + Bluetooth**
- **INMP441** for voice projects
- **MAX98357** DAC & speaker for sound
- Aimed at AI, IoT learning, smart DIY

---

## 3. Electrical / MCU facts Zoop cares about

| Item | Value |
| --- | --- |
| MCU | Espressif **ESP32-S3** (Xtensa LX7 dual-core, up to 240 MHz) |
| Flash (typical) | **16 MB** |
| PSRAM (typical) | **8 MB** — needed for camera frames + QR decode |
| Radio | Wi‑Fi 2.4 GHz + Bluetooth LE |
| USB | 1× Type‑C (program + serial @ 115200) |
| Camera | Onboard FPC → OV2640-class (confirm silk / module) |
| Mic bus | I²S → INMP441 |
| Amp bus | I²S → MAX98357A → speaker |
| Display bus | I²C → OLED (SSD1306-class unless kit card says otherwise) |
| Logic level | **3.3 V** GPIO; amp Vin often **5 V** on OceanLabz sibling guides |

---

## 4. Pin map (draft — verify with your kit card)

OceanLabz publishes slightly different tables for **camera+TFT** vs **OLED-only** boards. This kit is **camera + OLED**. Until the included wiring card is transcribed here after HIL, use the draft below and **always prefer the paper/PDF in your box**.

### 4.1 Zoop draft (Camera board + OLED kit)

Aligned with OceanLabz **ESP32-S3 Camera** audio wiring ([TFT-cam sibling guide](https://www.oceanlabz.in/esp32-s3-tft-cam/)), with OLED on free I²C pins that avoid mic SD=GPIO41.

| Function | GPIO / rail | Module pin | Notes |
| --- | --- | --- | --- |
| INMP441 WS | **39** | WS | OceanLabz camera-board ref |
| INMP441 SCK | **40** | SCK | |
| INMP441 SD | **41** | SD | |
| INMP441 VDD | **3V3** | VDD | |
| INMP441 GND | **GND** | GND | |
| INMP441 L/R | **GND** | L/R | Left channel |
| MAX98357 LRC | **21** | LRC | OceanLabz camera-board ref |
| MAX98357 DIN | **47** | DIN | |
| MAX98357 BCLK | **48** | BCLK | |
| MAX98357 Vin | **5V** (or 3V3 if module requires) | Vin | Sibling guide uses 5V |
| MAX98357 GND | **GND** | GND | |
| MAX98357 GAIN | **GND** | GAIN | Lower gain (common) |
| MAX98357 → speaker | Audio+ / Audio− | Speaker | Not on GPIO |
| OLED SDA | **8** | SDA | Draft — avoid clash with mic SD=41 |
| OLED SCL | **9** | SCL | Draft |
| OLED VCC | **3V3** | VCC | |
| OLED GND | **GND** | GND | |
| Camera | FPC | OV module | Board defaults / `esp_camera` |
| REC / BOOT | **0** | Button → GND | Flash + confirm |
| PWR / cancel | **14** (draft) | Button → GND | Pick free pin after HIL; avoid audio GPIOs |

> If your kit card disagrees, **update this table** and `firmware-qr/src/board/pins.rs` together.

### 4.2 OceanLabz sibling references (do not mix blindly)

| Guide | Mic | Amp | Display |
| --- | --- | --- | --- |
| [Camera + TFT](https://www.oceanlabz.in/esp32-s3-tft-cam/) | SD41 WS39 SCK40 | LRC21 DIN47 BCLK48 Vin5V | SPI TFT (not our OLED) |
| [OLED board (no cam)](https://www.oceanlabz.in/esp32-s3-with-0-96-inc-oled-display/) | SD8 WS46 SCK9 | LRC10 DIN3 BCLK11 Vin5V | OLED SDA41 SCL42 |

Visual community photos (technique only): [s60sc/ESP32-CAM_MJPEG2SD](https://github.com/s60sc/ESP32-CAM_MJPEG2SD) — see [`README.md` § Public repos](./README.md#public-repos--see-the-connections).

---

## 5. Software mapping (Zoop)

| Layer | Path |
| --- | --- |
| Host QR decode + debounce | `zoop-core` → `qr::` |
| OLED framebuffer UI | `zoop-core` → `display::oled` |
| Scan state machine | `zoop-core` → `state_qr` |
| Board pin constants | `firmware-qr/src/board/pins.rs` (**must match §4.1**) |
| Host demo (no flash) | `cargo run -p zoop-firmware-qr` |
| Product track | [`TODO_qr.md`](../TODO_qr.md) |

---

## 6. Change log

| Date | Change |
| --- | --- |
| 2026-07-20 | Initial OceanLabz DIY AI Voice Kit lock-in from listing ASIN B0G26QNQLD + OceanLabz sibling pin refs |
