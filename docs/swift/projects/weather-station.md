# Project: Weather Station

Build a battery-friendly outdoor/indoor station on **ESP32-S3**: read environment sensors over **I²C**, show live values on an OLED, and publish JSON over **MQTT**.

**Architecture:** ESP32-S3 ←I²C→ [BME280](../sensors/bme280.md) + [SSD1306](../displays/ssd1306.md) → Wi-Fi → MQTT broker.

**Prerequisites:** [17-i2c.md](../17-i2c.md), [20-wifi.md](../20-wifi.md), [examples/i2c-scanner.md](../examples/i2c-scanner.md).

---

## Learning Objectives

- Combine sensor + display + network in one firmware
- Own shared I²C bus safely (sequential transactions, no concurrent access)
- Design a sample → display → publish → sleep loop
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
| **M5** | Deep sleep between samples | [deep-sleep](../examples/deep-sleep.md), [24-low-power.md](../24-low-power.md) |

### Suggested Main Loop (Swift)

```swift
/// Illustrative control flow — adapt to your network stack.
/// Ownership: one I2C bus owner; sensors and display borrow it sequentially.

while true {
    let sample = try bme.read()           // mutates &i2c briefly
    try oled.showEnv(sample)              // same bus, different address
    try await mqtt.publish("weather/out", sample.toJSON())
    sleepSeconds(60)
}
```

**JSON payload example:**

```json
{"t_c":23.4,"rh":48.1,"p_hpa":1013.2,"vbat":3.91}
```

**ARC note:** Keep `bme`, `oled`, and `mqtt` as structs owned by `main` — pass `&mut i2c` sequentially rather than sharing a bus-holding class across tasks.

---

## Testing

- [ ] I²C scan finds both addresses
- [ ] OLED updates every 2 s (active mode) with plausible values
- [ ] MQTT topic `weather/out` receives JSON (`mosquitto_sub` / broker UI)
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

## Extensions

- Add [http-server](../examples/http-server.md) for local dashboard
- Log to SD via [data-logger](./data-logger.md)
- Add [battery-monitor](./battery-monitor.md) ADC channel to JSON

---

*Related: [examples/](../examples/README.md) · [communication/](../communication/README.md)*

*Back to [Embedded Swift](../README.md)*
