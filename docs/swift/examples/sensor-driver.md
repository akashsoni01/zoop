# Example: Sensor Driver

**Goal:** Register map pattern — hands-on Embedded Swift.

**Prerequisites:** See core lessons and [communication/i2c.md](../communication/i2c.md) if applicable.

**Protocol:** [communication/i2c.md](../communication/i2c.md)

---

## Wiring (ASCII)

```
ESP32-S3 DevKitC-1 — see board guide for default pins.

    GPIO48 ─── onboard LED (many DevKitC-1 boards)
    3V3 / GND ─── power and return
```

Adjust pins per your schematic and [boards/esp32-s3.md](../boards/esp32-s3.md).

---

## Swift Sketch

```swift
struct BME280 {
    var bus: I2C0
    let address: UInt8 = 0x76
    mutating func read() throws -> EnvironmentSample { /* ... */ }
}
```

### Ownership / ARC Notes

- Sensor holds I2C by value; caller owns bus lifecycle.
- Prefer **value types** (`struct`, `enum`) for drivers and state machines on embedded targets.
- Use **stack buffers** (`[UInt8]`, fixed arrays) instead of heap allocation in hot paths.
- If using **classes** (network stacks), watch for **retain cycles** in closures — use `[weak self]` or struct-based callbacks.

---

## Compile & Flash

```bash
# Embedded Swift project (ESP32-S3)
swift build -c release
espflash flash --monitor .build/release/App.bin
```

**Common build errors:**

| Error | Fix |
|-------|-----|
| Pin not found | Check board module / pin map |
| Linker error | Install embedded Swift toolchain + chip SDK |
| No output | Verify wiring and GPIO mapping |

---

## Extensions

- Combine with related examples in [README.md](./README.md).
- Promote to a full project in [projects/](../projects/README.md).

*Next: see [README.md](./README.md) for suggested learning order.*

---

*Back to [Embedded Swift](../README.md)*
