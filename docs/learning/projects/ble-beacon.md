# Project: BLE Beacon

Broadcast sensor data in advertising packets — no connection required.

---

## BOM

ESP32-S3 DevKitC-1, optional [BME280](../sensors/bme280.md).

---

## Wiring

Sensor on I²C if used; otherwise onboard temp (chip internal).

---

## Firmware Plan

1. [ble-server.md](../examples/ble-server.md) — configure advertising
2. Encode Eddystone-TLM or manufacturer data with temp/humidity
3. [deep-sleep.md](../examples/deep-sleep.md) between adv bursts

---

## Testing

- [ ] Visible in nRF Connect / LightBlue
- [ ] Battery life > 1 week on LiPo (with sleep)

---

## Extensions

- iBeacon UUID for asset tracking
- Combine with [battery-monitor.md](./battery-monitor.md)

**Protocol:** [ble.md](../communication/ble.md)
