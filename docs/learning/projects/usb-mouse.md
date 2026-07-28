# Project: USB Mouse

HID relative pointing device with encoder or IMU.

---

## BOM

ESP32-S3, rotary encoder or [MPU6050](../sensors/mpu6050.md), optional buttons.

---

## Wiring

Encoder on GPIO18/19 — [encoder.md](../examples/encoder.md).

---

## Firmware Plan

1. [usb-hid.md](../examples/usb-hid.md) mouse report descriptor
2. Encoder delta → `x` movement
3. Buttons on GPIO4/5

---

## Testing

- [ ] Smooth cursor movement
- [ ] Click events register

---

## Extensions

- Scroll wheel second encoder
- Air mouse using IMU fusion
