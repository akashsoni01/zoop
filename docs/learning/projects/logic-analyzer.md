# Project: Logic Analyzer

Capture digital GPIO patterns — up to 8 channels via GPIO expander or direct pins.

---

## BOM

ESP32-S3, 74HC165 shift register (optional 8+ inputs), USB or Wi-Fi for host.

---

## Wiring

```
GPIO inputs ── DUT signals (3.3 V logic only)
Common GND with DUT
```

---

## Firmware Plan

1. GPIO snapshot @ timer ISR (max rate ~1 MHz practical)
2. Compress RLE encoding
3. [usb-cdc.md](../examples/usb-cdc.md) export Sigrok-compatible CSV
4. Optional [http-server.md](../examples/http-server.md) live view

---

## Testing

- [ ] Decode [I²C](../communication/i2c.md) START/STOP at 100 kHz
- [ ] Match Saleae reference on same signal

---

## Extensions

- SPI protocol decoder in firmware
- External level shifter for 5 V tolerance
