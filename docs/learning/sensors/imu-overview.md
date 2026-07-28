# IMU Concepts — Accelerometer, Gyroscope, Magnetometer

An **Inertial Measurement Unit (IMU)** fuses motion sensors to track orientation, acceleration, and (with magnetometer) heading. This guide covers theory before device-specific drivers ([mpu6050.md](./mpu6050.md), [lsm6ds3.md](./lsm6ds3.md)).

**Prerequisites:** [07-hal.md](../07-hal.md), [12-adc.md](../12-adc.md) (conceptual), [09-interrupts.md](../09-interrupts.md)

---

## Working Principles

### Accelerometer

Measures **specific force** (includes gravity). MEMS devices use a proof mass on springs; capacitive or piezo sensing detects displacement.

| Axis reading | At rest (flat) | Free fall |
|--------------|----------------|-----------|
| Z | ≈ +1 g (or −1 g by convention) | ≈ 0 g |
| X, Y | ≈ 0 g | ≈ 0 g |

**Units:** g (9.80665 m/s²) or m/s². Registers usually store **LSB** scaled by selected range (±2g, ±4g, etc.).

### Gyroscope

Measures **angular rate** (°/s or rad/s) around each axis. MEMS gyros use **Coriolis effect** on a vibrating mass.

Integrating gyro rate yields **orientation change**, but **bias drift** accumulates over time — hence sensor fusion.

### Magnetometer

Measures **magnetic field** (µT) to find heading relative to Earth's field — like a compass.

Distorted by hard-iron (permanent magnets) and soft-iron (ferromagnetic materials) effects. Requires calibration.

---

## Degrees of Freedom (DoF)

| Name | Sensors | Outputs |
|------|---------|---------|
| 6-DoF | Accel + Gyro | Tilt, rotation rate |
| 9-DoF | Accel + Gyro + Mag | Absolute heading (yaw) with fusion |
| 10-DoF | Above + barometer | Altitude aiding |

Common chips: MPU-6050 (6-DoF), MPU-9250 / ICM-20948 (9-DoF), LSM6DS3 (6-DoF, no mag).

---

## Coordinate Frames

**Sensor frame** — fixed to chip package (check datasheet diagram for X/Y/Z arrow directions).

**Body frame** — fixed to your product (e.g., phone: X right, Y up, Z out of screen).

Always apply a **rotation matrix** or simple axis remap in software when mounting the board at an angle.

Right-hand rule: positive rotation about +X is roll, +Y pitch, +Z yaw (common aviation convention — verify your fusion library).

---

## Key Datasheet Parameters

| Parameter | Meaning |
|-----------|---------|
| Full-scale range (FSR) | ±2/4/8/16 g (accel), ±250–2000 °/s (gyro) |
| Sensitivity | LSB/g or LSB/(°/s) — changes with FSR |
| Zero-rate offset | Gyro bias at rest |
| Noise density | µg/√Hz — sets effective resolution |
| ODR | Output data rate (Hz) |
| FIFO | On-chip buffer for burst reads / low-power |

---

## Protocol (Typical)

Most IMUs use **I²C** (400 kHz) or **SPI** (≥1 MHz for high ODR). Register map pattern:

```
WHO_AM_I (ID) → verify chip
PWR_MGMT / CTRL → wake, select clocks
CONFIG → DLPF (digital low-pass filter)
GYRO/ACCEL config → FSR
DATA registers → burst read 6–14 bytes
INT pin → data ready / motion detect / FIFO watermark
```

See [17-i2c.md](../17-i2c.md), [16-spi.md](../16-spi.md).

---

## Register Map Pattern (Generic)

| Offset | Content |
|--------|---------|
| `0x00` | WHO_AM_I |
| `0x06`–`0x0B` | Gyro X,Y,Z (16-bit, big-endian typical) |
| `0x0C`–`0x11` | Accel X,Y,Z |
| `0x12`–`0x17` | Temp (optional) |
| `0x1A` | Config (DLPF) |
| `0x1B` | Gyro config (FSR) |
| `0x1C` | Accel config (FSR) |
| `0x6B` | Power management |

