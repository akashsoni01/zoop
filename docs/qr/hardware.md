# Zoop QR — hardware

**Canonical kit + pin map:** [`physical-components/hardware_spec.md`](../../physical-components/hardware_spec.md)

Do not invent GPIOs here. Update `hardware_spec.md` first, then sync `firmware-qr/src/board/pins.rs`.

| Doc | Role |
| --- | --- |
| [`hardware_spec.md`](../../physical-components/hardware_spec.md) | OceanLabz DIY AI Voice Kit — SKU, specs, pin table |
| [`physical-components/README.md`](../../physical-components/README.md) | Beginner breadboard / wiring guide |
| [`scripts/README.md`](../../scripts/README.md) | Fresh laptop toolchain |
| [`TODO_qr.md`](../../TODO_qr.md) | Product phases |

## Kit (locked)

**OceanLabz DIY AI Voice Kit** — ESP32-S3 Camera Board + **1.54″ OLED** + INMP441 + MAX98357 + speaker  
ASIN [B0G26QNQLD](https://www.amazon.in/dp/B0G26QNQLD) · ~8 MB PSRAM · ~16 MB flash

## Crate layout

Dedicated **`firmware-qr/`** (`zoop-firmware-qr`) — separate from e-Paper collect (`firmware/`). Share `zoop-core` decode/debounce.

## Pin table (mirror — edit hardware_spec.md)

See §4.1 in [`hardware_spec.md`](../../physical-components/hardware_spec.md). Summary:

| Function | GPIO |
| --- | --- |
| Mic WS / SCK / SD | 39 / 40 / 41 |
| Amp LRC / DIN / BCLK | 21 / 47 / 48 |
| OLED SDA / SCL | 8 / 9 (draft) |
| REC / PWR | 0 / 14 (draft) |
