# GPIO — General Purpose Input/Output

**Prerequisites:** [08-gpio.md](../08-gpio.md), [07-hal.md](../07-hal.md)

---

## Theory

Each GPIO pin connects to a **pad** on the silicon, routed through an **I/O matrix** (ESP32-S3) or **alternate-function mux** (STM32). The CPU writes to **configuration registers** (direction, pull-up/down, drive strength) and **data registers** (set/clear/toggle).

| Mode | Behavior |
|------|----------|
| Output push-pull | Actively drives HIGH or LOW |
| Input floating | High impedance; undefined without external bias |
| Input pull-up | Weak internal resistor to VDD (~45 kΩ on ESP32-S3) |
| Input pull-down | Weak internal resistor to GND |
| Open-drain output | Can only pull LOW; HIGH requires external pull-up |

**Active-low** wiring is common: LED cathode to GPIO, anode through resistor to 3V3 — logic `0` turns the LED on.

---

## Timing Diagram (ASCII)

Output toggle at 1 kHz (software delay):

```
        ┌───┐   ┌───┐   ┌───┐
GPIO    │   │   │   │   │   │
    ────┘   └───┘   └───┘   └───
        |<->|
         1 ms period (500 µs high, 500 µs low)
```

Input with debouncing (button, ~20 ms filter):

```
Button (raw)  ──┐     ┌─┐     ┌──────────
                └─────┘ └─────┘
                     bounce

Debounced     ────────────────────────────
              (stable LOW after 20 ms)
```

---

## Packet Format

GPIO has no packets. Treat each pin as **1 bit** of state:

| Field | Size | Meaning |
|-------|------|---------|
| Level | 1 bit | 0 = LOW, 1 = HIGH |
| Direction | 1 bit | 0 = input, 1 = output |

Multi-pin **parallel GPIO** (e.g., 8-bit LCD data bus) sends one "byte" per write cycle — see [examples/lcd.md](../examples/lcd.md).

---

## Electrical Characteristics

| Parameter | ESP32-S3 (typical) | Notes |
|-----------|-------------------|-------|
| VIH | 0.75 × VDD | Input read as HIGH |
| VIL | 0.25 × VDD | Input read as LOW |
| VOH | ~3.3 V @ 20 mA | Depends on drive strength |
| Max source/sink | 40 mA per pin (abs max) | Stay ≤ 12 mA for reliability |
| Pull-up | ~45 kΩ internal | Optional |

**Always** use a current-limiting resistor with LEDs (220 Ω–1 kΩ).

---

## Swift HAL Sketch

```swift
import Embedded
import ESP32Hardware

// Value type owns pin config — no ARC on hot path.
struct LED {
    var pin: GPIO.Output<GPIO48>

    mutating func toggle() {
        pin.toggle()
    }
}

func blink(led: inout LED, delayMs: UInt32) -> Never {
    while true {
        led.toggle()
        Delay.milliseconds(delayMs)
    }
}

func readButton(_ btn: GPIO.Input<GPIO0>) -> Bool {
    btn.isLow  // active-low with internal pull-up
}
```

**Compile notes (ESP32-S3 Embedded Swift):**

```bash
swift build --triple riscv32-none-none-eabi  # or chip-specific triple
esptool.py write_flash 0x0 .build/release/App.bin
```

**ARC note:** Prefer `struct` wrappers for peripherals. Avoid `class` in ISRs — if you must share state, use `Unmanaged` or a lock-free ring buffer owned by one task.

---

## Bare-Metal Sketch (Register-Level)

```swift
// Illustrative register-level access — addresses from ESP32-S3 TRM
let gpioOut = MMIO<UInt32>(0x6000_4004)
let gpioEnable = MMIO<UInt32>(0x6000_4020)
let ledBit: UInt32 = 1 << 2

func initLedPin() {
    gpioEnable.modify { $0 |= ledBit }
}

func setLed(on: Bool) {
    gpioOut.modify { on ? ($0 | ledBit) : ($0 & ~ledBit) }
}
```

---

## Example Projects

- [smart-switch.md](../projects/smart-switch.md) — relay + button
- [logic-analyzer.md](../projects/logic-analyzer.md) — GPIO capture

---

## Common Mistakes

| Mistake | Consequence | Fix |
|---------|-------------|-----|
| No current-limiting resistor on LED | Damaged pin / LED | Use 330 Ω–1 kΩ |
| Floating input | Random reads | Enable pull-up/down |
| 5 V on ESP32 RX/inputs | Permanent damage | Level shift to 3.3 V |
| Strapping pins pulled wrong at reset | Boot failure | Check GPIO0/45/46 at power-on |

---

## Exercises

1. Blink onboard LED at 2 Hz using a `struct` that encapsulates the pin.
2. Read a button with 20 ms debounce — use a value type for state, not a class.
3. Drive 4 LEDs as a binary counter using `GPIO.Output` array.

---

## References

- [ESP32-S3 GPIO TRM](https://www.espressif.com/sites/default/files/documentation/esp32-s3_technical_reference_manual_en.pdf)
- [Embedded Swift documentation](https://github.com/apple/swift-evolution/blob/main/proposals/0413-embedded-swift.md)

---

*Back to [Embedded Swift](../README.md)*
