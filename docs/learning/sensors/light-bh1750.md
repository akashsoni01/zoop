# BH1750 — Ambient Light Sensor

The **ROHM BH1750** measures ambient light in **lux** using a photodiode array and internal ADC — simple **I²C** readout.

**Prerequisites:** [17-i2c.md](../17-i2c.md)

---

## Working Principle

Photodiodes detect visible light; internal logic converts to lux value based on measurement mode (continuous or one-time, high or low resolution).

---

## Datasheet Key Points

| Parameter | Value |
|-----------|-------|
| I²C address | `0x23` (ADDR low) / `0x5C` (ADDR high) |
| Range | ~1–65535 lx (mode dependent) |
| VDD | 2.4–3.6 V |
| Interface | I²C only |

---

## Protocol

BH1750 uses **command codes** (no register address byte):

| Command | Code | Description |
|---------|------|-------------|
| Power down | `0x00` | Low power |
| Power on | `0x01` | Wake |
| Reset | `0x07` | Clear data |
| Continuous H-res | `0x10` | 1 lx resolution, ~120 ms |
| One-time H-res | `0x20` | Single measurement |

After measurement command, read **2 bytes** (big-endian); lux = `raw / 1.2`.

---

## Register Map

Command-based — no traditional register file. Treat power/measurement as opcodes.

---

## ESP32-S3 Wiring

```
BH1750           ESP32-S3
──────           ────────
VCC  ──────────► 3V3
GND  ──────────► GND
SDA  ──────────► GPIO8
SCL  ──────────► GPIO9
ADDR ── GND ───► (address 0x23)
```

---

## Rust Driver Sketch

```rust
use embedded_hal::delay::DelayNs;
use embedded_hal::i2c::I2c;

const ADDR: u8 = 0x23;

pub struct Bh1750<I2C, DELAY> {
    i2c: I2C,
    delay: DELAY,
}

impl<I2C, DELAY, E> Bh1750<I2C, DELAY>
where
    I2C: I2c<Error = E>,
    DELAY: DelayNs,
{
    pub fn init(&mut self) -> Result<(), E> {
        self.i2c.write(ADDR, &[0x01])?; // power on
        self.i2c.write(ADDR, &[0x10])?; // continuous high-res
        self.delay.delay_ms(180);
        Ok(())
    }

    pub fn read_lux(&mut self) -> Result<f32, E> {
        let mut buf = [0u8; 2];
        self.i2c.read(ADDR, &mut buf)?;
        let raw = ((buf[0] as u16) << 8) | buf[1] as u16;
        Ok(raw as f32 / 1.2)
    }
}
```

**Crates:** `bh1750`.

---

## Bare-Metal Notes

- Wait **120 ms** (H-mode) or **16 ms** (L-mode) after one-time command before read.
- Window glass attenuates lux — calibrate for enclosure.
- I²C bus can share with other sensors ([bme280.md](./bme280.md)).

---

## HAL / embedded-hal Notes

```rust
pub trait AmbientLight {
    type Error;
    fn read_lux(&mut self) -> Result<f32, Self::Error>;
}
```

Map lux to NeoPixel brightness ([neopixel-ws2812.md](../displays/neopixel-ws2812.md)) for auto-dimming UI.

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| Always 0 | Sensor covered | Uncover; power on command |
| Stuck value | Continuous mode cached | Use one-time for fresh sample |
| Wrong address | ADDR pin | Try 0x5C |

---

## Example Project: Auto Backlight

1. BH1750 reads ambient lux.
2. PWM backlight on ST7789 ([st7789.md](../displays/st7789.md)) inversely proportional to lux.

---

## Exercises

1. Compare outdoor vs indoor readings; log min/max over 24 h.
2. Implement power-down between readings for battery savings.
3. Fuse BH1750 with BME280 in one I²C scan routine.

---

## References

- ROHM BH1750FVI datasheet
- [17-i2c.md](../17-i2c.md)
- `bh1750` crate
