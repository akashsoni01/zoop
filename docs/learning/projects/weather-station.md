# Project: Weather Station

Build a battery-friendly outdoor/indoor station on **ESP32-S3**: read environment sensors over **I²C** (Inter-Integrated Circuit), show live values on an OLED, and publish JSON over **MQTT** (Message Queuing Telemetry Transport).

**Architecture:** ESP32-S3 ←I²C→ [BME280](../sensors/bme280.md) + [SSD1306](../displays/ssd1306.md) → Wi-Fi → MQTT broker.

**Prerequisites:** [17-i2c.md](../17-i2c.md), [20-wifi.md](../20-wifi.md), [examples/i2c-scanner.md](../examples/i2c-scanner.md).

---

## Learning Objectives

- Combine sensor + display + network in one firmware
- Own shared I²C bus safely (no concurrent transactions)
- Design a simple sample → display → publish → sleep loop
- Validate with soak tests and broker logs

---

## BOM

| Part | Qty | Notes |
|------|-----|-------|
| ESP32-S3 DevKitC-1 | 1 | USB-C data cable |
| BME280 module (I²C) | 1 | Address usually `0x76` or `0x77` |
| SSD1306 OLED 128×64 | 1 | Address usually `0x3C` |
| Breadboard + wires | 1 | Shared SDA/SCL |
| Optional 18650 + charger | 1 | For field deploy |

---

## Wiring (ASCII)

```
ESP32-S3 DevKitC-1          BME280              SSD1306
─────────────────          ──────              ───────
GPIO8  (SDA) ────────────── SDA ─────────────── SDA
GPIO9  (SCL) ────────────── SCL ─────────────── SCL
3V3    ──────────────────── VCC ─────────────── VDD
GND    ──────────────────── GND ─────────────── GND

         4.7 kΩ pull-ups to 3V3 on SDA and SCL
         (many modules already include them)
```

---

## Firmware Plan

| Milestone | Goal | Link |
|-----------|------|------|
| **M1** | Confirm both devices on the bus | [i2c-scanner](../examples/i2c-scanner.md) → expect `0x76`/`0x77` and `0x3C` |
| **M2** | Read compensated T/H/P | [sensor-driver](../examples/sensor-driver.md), [bme280](../sensors/bme280.md) |
| **M3** | Render values on OLED | [oled](../examples/oled.md), [ssd1306](../displays/ssd1306.md) |
| **M4** | Join Wi-Fi + publish MQTT | [wifi-client](../examples/wifi-client.md), [mqtt-client](../examples/mqtt-client.md) |
| **M5** | Deep sleep between samples | [deep-sleep](../examples/deep-sleep.md), [24-low-power](../24-low-power.md) |

### Suggested Main Loop (sketch)

```rust
//! Illustrative control flow — adapt to esp-idf-svc or embassy-net.
//! Ownership: one I2c bus owner; sensors and display borrow it sequentially.

loop {
    let sample = bme.read()?;           // borrows &mut i2c briefly
    oled.show_env(&sample)?;            // same bus, different address
    mqtt.publish("weather/out", &sample.to_json())?;
    // Enter deep sleep; RTC timer wakes after N seconds
    deep_sleep_ms(60_000);
}
```

**JSON payload example:**

```json
{"t_c":23.4,"rh":48.1,"p_hpa":1013.2,"vbat":3.91}
```

---

## Testing

- [ ] I²C scan finds both addresses
- [ ] OLED updates every 2 s (active mode) with plausible values
- [ ] MQTT topic `weather/out` receives JSON (mosquitto_sub / broker UI)
- [ ] After enabling sleep: average current drops vs busy loop
- [ ] 24 h soak without watchdog reset

### Debugging Tips

| Symptom | Likely Cause | Fix |
|---------|--------------|-----|
| No I²C devices | Wiring / 5 V module on 3V3 MCU | Check GND common; level shift if needed |
| OLED garbage | Wrong geometry / I²C addr | Try `0x3C`/`0x3D`; SH1106 needs column offset |
| Wi-Fi OK, no MQTT | Broker ACL / TLS mismatch | Start with unencrypted LAN broker |
| Brownouts outdoors | Long USB cable / weak PSU | Use short cable or regulated 5 V |

---

## Performance Tips

- Burst-read BME280 once per wake; do not poll at 100 Hz outdoors
- Use MQTT QoS 0 or 1; avoid large retained payloads
- Keep OLED off during sleep (power gate VDD if designing a PCB)

---

## Extensions

- Dual sensor: add [BMP280](../sensors/bmp280.md) for pressure redundancy
- Off-grid uplink: [LoRa](../communication/lora.md)
- Local UI: [HTTP server](../examples/http-server.md) dashboard
- Async rewrite: [Embassy](../22-embassy.md) + `embassy-net`

**Related lessons:** [17-i2c.md](../17-i2c.md) · [20-wifi.md](../20-wifi.md) · [27-design-patterns.md](../27-design-patterns.md)

**Related board:** [boards/esp32-s3.md](../boards/esp32-s3.md)
