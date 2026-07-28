# Project: IoT Gateway

Bridge [LoRa](../communication/lora.md) or [Modbus](../communication/modbus.md) to [MQTT](../communication/mqtt.md)/Ethernet.

---

## BOM

ESP32-S3, SX1262 LoRa module (SPI), optional W5500 Ethernet, RS-485 transceiver.

---

## Wiring

```
LoRa: GPIO11 MOSI, GPIO12 SCK, GPIO13 MISO, GPIO10 CS, GPIO9 RST, GPIO8 BUSY
RS-485: GPIO15 TX, GPIO16 RX, GPIO17 DE
```

---

## Firmware Plan

1. [spi.md](../communication/spi.md) LoRa RX loop
2. Parse payload → normalize JSON
3. [wifi-client](../examples/wifi-client.md) MQTT publish upstream
4. Downlink: MQTT subscribe → LoRa TX

---

## Testing

- [ ] End-to-end latency < 2 s (LoRa SF7)
- [ ] Modbus read → MQTT within 500 ms

---

## Extensions

- [HTTP](../communication/http.md) admin UI — [http-server.md](../examples/http-server.md)
- Dual radio: LoRa + [Zigbee](../communication/zigbee.md) UART coprocessor
