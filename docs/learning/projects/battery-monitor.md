# Project: Battery Monitor

Monitor LiPo voltage via [ADC](../examples/adc.md), report over [BLE](../communication/ble.md).

---

## BOM

ESP32-S3, voltage divider (100 kΩ / 100 kΩ), JST LiPo connector.

---

## Wiring

```
Battery+ ── 100k ── GPIO6 (ADC) ── 100k ── GND
```

Use `attenuation` for 0–3.3 V range; scale to 0–4.2 V with divider ratio.

---

## Firmware Plan

1. Calibrated ADC read — [adc.md](../examples/adc.md)
2. Coulomb estimate (optional current sensor)
3. [ble-server.md](../examples/ble-server.md) Battery Service 0x180F
4. [deep-sleep.md](../examples/deep-sleep.md) between samples

---

## Testing

- [ ] Voltage within ±50 mV of multimeter
- [ ] Low-battery alert < 3.5 V

---

## Extensions

- [MQTT](../communication/mqtt.md) for fixed installations