Exact addresses differ by vendor — always use the chip-specific guide.

---

## Rust Driver Sketch (Generic 6-DoF)

```rust
use embedded_hal::i2c::I2c;

pub struct ImuReading {
    pub accel: [i16; 3],  // raw LSB
    pub gyro: [i16; 3],
    pub temp_c: Option<f32>,
}

pub trait SixDofImu {
    type Error;
    fn who_am_i(&mut self) -> Result<u8, Self::Error>;
    fn init(&mut self) -> Result<(), Self::Error>;
    fn read_raw(&mut self) -> Result<ImuReading, Self::Error>;
    fn accel_g(&self, raw: i16) -> f32 {
        // scale depends on configured FSR
        raw as f32 / self.accel_lsb_per_g()
    }
}
```

---

## Bare-Metal Driver Notes

1. **Wake sequence** — many IMUs ship in sleep; write PWR_MGMT before reads.
2. **Big-endian** — InvenSense/ST typically MSB first for 16-bit axes.
3. **Burst read** — one I²C transaction from first data register reduces skew between axes.
4. **DLPF** — trade bandwidth vs noise; 42 Hz DLPF is a common starting point.
5. **Sample sync** — gyro and accel at same ODR when fused.

---

## HAL / embedded-hal Notes

Higher-level ecosystem:

| Crate | Role |
|-------|------|
| `embedded-hal` | I2c, SpiDevice, InputPin (INT) |
| `nalgebra` / `micromath` | Rotation math in `no_std` |
| `ahrs` filters | Madgwick, Mahony (custom or port) |

Driver crates (`mpu6050`, `icm20948`, `lsm6ds3`) expose init + read; fusion stays application-side.

---

## Sensor Fusion Overview

| Algorithm | Inputs | Pros | Cons |
|-----------|--------|------|------|
| Complementary filter | Accel + Gyro | Simple | Manual tuning |
| Madgwick | 6 or 9-DoF | Good performance | CPU cost |
| Mahony | 6 or 9-DoF | Lighter than Madgwick | Similar |
| Kalman (EKF) | All + GPS | Best tracking | Complex, tuning |

**Accel** gives long-term "down" reference; **gyro** gives short-term smooth rotation; **mag** fixes yaw drift (when calibrated).

---

## Troubleshooting (General)

| Symptom | Cause | Fix |
|---------|-------|-----|
| WHO_AM_I wrong | Wrong address, bad wiring | Scan I²C; check AD0 pin |
| Saturated readings | FSR too small | Increase to ±8g / ±500 °/s |
| Gyro drift | Bias, temperature | Calibrate at rest; use DLPF |
| Yaw spins | No mag or bad cal | 9-DoF + mag calibration |
| Clipped FIFO | ODR > read rate | Raise read frequency or lower ODR |

---

## Example Project: Tilt Game Controller

MPU-6050 over I²C:

1. Read accel at 100 Hz.
2. Compute pitch/roll via `atan2` (`micromath`).
3. Map tilt to cursor on SSD1306 ([ssd1306.md](../displays/ssd1306.md)).
4. Button resets neutral (store offset bias).

---

## Exercises

1. Plot raw accel when rotating board 90° on each axis — verify sign conventions.
2. Estimate gyro bias: average 1000 samples at rest, subtract from readings.
3. Implement complementary filter (95% gyro, 5% accel) for one axis tilt.
4. Compare DLPF settings — log noise standard deviation at rest.

---

## References

- [mpu6050.md](./mpu6050.md) — 6-DoF InvenSense
- [mpu9250.md](./mpu9250.md) — 9-DoF
- [icm20948.md](./icm20948.md) — modern 9-DoF
- [lsm6ds3.md](./lsm6ds3.md) — ST 6-DoF
- Sebastian Madgwick's AHRS paper
- [17-i2c.md](../17-i2c.md)
