# INA219 — I²C Current/Power Monitor

The **Texas Instruments INA219** measures **shunt voltage**, **bus voltage**, and computes **current** and **power** over I²C.

**Prerequisites:** [17-i2c.md](../17-i2c.md), [07-hal.md](../07-hal.md)

---

## Working Principle

High-side or low-side **shunt resistor** (typically 0.1 Ω) develops voltage proportional to current. INA219 amplifies shunt voltage with programmable gain (**PGA**) and digitizes with 12-bit ADC. Bus voltage channel measures VIN+ up to 26 V.

---

## Datasheet Key Points

| Parameter | Value |
|-----------|-------|
| I²C address | `0x40`–`0x4F` (A0/A1 pins) |
| Shunt FSR | ±40 mV, ±80 mV, ±160 mV, ±320 mV (PGA) |
| Bus voltage | 0–26 V |
| Resolution | 12-bit |
| Calibration reg | User sets for current LSB |

---

## Protocol

Standard I²C register access. Registers are **16-bit**, big-endian.

---

## Register Map

| Addr | Name | Description |
|------|------|-------------|
| `0x00` | CONFIG | Reset, bus/shunt ADC, PGA, mode |
| `0x01` | SHUNT_VOLTAGE | Signed, 10 µV LSB |
| `0x02` | BUS_VOLTAGE | 4 mV LSB, shift right 3 |
| `0x03` | POWER | 20× current LSB |
| `0x04` | CURRENT | Signed, scaled by CAL |
| `0x05` | CALIBRATION | Sets current LSB |

### CONFIG Bits (0x00)

| Bits | Field |
|------|-------|
| 15 | RST reset |
| 13 | BRNG (0=16V, 1=32V bus range) |
| 12–11 | PGA gain |
| 10–8 | BADC bus ADC |
| 7–4 | SADC shunt ADC |
| 3–0 | Mode (continuous shunt+bus, etc.) |

### Calibration

Choose current LSB (e.g. 0.1 mA):

```text
CAL = trunc(0.04096 / (current_LSB × R_shunt))
```

Write CAL to `0x05`. Current register × LSB = amps.

---

## ESP32-S3 Wiring

```
INA219 Module     ESP32-S3        Load
─────────────     ────────        ────
VCC  ───────────► 3V3
GND  ───────────► GND
SDA  ───────────► GPIO8
SCL  ───────────► GPIO9
VIN+ ──── inline shunt ──► Load +
VIN- ─────────────────────► Load -
```

Shunt is on module (0.1 Ω) for low current; use external shunt for high current.

---

## Rust Driver Sketch

```rust
use embedded_hal::i2c::I2c;

const ADDR: u8 = 0x40;
const R_SHUNT: f32 = 0.1; // ohms

pub struct Ina219<I2C> {
    i2c: I2C,
    current_lsb: f32,
}

impl<I2C, E> Ina219<I2C>
where
    I2C: I2c<Error = E>,
{
    pub fn init(&mut self) -> Result<(), E> {
        let cal = (0.04096 / (0.0001 * R_SHUNT)) as u16;
        self.write_reg(0x05, cal)?;
        self.current_lsb = 0.0001;
        // Config: 32V range, PGA /8, 12-bit, continuous
        self.write_reg(0x00, 0x019F)?;
        Ok(())
    }

    pub fn read_current_ma(&mut self) -> Result<f32, E> {
        let raw = self.read_reg(0x04)? as i16;
        Ok(raw as f32 * self.current_lsb * 1000.0)
    }

    pub fn read_bus_voltage(&mut self) -> Result<f32, E> {
        let raw = self.read_reg(0x02)? >> 3;
        Ok(raw as f32 * 0.004)
    }
}
```

**Crates:** `ina219`, `ina2xx`.

---

## Bare-Metal Notes

- **Sign convention** — current direction depends on shunt orientation (datasheet diagram).
- **Averaging** — CONFIG ADC mode selects conversion time (140 µs – 8.5 ms); longer = quieter.
- **Overflow** — if shunt voltage exceeds PGA range, reading invalid — increase PGA or smaller shunt.

---

## HAL / embedded-hal Notes

```rust
pub trait PowerMonitor {
    type Error;
    fn read_mw(&mut self) -> Result<f32, Self::Error>;
}
```

Combine with ESP32 deep sleep policies to log energy per wake cycle.

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| Zero current | Shunt not in path | Wire VIN+ / VIN- in series |
| Negative current | Reverse connection | Swap shunt leads |
| Saturated | PGA too low | Increase PGA or reduce R_shunt |
| Wrong address | A0/A1 straps | Scan 0x40–0x4F |

---

## Example Project: Solar Charger Monitor

1. INA219 on panel → battery line.
2. Log mA and V every 10 s to flash.
3. Display power on ILI9341 dial.

---

## Exercises

1. Compute CAL for 1 mA LSB with 0.05 Ω shunt.
2. Measure ESP32-S3 active vs Wi-Fi TX current draw.
3. Trigger alert when bus voltage drops below 3.0 V.

---

## References

- TI INA219 datasheet (SBOS548)
- [17-i2c.md](../17-i2c.md)
- `ina219` crate
