# Touch Sensors

Capacitive **touch** sensing — from simple **TTP223** modules to multi-channel **MPR121** I²C controllers.

**Prerequisites:** [08-gpio.md](../08-gpio.md), [17-i2c.md](../17-i2c.md)

---

## Working Principle

**Capacitive sensing:** electrode capacitance increases when finger (conductive) approaches. IC detects delta vs baseline.

| Type | Channels | Interface |
|------|----------|-----------|
| TTP223 | 1 | Digital GPIO |
| TTP229 | 16 | I²C or serial |
| MPR121 | 12 | I²C |
| ESP32 touch pads | 14 | On-chip peripheral |

---

## Datasheet Key Points (TTP223)

| Parameter | Value |
|-----------|-------|
| Output | Digital HIGH when touched |
| Modes | Toggle / momentary (pad on module) |
| Supply | 2.0–5.5 V |

### MPR121 Highlights

| Parameter | Value |
|-----------|-------|
| I²C address | `0x5A` / `0x5B` |
| Electrodes | 12 capacitive |
| Registers | Touch status, baseline, sensitivity |

---

## Protocol

**TTP223:** Single GPIO digital read.

**MPR121:** I²C register map — read `0x00`–`0x01` touch status (24 bits for 12 keys).

### MPR121 Register Map (Essential)

| Addr | Name | Description |
|------|------|-------------|
| `0x00`–`0x01` | Touch status | Bit per electrode |
| `0x5B` | Electrode config | Charge/discharge timing |
| `0x5E`–`0x5F` | Baseline filters | |
| `0x41`–`0x5A` | Thresholds | Touch/release per channel |
| `0x80` | Reset | Soft reset `0x63` |

---

## ESP32-S3 Wiring

### TTP223

```
TTP223           ESP32-S3
──────           ────────
VCC  ──────────► 3V3
GND  ──────────► GND
SIG  ──────────► GPIO4
```

### MPR121

```
MPR121           ESP32-S3
──────           ────────
SDA  ──────────► GPIO8
SCL  ──────────► GPIO9
IRQ  ──────────► GPIO7
VCC/GND          3V3/GND
```

Connect electrode pads to MPR121 `E0`–`E11` with proper layout (ground plane, series resistor).

---

## Rust Driver Sketch (MPR121)

```rust
use embedded_hal::i2c::I2c;

const ADDR: u8 = 0x5A;

pub struct Mpr121<I2C> {
    i2c: I2C,
}

impl<I2C, E> Mpr121<I2C>
where
    I2C: I2c<Error = E>,
{
    pub fn init(&mut self) -> Result<(), E> {
        self.write(0x80, 0x63)?; // reset
        self.delay_ms(1);
        self.write(0x5E, 0x00)?; // run mode stop
        // configure thresholds, filters...
        self.write(0x5E, 0x08)?; // start with all electrodes
        Ok(())
    }

    pub fn touched(&mut self) -> Result<u16, E> {
        let mut b = [0u8; 2];
        self.i2c.write_read(ADDR, &[0x00], &mut b)?;
        Ok(((b[0] as u16) << 8) | b[1] as u16)
    }
}
```

**Crates:** `mpr121`, `esp-hal` touch API for native ESP32 pads.

---

## Bare-Metal Notes

- **Baseline auto-calibration** — MPR121 tracks slow environmental drift.
- **Water rejection** poor on cheap modules — use proper controller for outdoor.
- ESP32-S3 **touch pad** peripheral avoids external IC for simple UIs.

---

## HAL / embedded-hal Notes

```rust
pub trait TouchPad {
    type Error;
    fn is_touched(&mut self, ch: u8) -> Result<bool, Self::Error>;
}
```

Debounce in application (20–50 ms) even when hardware debounces edges.

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| False triggers | Noise / long wires | Shorter traces; series 1k |
| No response | Electrode not connected | Check pad solder |
| All bits set | Water / grease | Clean; dry |
| MPR121 NACK | ADDR pin | Try 0x5B |

---

## Example Project: Capacitive Piano

1. MPR121 with 8 copper tape keys.
2. Map touches to buzzer tones ([speakers.md](./speakers.md)).
3. Visualize active key on NeoPixel strip.

---

## Exercises

1. Tune MPR121 touch/release thresholds for 2 mm plastic overlay.
2. Implement edge detection (press vs release events).
3. Compare TTP223 vs ESP32 native touch on same pad layout.

---

## References

- TTP223 datasheet
- NXP MPR121 datasheet
- Espressif ESP32-S3 Touch Sensor docs
- [17-i2c.md](../17-i2c.md)
