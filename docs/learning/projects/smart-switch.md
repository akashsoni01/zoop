# Project: Smart Switch

Wall-switch form factor: button + relay + MQTT — compact [smart-home-node](./smart-home-node.md).

---

## BOM

ESP32-S3 mini module, relay, button, 3D printed enclosure (mains work: qualified electrician).

---

## Wiring

```
GPIO4 ── touch/button
GPIO5 ── relay (low-level)
```

---

## Firmware Plan

1. [button.md](../examples/button.md) debounced toggle
2. [mqtt-client.md](../examples/mqtt-client.md) state sync
3. [eeprom.md](../examples/eeprom.md) last state on boot
4. OTA — [ota-update.md](../examples/ota-update.md)

---

## Testing

- [ ] Local toggle < 50 ms feel
- [ ] MQTT retained message restores state after power loss

---

## Extensions

- Matter over Wi-Fi (future esp-matter stack)
