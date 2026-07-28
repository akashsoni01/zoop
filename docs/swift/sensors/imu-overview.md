# IMU Overview — Motion Sensing Concepts

Conceptual guide to **Inertial Measurement Units (IMUs)**: accelerometers, gyroscopes, magnetometers, fusion, and how to choose a part before diving into device-specific drivers.

**Prerequisites:** [17-i2c.md](../17-i2c.md), [16-spi.md](../16-spi.md)

---

## Working Principle

### Accelerometer

Measures **specific force** (including gravity) on three axes via MEMS capacitive or piezo elements. At rest, Z reads ~1 g when axis points up. Used for tilt, vibration, tap detection, and free-fall.

### Gyroscope

Measures **angular rate** (°/s) around three axes. Integrating rate yields orientation change (drifts over time). Used for rotation tracking and complementary filtering with accel.

### Magnetometer

Measures **magnetic field** (µT) for compass heading. Distorted by motors, speakers, and PCB current — requires hard/soft-iron calibration.

### 6-DoF vs 9-DoF

| Type | Sensors | Output |
|------|---------|--------|
| 6-DoF | Accel + Gyro | Tilt, rotation rate |
| 9-DoF | Accel + Gyro + Mag | Absolute heading (with fusion) |

---

## Datasheet Notes (General)

When comparing IMU datasheets, check:

| Parameter | Why It Matters |
|-----------|----------------|
| Full-scale range (FSR) | ±2g vs ±16g accel; ±250 vs ±2000 °/s gyro |
| LSB sensitivity | Scale raw int16 to physical units |
| Output data rate (ODR) | 10 Hz vs 1 kHz — power and aliasing |
| Digital low-pass filter (DLPF) | Noise vs bandwidth trade-off |
| FIFO depth | Batch reads reduce I²C traffic |
| Interface | I²C address, SPI mode, max clock |

---

## Protocol

Most hobby IMUs use **I²C** (400 kHz) or **SPI** (often mode 0 or 3). Register data is typically **big-endian 16-bit** two's complement.

See [17-i2c.md](../17-i2c.md) and [16-spi.md](../16-spi.md) for ESP32-S3 bus wiring.

---

## Register Map (Common Patterns)

While each chip differs, IMU register layouts share patterns:

| Typical Addr | Name | Purpose |
|--------------|------|---------|
| `WHO_AM_I` | ID | Verify device (0x68, 0x47, etc.) |
| `PWR_MGMT` | Power | Exit sleep, select clock |
| `CONFIG` / `DLPF` | Filter | Bandwidth |
| `*_CONFIG` | FSR | Accel/gyro full-scale |
| `DATA_START` | Burst | 6–14 bytes XYZ raw |

Always read WHO_AM_I before configuration.

---

## Swift Driver Sketch (Generic IMU Protocol)

```swift
struct Vec3f: Sendable {
    var x: Float, y: Float, z: Float
}

protocol IMU6Axis {
    mutating func initDevice() throws
    mutating func readAccel() throws -> Vec3f   // units: g
    mutating func readGyro() throws -> Vec3f    // units: °/s
    mutating func readTempC() throws -> Float
}

struct ComplementaryFilter {
    var alpha: Float = 0.98
    var pitch: Float = 0
    var roll: Float = 0

    mutating func update(accel: Vec3f, gyro: Vec3f, dt: Float) {
        let accelPitch = atan2(accel.y, sqrt(accel.x * accel.x + accel.z * accel.z))
        let accelRoll = atan2(-accel.x, accel.z)
        pitch = alpha * (pitch + gyro.x * dt) + (1 - alpha) * accelPitch
        roll = alpha * (roll + gyro.y * dt) + (1 - alpha) * accelRoll
    }
}
```

Device-specific structs (`Mpu6050`, `Lsm6ds3`) implement `IMU6Axis`.

---

## Bare-Metal Notes

- Wake from sleep: clear sleep bit in power management register, wait ~100 ms for oscillator stable.
- Align sensor axes with your PCB silkscreen — chips may be mounted rotated.
- High ODR + polling loop can saturate I²C; use FIFO or lower rate for background tasks.

---

## HAL / Protocol-Oriented Driver Notes

Define `IMU6Axis` and optional `IMU9Axis` (adds magnetometer) protocols. Keep scaling constants (LSB/g, LSB/°/s) as `let` properties set at init from FSR config. Fusion filters live in separate pure-Swift modules.

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| WHO_AM_I mismatch | Wrong chip or bad wiring | Verify part number and [17-i2c.md](../17-i2c.md) |
| Saturation (±32768) | FSR too small | Increase full-scale range |
| Drifting heading | No mag or no fusion | Add magnetometer + calibration |
| Noisy accel at rest | DLPF off / vibration | Enable DLPF; mechanical isolation |

---

## Example Project

**Digital level:** MPU-6050 ([mpu6050.md](./mpu6050.md)) with complementary filter. Draw bubble level on ST7789 ([displays/st7789.md](../displays/st7789.md)). Calibrate zero-g offset at startup.

---

## References

- [MPU-6050 Register Map (InvenSense)](https://invensense.tdk.com/products/motion-tracking/6-axis/mpu-6050/)
- [mpu6050.md](./mpu6050.md), [mpu9250.md](./mpu9250.md), [lsm6ds3.md](./lsm6ds3.md)
- [17-i2c.md](../17-i2c.md), [16-spi.md](../16-spi.md)
