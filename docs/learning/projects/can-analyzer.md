# Project: CAN Bus Analyzer

Sniff [CAN](../communication/can.md) frames via ESP32-S3 TWAI, log to USB serial or Wi-Fi.

---

## BOM

ESP32-S3, SN65HVD230 transceiver, 120 Ω resistor, CAN bus connector.

---

## Wiring

```
ESP32 TWAI TX ── transceiver TXD
ESP32 TWAI RX ── transceiver RXD
CAN_H, CAN_L ── bus (120 Ω termination at ends)
```

---

## Firmware Plan

1. TWAI init 500 kbit/s — [can.md](../communication/can.md)
2. RX interrupt → ring buffer
3. Format CSV: `timestamp,id,dlc,data`
4. [usb-cdc.md](../examples/usb-cdc.md) or [wifi-client](../examples/wifi-client.md) stream

---

## Testing

- [ ] Decode OBD-II PID request/response on vehicle (parked, safe)
- [ ] No frame loss @ 50% bus load (bench generator)

---

## Extensions

- DBC decode in Rust
- [HTTP](../communication/http.md) live view

**Lesson:** [18-can.md](../18-can.md)
