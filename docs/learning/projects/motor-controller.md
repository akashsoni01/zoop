# Project: Motor Controller

PID closed-loop speed control for DC motor with encoder feedback.

---

## BOM

| Part | Qty |
|------|-----|
| ESP32-S3 | 1 |
| DRV8833 or L298N | 1 |
| DC motor + quadrature encoder | 1 |
| Power supply 6–12V | 1 |

---

## Wiring

```
GPIO12/13 ── motor driver inputs
GPIO14    ── PWM speed
GPIO18/19 ── encoder A/B
VMOT separate from USB 5V
```

---

## Firmware Plan

1. [motor-driver.md](../examples/motor-driver.md) open-loop spin
2. [encoder.md](../examples/encoder.md) — RPM measurement
3. PID loop @ 100 Hz timer — [timers.md](../examples/timers.md)
4. Optional [CAN](../communication/can.md) setpoint from [can-analyzer.md](./can-analyzer.md)

---

## Testing

- [ ] Step response logged (setpoint 50% → stable < 2 s)
- [ ] Direction reversal without runaway

---

## Extensions

- [Modbus](../communication/modbus.md) holding register for speed
- Current sense via [adc.md](../examples/adc.md)
