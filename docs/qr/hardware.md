# Zoop QR — hardware (draft)

Draft BOM and pin map for the **scan → string** kit. Lock SKUs in Phase 0 HIL; values marked **TBD** are placeholders.

Related: [`TODO_qr.md`](../../TODO_qr.md) · [`TODO_camera.md`](../../TODO_camera.md)

## Crate decision

Dedicated **`firmware-qr/`** (`zoop-firmware-qr`) — separate from e-Paper collect (`firmware/`). Share `zoop-core` decode/debounce; do not mix e-Paper BSP.

## BOM (draft — Kit A style)

| Part | Role | Candidate | Notes |
| --- | --- | --- | --- |
| ESP32-S3 camera board | MCU + sensor | DIY AI Voice+Vision S3-CAM (OV + PSRAM) | KS5028-class / similar |
| OLED | Aiming / result | SSD1306 128×64 I²C | SH1106 alt |
| Microphone | Trigger / levels | INMP441 (I²S MEMS) | Often onboard on Sense |
| DAC / amp | Beeps | MAX98357A → speaker | I²S TX |
| Buttons | Accept / cancel | ≥2 GPIO | REC / PWR |
| Battery | Portable | LiPo + ADC | TBD sense pin |

## Target kit

**DIY AI Voice & Vision** — ESP32-S3 Camera Board + OLED + INMP441 + MAX98357A + speaker  
(Keyestudio **KS5028**-class wiring used as the draft below; always check your board silkscreen.)

Fresh laptop setup: [`scripts/README.md`](../../scripts/README.md)

## Pin table (draft — KS5028-class)

| Function | GPIO | Bus | Status |
| --- | --- | --- | --- |
| OLED SDA | 8 | I²C | Draft (freeze HIL) |
| OLED SCL | 9 | I²C | Draft (freeze HIL) |
| OLED RST | — | — | Often NC |
| Button REC | 0 | GPIO | BOOT / IO0 — bootstrap aware |
| Button PWR | 21 | GPIO | Draft (GPIO 1 taken by mic WS on KS5028) |
| INMP441 WS | 1 | I²S RX | KS5028 example |
| INMP441 SCK | 2 | I²S RX | KS5028 example |
| INMP441 SD | 42 | I²S RX | KS5028 example |
| MAX98357 DIN | 39 | I²S TX | KS5028 example |
| MAX98357 BCLK | 40 | I²S TX | KS5028 example |
| MAX98357 LRC | 41 | I²S TX | KS5028 example |
| Camera | onboard FPC | DVP | Board defaults (`esp_camera`) |
| Battery ADC | TBD | ADC | |

> **Conflict note:** KS5028 maps mic WS to GPIO 1. If you also want a PWR button on GPIO 1, remount the button or move mic WS after HIL.

Constants: `firmware-qr/src/board/pins.rs`.
