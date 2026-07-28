# Project: Differential Drive Robot

Two motors, encoders, IMU — mobile platform firmware.

---

## BOM

ESP32-S3, 2× motor+encoder, [MPU6050](../sensors/mpu6050.md), LiPo + regulator, chassis.

---

## Wiring

```
Left motor  ── driver CH A (GPIO12-14)
Right motor ── driver CH B (GPIO15-17)
IMU I²C ── GPIO8/9 @ 0x68
```

---

## Firmware Plan

1. [motor-driver.md](../examples/motor-driver.md) + [encoder.md](../examples/encoder.md)
2. IMU read — [sensors/imu-overview.md](../sensors/imu-overview.md)
3. Kinematics: `v, ω → left_rpm, right_rpm`
4. Teleop via [mqtt.md](../communication/mqtt.md) or BLE

---

## Testing

- [ ] Straight line test 1 m (encoder odometry)
- [ ] 90° turn repeatable ±5°

---

## Extensions

- SLAM (off-device processing)
- [CAN](../communication/can.md) bus to arm module
