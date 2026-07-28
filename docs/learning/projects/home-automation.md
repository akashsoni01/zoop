# Project: Home Automation Hub

Multi-protocol: MQTT, BLE, optional Zigbee UART — central [smart-home-node](./smart-home-node.md) scaled up.

---

## BOM

ESP32-S3, relay bank (4 ch), Ethernet W5500 optional, CC2652 Zigbee stick optional.

---

## Firmware Plan

1. Embassy multi-task — [async-embassy.md](../examples/async-embassy.md)
2. MQTT broker bridge (local Mosquitto + device topics)
3. Rules engine: `if temp > 28 → relay ON`
4. Web UI — [http-server.md](../examples/http-server.md)

---

## Testing

- [ ] Rule triggers within 1 s of sensor update
- [ ] 4 relays independent

---

## Extensions

- [WebSockets](../communication/websockets.md) live dashboard
- Voice assistant webhook via [HTTP](../communication/http.md)
