# LSM6DS3 — 6-Axis IMU (STMicroelectronics)

The **LSM6DS3** (and **LSM6DS3TR-C**) is ST's low-power 6-DoF IMU — popular for wearables and IoT. No onboard magnetometer (pair with LIS3MDL for 9-DoF).

**Prerequisites:** [imu-overview.md](./imu-overview.md), [17-i2c.md](../17-i2c.md)

---

## Working Principle

Capacitive MEMS accelerometer + vibrating gyroscope on one die. Supports **pedometer**, **tilt**, and **free-fall** embedded functions via **MLC** (machine learning core) on TR variant — basic variant uses simpler interrupt engines.

See [imu-overview.md](./imu-overview.md) for axis conventions.

---

## Datasheet Key Points

| Parameter | Value |
|-----------|-------|
| WHO_AM_I (`0x0F`) | `0x69` (DS3) / `0x6A` (DS3TR-C) |
| I²C address | `0x6A` (SA0=0), `0x6B` (SA0=1) |
| ODR | Up to 6.66 kHz (gyro), 6.66 kHz (accel) |
| VDD | 1.71–3.6 V |

---

## Protocol

**I²C** (400 kHz / 1 MHz) or **SPI** (3- or 4-wire). Registers are 8-bit address, 16-bit data **little-endian** (ST convention — opposite of InvenSense).

Auto-increment bit in sub-address MSB for burst read.

---

## Register Map (Essential)

| Addr | Name | Description |
|------|------|-------------|
| `0x0F` | WHO_AM_I | Chip ID |
| `0x10` | CTRL1_XL | Accel ODR, FS |
| `0x11` | CTRL2_G | Gyro ODR, FS |
| `0x12` | CTRL3_C | BDU, H_LACTIVE, IF_INC |
| `0x13` | CTRL4_C | I²C disable, DRDY masks |
| `0x19` | CTRL9_XL | Accel user offset |
| `0x22` | OUTX_L_G | Gyro data start |
| `0x28` | OUTX_L_XL | Accel data start |
| `0x20` | STATUS_REG | DRDY flags |

### CTRL3_C Important Bits

| Bit | Name | Purpose |
|-----|------|---------|
| 6 | BDU | Block data update — latch until both bytes read |
| 2 | IF_INC | Auto-increment on burst read |

**Always enable BDU + IF_INC** for consistent multi-byte reads.

---

## ESP32-S3 Wiring

```
ESP32-S3          LSM6DS3 Breakout
────────          ─────────────────
GPIO8  (SDA) ───► SDA
GPIO9  (SCL) ───► SCL
3V3          ───► VDD
GND          ───► GND
GPIO7        ───► INT1 (DRDY)
```

---

## Rust Driver Sketch

```rust
use embedded_hal::i2c::I2c;

const ADDR: u8 = 0x6A;

pub struct Lsm6ds3<I2C> {
    i2c: I2C,
}

impl<I2C, E> Lsm6ds3<I2C>
where
    I2C: I2c<Error = E>,
{
    pub fn init(&mut self) -> Result<(), E> {
        assert_eq!(self.read_reg(0x0F)?, 0x69);
        self.write_reg(0x12, 0x44)?; // BDU | IF_INC
        self.write_reg(0x10, 0x60)?; // accel 416 Hz, ±2g
        self.write_reg(0x11, 0x60)?; // gyro 416 Hz, ±245 dps
        Ok(())
    }

    pub fn read_accel(&mut self) -> Result<[i16; 3], E> {
        let mut b = [0u8; 6];
        self.i2c.write_read(ADDR, &[0x28], &mut b)?;
        Ok([
            i16::from_le_bytes([b[0], b[1]]),
            i16::from_le_bytes([b[2], b[3]]),
            i16::from_le_bytes([b[4], b[5]]),
        ])
    }
}
```

**Crates:** `lsm6ds3`, `lsm6dsxx`.

---

## Bare-Metal Notes

- **Little-endian** — use `from_le_bytes`, not `from_be_bytes`.
- Scale factors depend on FS bits in CTRL1_XL / CTRL2_G (datasheet Table 3).
- **FIFO** up to 4096 bytes — configure via `FIFO_CTRL1`–`4` for batch logging.
- **Self-test** procedure in datasheet — verify mechanical integrity after reflow.

---

## HAL / embedded-hal Notes

```rust
pub trait Accelerometer {
    type Error;
    fn read_mg(&mut self) -> Result<[f32; 3], Self::Error>;
}
```

Map raw to mg: `mg = raw * sensitivity_mg_per_lsb` from selected full scale.

Connect INT1 to GPIO with interrupt for DRDY-driven sampling ([09-interrupts.md](../09-interrupts.md)).

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| WHO_AM_I 0x6A | TR-C variant | Accept or branch on ID |
| Stale data | BDU off | Set CTRL3_C BDU |
| Axis swap | Board rotation | Remap in software |
| I²C NACK at 0x6A | SA0 high | Try `0x6B` |

---

## Example Project: Tap Detector

1. Enable INT1 on double-tap (registers `0x58`–`0x5A` wake-up config).
2. On IRQ, read timestamp and flash NeoPixel ([neopixel-ws2812.md](../displays/neopixel-ws2812.md)).
3. Debounce in software.

---

## Exercises

1. Enable FIFO stream mode; drain 100 samples per interrupt.
2. Compare accel noise at 26 Hz vs 833 Hz ODR.
3. Pair with LIS3MDL magnetometer for 9-DoF on same I²C bus.

---

## References

- ST LSM6DS3 datasheet (DS11454)
- AN5040 — FIFO usage
- [imu-overview.md](./imu-overview.md), [17-i2c.md](../17-i2c.md)
