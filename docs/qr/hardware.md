# Zoop QR — hardware (draft)

Draft BOM and pin map for the **scan → string** kit. Lock SKUs in Phase 0 HIL; values marked **TBD** are placeholders.

Related: [`TODO_qr.md`](../../TODO_qr.md) · [`TODO_camera.md`](../../TODO_camera.md)

## Crate decision

Dedicated **`firmware-qr/`** (`zoop-firmware-qr`) — separate from e-Paper collect (`firmware/`). Share `zoop-core` decode/debounce; do not mix e-Paper BSP.

## BOM (draft — Kit A style)

| Part | Role | Candidate | Notes |
| --- | --- | --- | --- |
| ESP32-S3 camera board | MCU + sensor | Generic S3-CAM OV2640 + PSRAM | Or XIAO ESP32S3 Sense |
| OLED | Aiming / result | SSD1306 128×64 I²C | SH1106 alt |
| Microphone | Trigger / levels | INMP441 (I²S MEMS) | Often onboard on Sense |
| DAC / amp | Beeps | MAX98357A → speaker | I²S TX |
| Buttons | Accept / cancel | ≥2 GPIO | REC / PWR |
| Battery | Portable | LiPo + ADC | TBD sense pin |

## Pin table (draft)

| Function | GPIO | Bus | Status |
| --- | --- | --- | --- |
| OLED SDA | 8 | I²C | Draft |
| OLED SCL | 9 | I²C | Draft |
| OLED RST | — | — | Often NC / TBD |
| Button REC | 0 | GPIO | Draft (boot-strap aware) |
| Button PWR | 1 | GPIO | Draft |
| I²S BCLK (mic) | 41 | I²S RX | Placeholder |
| I²S WS (mic) | 42 | I²S RX | Placeholder |
| I²S DIN (mic) | 40 | I²S RX | Placeholder |
| I²S BCLK (DAC) | TBD | I²S TX | Time-slice with mic |
| I²S WS (DAC) | TBD | I²S TX | |
| I²S DOUT (DAC) | TBD | I²S TX | |
| Camera DVP / SCCB | board defaults | — | **TBD** per camera SKU |
| Battery ADC | TBD | ADC | |

Camera pins depend on the chosen module (OV2640 vs Sense): document the final map when the board SKU is frozen.
