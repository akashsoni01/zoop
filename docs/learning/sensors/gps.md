# GPS Modules (NEO-6M, u-blox)

**Global Positioning System (GPS)** receivers output position, velocity, and time via **NMEA** text sentences over **UART**.

**Prerequisites:** [08-gpio.md](../08-gpio.md), UART lesson (planned), [09-interrupts.md](../09-interrupts.md)

---

## Working Principle

GPS receiver chip tracks **satellite signals** (L1, 1575.42 MHz). Trilateration from ≥4 satellites yields:

- **Latitude / Longitude** (WGS84)
- **Altitude** (MSL, coarse)
- **Speed / Course**
- **UTC time**

Cold start: 30+ s; warm/hot start faster with almanac/ephemeris cached.

---

## Datasheet Key Points (NEO-6M / u-blox 6)

| Parameter | Value |
|-----------|-------|
| Supply | 3.3 V (some modules include USB/UART level shifter) |
| Baud | Default 9600 8N1 |
| Protocol | NMEA 0183 (text), optional UBX (binary) |
| Fix types | No fix, 2D, 3D |
| Current | ~25 mA tracking |

**NEO-6M** breakout often includes EEPROM for config and **PPS** (pulse per second) pin.

---

## Protocol

### UART Wiring (ESP32-S3)

```
ESP32-S3          GPS Module (NEO-6M)
────────          ───────────────────
GPIO17 (TX) ────► RX   (ESP TX → GPS RX)
GPIO18 (RX) ◄──── TX   (GPS TX → ESP RX)
3V3         ─────► VCC
GND         ─────► GND
```

Cross-connect TX/RX. GPS TX is **3.3 V** on most modules — safe for ESP32-S3.

Optional: GPIO19 ← PPS (1 Hz square wave, rising edge aligned to UTC second).

### NMEA Sentences (Common)

| Sentence | Content |
|----------|---------|
| `$GPGGA` | Fix quality, lat/lon, satellites, altitude |
| `$GPRMC` | Time, lat/lon, speed, course, date |
| `$GPGSV` | Satellites in view (signal strength) |
| `$GPGSA` | DOP, active satellites |

Example `$GPRMC`:

```text
$GPRMC,123519,A,4807.038,N,01131.000,E,022.4,084.4,230394,003.1,W*6A
```

Fields: time, status (A=valid), lat, N/S, lon, E/W, speed knots, course, date.

### UBX (Advanced)

u-blox **binary protocol** for configuration (baud rate, update rate, constellations). Requires checksum (CK_A, CK_B). Use when NMEA parsing is insufficient.

---

## Register Map

GPS modules are not register-based like I²C sensors. Configuration is via:

- **NMEA commands** (limited)
- **UBX-CFG-*** messages
- **u-center** PC tool (save to flash)

---

## Rust Driver Sketch

```rust
use heapless::String;

pub struct NmeaParser {
    buf: String<128>,
}

impl NmeaParser {
    pub fn push_byte(&mut self, b: u8) -> Option<GprmcData> {
        match b {
            b'\n' => {
                let line = self.buf.clone();
                self.buf.clear();
                return parse_gprmc(&line);
            }
            b'\r' => {}
            c if self.buf.len() < 128 => { let _ = self.buf.push(c as char); }
            _ => {}
        }
        None
    }
}

pub struct GprmcData {
    pub valid: bool,
    pub lat: f64,
    pub lon: f64,
    pub speed_knots: f32,
}
```

**Crates:** `nmea-parser`, `ublox` (full UBX stack).

### UART Setup (esp-hal sketch)

```rust
// Configure UART2 @ 9600 on GPIO17/18
// Read bytes in loop or DMA ring buffer
// Feed each byte to NmeaParser
```

---

## Bare-Metal Notes

- Use **interrupt or DMA** RX — NMEA arrives asynchronously at 9600 baud (~1 ms/byte).
- Validate **NMEA checksum** (XOR between `$` and `*`).
- **Float parsing** in `no_std`: use `micromath` or integer micro-degrees.
- Antenna must see sky — indoor fixes are unreliable.

---

## HAL / embedded-hal Notes

`embedded-hal` 1.0 defines `Serial` / `Read` traits. GPS driver consumes `impl Read`:

```rust
pub trait GpsReceiver {
    type Error;
    fn poll(&mut self) -> Result<Option<Fix>, Self::Error>;
}
```

Compose with async UART for Embassy.

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| No output | Wrong baud | Try 9600, 38400; check module LED blink |
| Garbage chars | TX/RX swapped | Cross-connect |
| No fix indoors | Signal | Move near window; wait 5 min cold start |
| Jumpy position | Poor HDOP | Wait for ≥6 satellites; check `$GPGSA` |
| 5 V module on ESP32 | Level mismatch | Use 3.3 V module or level shifter |

---

## Example Project: GPS Logger

1. Parse `$GPRMC` at 1 Hz.
2. When fix valid, append lat/lon/time to CSV on SPI flash ([flash-memory.md](./flash-memory.md)).
3. Show satellite count on character LCD ([lcd-character.md](../displays/lcd-character.md)).

---

## Exercises

1. Parse `$GPGGA` and extract HDOP and satellite count.
2. Convert NMEA `ddmm.mmmm` lat/lon to decimal degrees.
3. Send UBX command to increase update rate to 5 Hz (document hex payload).

---

## References

- u-blox 6 Receiver Description Protocol Spec (NMEA, UBX)
- NMEA 0183 standard overview
- `nmea-parser` crate docs
- [flash-memory.md](./flash-memory.md)
