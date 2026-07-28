# Project: Digital Oscilloscope (Entry Level)

Stream [ADC](../examples/adc.md) samples to web UI via [Wi-Fi](../communication/wifi.md).

---

## BOM

ESP32-S3, voltage divider/probe (≤3.3 V input!), optional [SPI display](../examples/spi-display.md).

---

## Wiring

```
Signal ── 100k ── GPIO6 (ADC) ── 100k ── GND
         (max 3.3 V at pin — use divider for higher)
```

---

## Firmware Plan

1. ADC continuous mode / timer trigger @ 10–100 kHz
2. Ring buffer in RAM
3. [websockets.md](../communication/websockets.md) binary stream to browser canvas
4. Trigger: rising edge threshold

---

## Testing

- [ ] Displays 1 kHz sine (function generator, attenuated)
- [ ] No clipping at full scale

---

## Extensions

- Dual channel second ADC pin
- FFT on host

**Warning:** Never exceed 3.3 V on GPIO — use proper probe/divider.
