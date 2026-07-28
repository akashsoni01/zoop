# LSM6DS3 — 6-Axis IMU (ST)

The **STMicroelectronics LSM6DS3** is a low-power 6-DoF IMU with I²C/SPI, FIFO, and embedded pedometer functions.

**Prerequisites:** [imu-overview.md](./imu-overview.md), [17-i2c.md](../17-i2c.md)

---

## Working Principle

ST MEMS **accelerometer + gyroscope** with unified register interface. Supports hardware FIFO up to 4096 bytes and programmable interrupt pins for motion detection.

---

## Datasheet Notes

| Parameter | Value |
|-----------|-------|
| I²C address | `0x6A` / `0x6B` (SA0 pin) |
| WHO_AM_I (`0x0F`) | `0x69` |
| Accel FSR | ±2, ±4, ±8, ±16 g |
| Gyro FSR | ±125 … ±2000 dps |
| ODR | Up to 6.66 kHz (gyro) |

---

## Protocol

**I²C** or **SPI** (mode 3). Data registers support **auto-increment** burst read when bit 7 set on sub-address.

See [17-i2c.md](../17-i2c.md) for ESP32-S3 I²C wiring.

---

## Register Map

| Addr | Name | Description |
|------|------|-------------|
| `0x0F` | WHO_AM_I | `0x69` |
| `0x10` | CTRL1_XL | Accel ODR + FSR |
| `0x11` | CTRL2_G | Gyro ODR + FSR |
| `0x12` | CTRL3_C | BDU, auto-increment |
| `0x22`–`0x2D` | OUTX_L_G … | Gyro + accel data |
| `0x0E` | FIFO_CTRL1 | FIFO threshold |

Enable **BDU** (block data update) to prevent mixed old/new bytes during read.

---

## ESP32-S3 Wiring

```
ESP32-S3          LSM6DS3
────────          ───────
GPIO8  ─────────► SDA
GPIO9  ─────────► SCL
3V3    ─────────► VDD
GND    ─────────► GND
GPIO7  ─────────► INT1 (optional)
```

SA0 low → `0x6A`; SA0 high → `0x6B`.

---

## Swift Driver Sketch

```swift
struct LSM6DS3<B: I2CBus>: IMU6Axis {
    var bus: B
    let address: UInt8 = 0x6A
    var accelScale: Float = 0.061  // mg/LSB at ±2g
    var gyroScale: Float = 8.75    // mdps/LSB at ±250 dps

    mutating func initDevice() throws {
        guard try bus.read(from: address, register: 0x0F, count: 1)[0] == 0x69 else {
            throw SensorError.wrongDevice
        }
        try bus.write(to: address, register: 0x12, data: [0x44]) // BDU + auto-inc
        try bus.write(to: address, register: 0x10, data: [0x60]) // accel 416 Hz, ±2g
        try bus.write(to: address, register: 0x11, data: [0x60]) // gyro 416 Hz, ±250 dps
    }

    mutating func readAccel() throws -> Vec3f {
        let d = try bus.read(from: address, register: 0x28, count: 6)
        return Vec3f(
            x: Float(Int16(bitPattern: UInt16(d[0]) | UInt16(d[1]) << 8)) * accelScale / 1000.0,
            y: Float(Int16(bitPattern: UInt16(d[2]) | UInt16(d[3]) << 8)) * accelScale / 1000.0,
            z: Float(Int16(bitPattern: UInt16(d[4]) | UInt16(d[5]) << 8)) * accelScale / 1000.0
        )
    }
}
```

---

## Bare-Metal Notes

- LSM6DS3 data is **little-endian** (LSB first) — opposite of InvenSense.
- Use FIFO watermark interrupt for efficient batch processing.
- `CTRL3_C` SW_RESET bit for soft reset.

---

## HAL / Protocol-Oriented Driver Notes

Implement `IMU6Axis`. Provide `ODR` and `FullScale` enums mapping to register bit patterns. ST also offers LSM6DSO/LSM6DSOX with similar API — abstract common init pattern.

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| WHO_AM_I 0x6A | LSM6DSO variant | Check exact part number |
| Jittery values | BDU off | Set `CTRL3_C` BDU bit |
| Wrong scale | FSR mismatch | Update scale constants after config |

---

## Example Project

**Pedometer:** Enable embedded step counter (DS3 feature) or implement peak detection on accel Z at 52 Hz. Show step count on SSD1306.

---

## References

- [LSM6DS3 Datasheet (ST)](https://www.st.com/resource/en/datasheet/lsm6ds3.pdf)
- [imu-overview.md](./imu-overview.md)
- [17-i2c.md](../17-i2c.md)
