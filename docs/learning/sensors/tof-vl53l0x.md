# VL53L0X — Time-of-Flight Distance Sensor

The **STMicro VL53L0X** measures distance up to ~2 m using **940 nm VCSEL** laser time-of-flight with SPAD array.

**Prerequisites:** [17-i2c.md](../17-i2c.md)

---

## Working Principle

Emits short laser pulses; measures photon return time to compute **time-of-flight (ToF)**. On-chip microcontroller runs ranging firmware — host reads result via I²C (no timing-critical GPIO).

---

## Datasheet Key Points

| Parameter | Value |
|-----------|-------|
| I²C address | `0x29` (default, programmable) |
| Range | up to 2000 mm (mode dependent) |
| Resolution | 1 mm |
| Field of view | ~25° |
| VDD | 2.6–3.5 V |

---

## Protocol

**I²C** up to 400 kHz. 16-bit register index (8-bit in practice). Requires **2.8 V** internal core — breakout includes LDO; connect **3V3** and **2.8V** (often tied on module) per schematic.

**XSHUT** pin: active low shutdown; pull high for operation. Use to set alternate addresses on multiple sensors.

---

## Register Map (Essential)

| Addr | Name | Description |
|------|------|-------------|
| `0x00` | `SYSRANGE_START` | Write `0x01` to start ranging |
| `0x01` | `SYSTEM_THRESH_HIGH` | Interrupt threshold |
| `0x13` | `RESULT_INTERRUPT_STATUS` | New sample ready |
| `0x14` | `RESULT_RANGE_STATUS` | Validity, error codes |
| `0x1E` | `RESULT_FINAL_CROSSTALK_CORRECTED_RANGE_MM` | Distance mm MSB |
| `0x1F` | | Distance mm LSB |
| `0x52` | `HISTOGRAM_CONFIG` | (init sequence) |
| `0x83` | | Magic init values in ST driver |

**Important:** Full init is **~ dozens of register writes** from ST API — use proven init table or crate.

### Typical Read Sequence

1. Write `0x01` to `0x00` (start)
2. Poll `0x14` until bit 0 set (or wait 30 ms)
3. Read mm from `0x1E`–`0x1F`
4. Clear interrupt if using GPIO

---

## ESP32-S3 Wiring

```
VL53L0X           ESP32-S3
───────           ────────
VCC  ───────────► 3V3
GND  ───────────► GND
SDA  ───────────► GPIO8
SCL  ───────────► GPIO9
XSHUT───────────► GPIO6  (optional, for address change)
GPIO1 (INT) ◄──── GPIO7  (optional)
```

---

## Rust Driver Sketch

```rust
use embedded_hal::delay::DelayNs;
use embedded_hal::i2c::I2c;

const ADDR: u8 = 0x29;

pub struct Vl53l0x<I2C, DELAY> {
    i2c: I2C,
    delay: DELAY,
}

impl<I2C, DELAY, E> Vl53l0x<I2C, DELAY>
where
    I2C: I2c<Error = E>,
    DELAY: DelayNs,
{
    pub fn init(&mut self) -> Result<(), E> {
        st_vl53l0x_data_init(&mut self.i2c)?; // use crate's init table
        Ok(())
    }

    pub fn read_mm(&mut self) -> Result<u16, E> {
        self.write_reg(0x00, 0x01)?;
        self.delay.delay_ms(30);
        let mut buf = [0u8; 12];
        self.i2c.write_read(ADDR, &[0x14], &mut buf)?;
        let mm = ((buf[10] as u16) << 8) | (buf[11] as u16);
        Ok(mm)
    }
}
```

**Crates:** `vl53l0x`, `vl53l1x` (newer sibling).

---

## Bare-Metal Notes

- Do **not** hand-roll init unless you port ST's full sequence — sensor fails silently otherwise.
- **Cover glass** above sensor requires **crosstalk calibration** (ST tools).
- Multiple sensors: hold all XSHUT low, release one at a time, reprogram address `0x212` (see AN4846).
- Ranging fails on **out-of-range**, **too dark**, or **absorbing** targets (black cloth).

---

## HAL / embedded-hal Notes

```rust
pub trait DistanceSensor {
    type Error;
    fn read_mm(&mut self) -> Result<u16, Self::Error>;
}
```

Implement for VL53L0X; swap with ultrasonic in tests via trait object or generics.

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| Always 8191 / max | Init incomplete | Use crate init |
| Unstable mm | Target angle/ material | Perpendicular to matte surface |
| I²C NACK | XSHUT low | Drive XSHUT high |
| Short range outdoors | Sun IR saturation | Use indoors or cover |

---

## Example Project: Liquid Level

1. VL53L0X downward-facing in tank lid.
2. Convert mm to fill percentage; smooth with moving average.
3. Display on SSD1306 gauge.

---

## Exercises

1. Program second sensor at `0x30` using XSHUT sequencing.
2. Log 100 samples/sec; compute standard deviation on fixed target.
3. Compare with [lidar.md](./lidar.md) TF-Luna at 1 m.

---

## References

- ST VL53L0X datasheet (DS9844)
- AN4846 — Using multiple VL53L0X
- `vl53l0x` crate
- [17-i2c.md](../17-i2c.md)
