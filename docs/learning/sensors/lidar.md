# LiDAR Sensors (TF-Luna, RPLidar)

**Light Detection and Ranging (LiDAR)** measures distance using laser time-of-flight or triangulation. This guide covers UART **TF-Luna** and scanning **RPLidar** modules.

**Prerequisites:** [08-gpio.md](../08-gpio.md), [10-timers.md](../10-timers.md)

---

## Working Principle

### TF-Luna (Single-Point ToF)

940 nm VCSEL laser + SPAD receiver. Measures **one distance** along boresight via time-of-flight. Outputs binary frame over UART.

### RPLidar (2D Scanner)

Rotating laser unit sweeps 360°; each angle returns distance. High throughput binary protocol over UART (often 256000 baud).

---

## Datasheet Key Points (TF-Luna)

| Parameter | Value |
|-----------|-------|
| Range | 0.2–8 m (white target) |
| Accuracy | ±6 cm (typ.) |
| Interface | UART (115200 default) or I²C |
| Frame rate | 100–1000 Hz |
| Supply | 5 V (100 mA peak) |

---

## Protocol

### TF-Luna UART Frame (9 bytes)

```text
0x59 0x59 Dist_L Dist_H Strength_L Strength_H Temp_L Temp_H Checksum
```

Checksum = sum of bytes 0–7, low 8 bits.

Distance cm = `Dist_L | (Dist_H << 8)`.

### RPLidar (Overview)

Start scan command → continuous stream of `{angle, distance, quality}` nodes. Requires parser state machine and large RX buffer (`heapless::Vec` or ring buffer).

---

## Register Map

TF-Luna I²C mode uses register addresses for config (see Benewake datasheet). UART mode uses **command frames** instead:

| Command | Purpose |
|---------|---------|
| `0x5A` + payload | Product info |
| `0x5A 0x06 ...` | Set output format |

RPLidar: command protocol in Slamtec documentation (no simple register map).

---

## ESP32-S3 Wiring (TF-Luna)

```
TF-Luna           ESP32-S3
───────           ────────
VCC  ◄── 5V (or 3V3 per module variant)
GND  ───────────► GND
TX   ───────────► GPIO18 (ESP RX)
RX   ◄─────────── GPIO17 (ESP TX)
```

Cross-connect serial lines.

---

## Rust Driver Sketch (TF-Luna)

```rust
pub struct TfLuna<R: Read> {
    uart: R,
    buf: [u8; 9],
    idx: usize,
}

impl<R: Read> TfLuna<R> {
    pub fn poll(&mut self) -> Result<Option<u16>, R::Error> {
        let mut b = [0u8; 1];
        self.uart.read(&mut b)?;
        if self.idx == 0 && b[0] != 0x59 {
            return Ok(None);
        }
        self.buf[self.idx] = b[0];
        self.idx += 1;
        if self.idx == 9 {
            self.idx = 0;
            if self.buf[1] == 0x59 && checksum(&self.buf) {
                let dist = (self.buf[2] as u16) | ((self.buf[3] as u16) << 8);
                return Ok(Some(dist));
            }
        }
        Ok(None)
    }
}

fn checksum(f: &[u8; 9]) -> bool {
    f[8] == f[..8].iter().map(|&x| x as u16).sum::<u16>() as u8
}
```

**Crates:** `rplidar-rs` for RPLidar; TF-Luna often custom parser.

---

## Bare-Metal Notes

- Allocate **≥256 byte** UART RX buffer for RPLidar bursts.
- TF-Luna: sync on dual `0x59` header — resync on any mismatch.
- **5 V power** may be required for full range — check module label.
- Mount sensor **perpendicular** to target; angle error spreads distance error.

---

## HAL / embedded-hal Notes

```rust
use embedded_io::Read; // or embedded-hal serial equivalent
```

Async UART recommended for RPLidar continuous scan task separate from fusion task.

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| No frames | Wrong baud | TF-Luna 115200; RPLidar often 256000 |
| Zero distance | Out of range / glass | Glass is transparent to IR — use matte target |
| Checksum fail | EMI on UART | Shorter cable, common GND |
| RPLidar spin fails | Insufficient 5V | Separate 5V 2A supply |

---

## Example Project: Obstacle Stop

TF-Luna forward-facing on small robot:

1. Read distance at 50 Hz.
2. If < 25 cm, disable motor PWM.
3. Show distance on ST7789 bar ([st7789.md](../displays/st7789.md)).

---

## Exercises

1. Parse TF-Luna temperature field from frame bytes 6–7.
2. Implement RPLidar scan start/stop and plot 10 points over UART `defmt`.
3. Compare TF-Luna vs HC-SR04 ([ultrasonic-hc-sr04.md](./ultrasonic-hc-sr04.md)) indoors.

---

## References

- Benewake TF-Luna datasheet / communication protocol
- Slamtec RPLidar protocol manual
- [tof-vl53l0x.md](./tof-vl53l0x.md)
- [ultrasonic-hc-sr04.md](./ultrasonic-hc-sr04.md)
