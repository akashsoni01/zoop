# Project: Wi-Fi Sensor Node

Minimal telemetry: one sensor, Wi-Fi, MQTT/HTTP — based on [wifi-sensor](../examples/wifi-client.md) pattern.

---

## BOM

ESP32-S3, BME280 or [SHT](../sensors/temperature-humidity.md)-compatible sensor, USB power.

---

## Wiring

I²C: GPIO8/9 to sensor — see [weather-station.md](./weather-station.md).

---

## Firmware Plan

1. Sensor read task (Embassy) — [async-embassy.md](../examples/async-embassy.md)
2. [mqtt-client.md](../examples/mqtt-client.md) QoS 1 publish every 30 s
3. OTA — [ota-update.md](../examples/ota-update.md)

---

## Testing

- [ ] RSSI logged each publish
- [ ] QoS 1 retained message on broker restart

---

## Extensions

- TLS MQTT port 8883
- Batch readings in [file-system.md](../examples/file-system.md) when offline
