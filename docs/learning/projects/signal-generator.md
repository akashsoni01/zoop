# Project: Signal Generator

Output square/sine via [PWM](../examples/pwm.md) or [DAC](../examples/dac.md).

---

## BOM

ESP32-S3, MCP4725 I²C DAC or RC-filtered PWM, optional amplifier.

---

## Wiring

```
PWM GPIO5 ── 10k ── 100nF ── GND (low-pass) ── OUT
Or MCP4725 on I²C GPIO8/9
```

---

## Firmware Plan

1. [pwm.md](../examples/pwm.md) variable frequency square
2. Sine table DDS via timer interrupt
3. [encoder.md](../examples/encoder.md) adjust frequency
4. [oled.md](../examples/oled.md) show Hz

---

## Testing

- [ ] 100 Hz–10 kHz range stable
- [ ] Sine THD acceptable for bench use (scope FFT)

---

## Extensions

- Sweep mode for filter characterization
- Pair with [frequency-counter.md](./frequency-counter.md) loopback test

**Lesson:** [11-pwm.md](../11-pwm.md), [13-dac.md](../13-dac.md)
