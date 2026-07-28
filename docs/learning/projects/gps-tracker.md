# Project: GPS Tracker

Parse NMEA from [GPS module](../sensors/gps.md), log track, optional LTE/Wi-Fi upload.

---

## BOM

ESP32-S3, NEO-6M/NEO-M8N GPS (UART), microSD, LiPo.

---

## Wiring

```
GPS TX ── ESP32 RX (GPIO44)
GPS RX ── ESP32 TX (GPIO43) optional
GPS VCC 3V3, GND
```

---

## Firmware Plan

1. [uart-echo.md](../examples/uart-echo.md) adapted — parse `$GPGGA`, `$GPRMC`
2. Store fixes in [file-system.md](../examples/file-system.md)
3. [wifi-client.md](../examples/wifi-client.md) batch upload [MQTT](../communication/mqtt.md)

---

## Testing

- [ ] Fix outdoors < 2 min cold start
- [ ] Logged coordinates match phone GPS ±10 m

---

## Extensions

- Geofence alerts
- [BLE](../communication/ble.md) broadcast last fix for phone app
