# Project: Home Automation Hub

Build **Multi-protocol** on **ESP32-S3** using Embedded Swift.

**Architecture:** ESP32-S3 ↔ sensors/actuators ↔ network (Wi-Fi / BLE / USB as needed).

**Prerequisites:** Core lessons, relevant [communication/](../communication/README.md) guides, and [examples/](../examples/README.md).

---

## Learning Objectives

- Integrate multiple peripherals in one firmware image
- Apply Swift value-type drivers and careful ARC boundaries
- Validate with hardware tests and soak runs

---

## BOM

| Part | Qty | Notes |
|------|-----|-------|
| ESP32-S3 DevKitC-1 | 1 | USB-C data cable |
| Relays + sensors | 1 | See [sensors/](../sensors/README.md) |
| Various | 1 | See [displays/](../displays/README.md) if applicable |
| Breadboard + wires | 1 | |
| Optional power | 1 | Battery pack for portable builds |

---

## Wiring (ASCII)

```
ESP32-S3 DevKitC-1          Peripherals
─────────────────          ───────────
Mixed GPIO/I²C
3V3  ──────────────────── VCC
GND  ──────────────────── GND
```

Verify pin map in [boards/esp32-s3.md](../boards/esp32-s3.md).

---

## Firmware Plan

| Milestone | Goal |
|-----------|------|
| M1: MQTT hub
| M2: rules engine
| M3: web config |

### Suggested Main Loop (Swift)

```swift
// Illustrative — adapt to your drivers
while true {
    let sample = try sensor.read()
    display.update(sample)
    try await network.publish(sample)
    sleepSeconds(sampleInterval)
}
```

**ARC note:** Keep `sensor`, `display`, and `network` as owned structs in `main` — avoid singleton classes retaining each other.

---

## Testing

- [ ] Power-up: no brownout, USB enumerates
- [ ] Each milestone passes before proceeding
- [ ] 1 h soak without watchdog reset
- [ ] Measured current meets budget (if battery-powered)

### Debugging Tips

| Symptom | Likely Cause | Fix |
|---------|--------------|-----|
| No I²C devices | Wiring / address | Run [i2c-scanner](../examples/i2c-scanner.md) |
| Wi-Fi OK, no cloud | Credentials / TLS | Test with LAN broker first |
| USB not seen | Wrong USB port / driver | Use native USB pins on S3 |

---

## Extensions

- Add [deep-sleep](../examples/deep-sleep.md) between samples
- Expose Web UI via [http-server](../examples/http-server.md)
- Log to SD via [data-logger](./data-logger.md) patterns

---

*Related: [examples/](../examples/README.md) · [communication/](../communication/README.md)*

---

*Back to [Embedded Swift](../README.md)*
