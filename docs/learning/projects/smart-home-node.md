# Project: Smart Home Node

**Architecture:** Configurable MQTT/BLE node with NVS credentials, relay output, and button input.

---

## BOM

| Part | Qty |
|------|-----|
| ESP32-S3 DevKitC-1 | 1 |
| 5V relay module (low-level trigger) | 1 |
| Tactile button | 1 |
| 1 kΩ resistor (relay GPIO) | 1 |

---

## Wiring

```
GPIO4  ── button ── GND (internal pull-up)
GPIO5  ── relay IN
Relay COM/NO ── load (lamp — mains only with qualified enclosure)
```

---

## Firmware Plan

1. [button.md](../examples/button.md) + [eeprom.md](../examples/eeprom.md) — store SSID
2. [wifi-client](../examples/wifi-client.md) + [mqtt-client](../examples/mqtt-client.md)
3. Subscribe `home/node1/set` → toggle relay
4. Optional: [ble-server](../examples/ble-server.md) for provisioning

---

## Testing

- [ ] MQTT command toggles relay within 200 ms
- [ ] Survives AP reboot (reconnect)
- [ ] Button local override works

---

## Extensions

- [Zigbee](../communication/zigbee.md) coordinator bridge
- [Home Assistant](https://www.home-assistant.io/) autodiscovery JSON

**Lessons:** [08-gpio.md](../08-gpio.md), [20-wifi.md](../20-wifi.md)
