# Project: USB Serial Converter

Bridge USB-CDC to UART ([RS-232](../communication/rs232.md) optional via MAX3232).

---

## BOM

ESP32-S3, MAX3232 (optional RS-232), USB-C.

---

## Wiring

```
USB ── ESP32-S3 internal PHY
GPIO43 TX ── RX external device
GPIO44 RX ── TX external device
Optional: MAX3232 between TTL and DB9
```

---

## Firmware Plan

1. [usb-cdc.md](../examples/usb-cdc.md) device init
2. [uart-echo.md](../examples/uart-echo.md) bridge CDC ↔ UART
3. Baud rate command `AT+BAUD=115200` via CDC

---

## Testing

- [ ] Loopback test PC → device → loopback pin → PC
- [ ] 921600 baud short cable

---

## Extensions

- Galvanic isolation (ADuM1201) for industrial

**Protocol:** [uart-usart.md](../communication/uart-usart.md)
