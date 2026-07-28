# ADS1115 — 16-Bit I²C ADC

The **Texas Instruments ADS1115** is a **16-bit delta-sigma ADC** with I²C interface — ideal for precision voltage measurements beyond the MCU's built-in ADC.

**Prerequisites:** [12-adc.md](../12-adc.md), [17-i2c.md](../17-i2c.md)

---

## Working Principle

Delta-sigma converter integrates input over programmable **sample time**, trading speed for noise rejection. Four single-ended or two differential channels via internal **MUX**.

---

## Datasheet Key Points

| Parameter | Value |
|-----------|-------|
| I²C address | `0x48`–`0x4B` (ADDR pin) |
| Resolution | 16 bits signed |
| Sample rate | 8–860 SPS (programmable) |
| PGA | ±256 mV to ±6.144 V full scale |
| Channels | 4 SE or 2 diff |

---

## Protocol

I²C pointer register + 16-bit config/conversion register, **big-endian**.

---

## Register Map

| Pointer | Name | Description |
|---------|------|-------------|
| `0x00` | Conversion | Latest result (signed 16-bit) |
| `0x01` | Config | MUX, PGA, mode, data rate |
| `0x02` | Lo_thresh | Comparator (optional) |
| `0x03` | Hi_thresh | Comparator (optional) |

### CONFIG Register (0x01)

| Bits | Field | Notes |
|------|-------|-------|
| 15 | OS | Start single conversion (write 1) |
| 14–12 | MUX | Input selection |
| 11–9 | PGA | Full-scale range |
| 8 | MODE | 0 continuous, 1 single-shot |
| 7–5 | DR | Data rate |
| 4 | COMP_MODE | |
| 0 | QUEUE | Disable comp: `11` |

### MUX Codes (single-ended)

| Code | Input |
|------|-------|
| 100 | AIN0 vs GND |
| 101 | AIN1 vs GND |
| 110 | AIN2 vs GND |
| 111 | AIN3 vs GND |

---

## ESP32-S3 Wiring

```
ADS1115          ESP32-S3
───────          ────────
VDD  ──────────► 3V3
GND  ──────────► GND
SDA  ──────────► GPIO8
SCL  ──────────► GPIO9
AIN0 ◄── sensor voltage (0–4 V with PGA)
```

Use **voltage divider** if measuring > VDD range at PGA setting.

---

## Rust Driver Sketch

```rust
use embedded_hal::delay::DelayNs;
use embedded_hal::i2c::I2c;

const ADDR: u8 = 0x48;

pub struct Ads1115<I2C, DELAY> {
    i2c: I2C,
    delay: DELAY,
    pga_v: f32, // full scale voltage (e.g. 4.096)
}

impl<I2C, DELAY, E> Ads1115<I2C, DELAY>
where
    I2C: I2c<Error = E>,
    DELAY: DelayNs,
{
    pub fn read_channel(&mut self, mux: u16) -> Result<f32, E> {
        let config = 0x8000 | mux | 0x0100 | 0x0080 | 0x0003; // OS|mux|PGA|single|860sps
        self.write_reg(0x01, config)?;
        self.delay.delay_ms(2);
        let raw = self.read_reg(0x00)? as i16;
        Ok(raw as f32 / 32768.0 * self.pga_v)
    }
}
```

**Crates:** `ads1x1x`, `ads1115`.

---

## Bare-Metal Notes

- **Single-shot:** poll OS bit or wait `1/DR + 1 ms`.
- **Continuous mode:** read conversion reg anytime; watch for stale data at low DR.
- **Floating inputs** read noise — tie unused channels to GND.
- Lower **SPS** improves 50/60 Hz rejection ( mains noise).

---

## HAL / embedded-hal Notes

Wrap as generic analog front-end:

```rust
pub trait VoltageReader {
    type Error;
    fn read_volts(&mut self, ch: u8) -> Result<f32, Self::Error>;
}
```

Use for gas sensors ([gas-sensors.md](./gas-sensors.md)) needing better resolution than ESP32 SAR ADC.

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| Saturated ±32768 | Input > PGA | Increase PGA range or divider |
| Noisy readings | Long wires | Twisted pair; 0.1 µF on AIN |
| Wrong channel | MUX typo | Verify config nibble |
| I²C NACK | ADDR pin | Try 0x49, 0x4A, 0x4B |

---

## Example Project: 4-Channel Data Logger

1. AIN0–3: potentiometers or analog sensors.
2. Scan at 10 SPS; print CSV over USB-CDC.
3. Optional: compare with ESP32 internal ADC ([12-adc.md](../12-adc.md)).

---

## Exercises

1. Measure LSB noise at 8 SPS vs 860 SPS on shorted input.
2. Differential read across thermocouple amplifier.
3. Build auto-ranging wrapper that switches PGA when near limits.

---

## References

- TI ADS1115 datasheet (SBAS444)
- [12-adc.md](../12-adc.md), [17-i2c.md](../17-i2c.md)
- `ads1x1x` crate
