# Project: Environmental Monitor

Multi-sensor air quality station: [BME280](../sensors/bme280.md), [gas sensors](../sensors/gas-sensors.md), OLED, Modbus/ MQTT output.

---

## BOM

ESP32-S3, BME280, MQ-135 (ADC), SSD1306, optional RS-485 module.

---

## Wiring

```
I²C sensors ── GPIO8/9
MQ-135 AO ── GPIO6 (ADC)
RS-485 ── GPIO15/16/17 per rs485.md
```

---

## Firmware Plan

1. [i2c-scanner.md](../examples/i2c-scanner.md)
2. Gas sensor warmup timer 24 h first use
3. [oled.md](../examples/oled.md) + [mqtt-client.md](../examples/mqtt-client.md)
4. [Modbus](../communication/modbus.md) slave registers for SCADA

---

## Testing

- [ ] BME280 vs reference hygrometer ±3% RH
- [ ] Modbus read matches OLED

---

## Extensions

- [LoRa](../communication/lora.md) for outdoor nodes
