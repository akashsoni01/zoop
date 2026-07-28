# Project: OLED Dashboard

Local [OLED](../examples/oled.md) UI + [HTTP](../examples/http-server.md) mirror for sensor data.

---

## BOM

ESP32-S3, SSD1306 OLED, BME280, optional rotary encoder.

---

## Wiring

Shared I²C bus — see [weather-station.md](./weather-station.md).

---

## Firmware Plan

1. OLED pages: overview, graphs (simple bar), settings
2. [encoder.md](../examples/encoder.md) page navigation
3. [http-server.md](../examples/http-server.md) `/api/live` JSON
4. Optional [websockets.md](../communication/websockets.md) push

---

## Testing

- [ ] OLED refresh 5 Hz without flicker
- [ ] HTTP API matches OLED values

---

## Extensions

- [displays/README.md](../displays/README.md) upgrade to color TFT
