# AVR Board Guide (ATmega328P)

**8-bit AVR** — Arduino Uno / Nano form factor. **Not supported** for Embedded Swift today.

---

## Overview

| Spec | Arduino Uno (ATmega328P) |
|------|--------------------------|
| **CPU** | 8-bit AVR @ 16 MHz |
| **Flash** | 32 KB |
| **SRAM** | 2 KB |
| **EEPROM** | 1 KB |

---

## Embedded Swift Maturity

| Path | Status |
|------|--------|
| Embedded Swift | **Not viable** — 2 KB RAM insufficient for Swift runtime/subset |
| Arduino C/C++ | Standard approach |
| Host Swift | Use macOS/iOS to talk to Uno over **USB serial** if running serial firmware |

Honest recommendation: keep AVR firmware in **C/C++**; use Swift on the host for UI, logging, and test harnesses — [15-uart.md](../15-uart.md), [26-testing.md](../26-testing.md).

---

## Memory Map Overview

```
32 KB flash
2 KB SRAM
1 KB EEPROM
Memory-mapped I/O registers
```

---

## GPIO Notes

Arduino pin numbers ≠ AVR port bits — use schematic when wiring. 5 V logic on Uno — level-shift to 3.3 V ESP32 if interfacing.

---

## If You Must Experiment

Community Embedded Swift targets start at **32 KB+ flash / 8 KB+ RAM** (e.g., STM32C011). AVR is below practical Swift minimums as of 2026.

---

## Host Swift Serial Bridge

```swift
// Talk to Arduino running C firmware
let port = ORSSerialPort(path: "/dev/cu.usbserial-1410")!
port.baudRate = 9600
port.open()
port.sendData("READ_SENSOR\n".data(using: .utf8)!)
```

---

*Supported alternative: [esp32-c3.md](./esp32-c3.md) at similar price point*
