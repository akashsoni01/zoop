# Project: Frequency Counter

Measure input signal frequency on GPIO using PCNT or timer capture.

---

## BOM

ESP32-S3, signal source (function generator or 555 timer).

---

## Wiring

```
GPIO4 ── square wave input (0–3.3 V)
GND common
```

---

## Firmware Plan

1. PCNT or pulse counter for gate time 1 s
2. Display on [oled.md](../examples/oled.md)
3. [mqtt.md](../communication/mqtt.md) publish reading

---

## Testing

- [ ] 1 kHz ±1 Hz accuracy
- [ ] 100 kHz upper limit documented

---

## Extensions

- Duty cycle measurement
- Period min/max stats

**Related:** [timers.md](../examples/timers.md)
