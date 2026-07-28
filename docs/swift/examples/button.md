# Example: Button Input

**Goal:** Input + debounce — hands-on Embedded Swift.

**Prerequisites:** See core lessons and [communication/gpio.md](../communication/gpio.md) if applicable.

**Protocol:** [communication/gpio.md](../communication/gpio.md)

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
struct Debouncer {
    var stable: Bool = true
    var lastChangeMs: UInt32 = 0

    mutating func update(raw: Bool, nowMs: UInt32) -> Bool? {
        if raw != stable && nowMs &- lastChangeMs >= 20 {
            stable = raw
            return stable
        }
        if raw != stable { lastChangeMs = nowMs }
        return nil
    }
}
```

### Ownership / ARC Notes

- Debounce state is a value type on the stack — no ARC traffic.
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
