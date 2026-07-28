# Lesson 25 — Debugging: JTAG, GDB, Logging, Serial

**Prerequisites:** [04-cargo.md](./04-cargo.md), [08-gpio.md](./08-gpio.md)

**Board focus:** [ESP32-S3](./boards/esp32-s3.md) (USB-JTAG), [STM32](./boards/stm32.md) (SWD), [RP2040](./boards/rp2040.md).

**Maturity note:** Embedded Swift debugging **follows the same tools as C/Rust** on each platform — GDB, OpenOCD, probe-rs, serial monitors. Swift-specific: **symbol demangling**, **Embedded Swift `-g` debug info**, and **host Swift** test harnesses. No Swift equivalent of Rust `defmt` yet — use structured UART logging or ESP-IDF logging.

---

## Theory

Embedded debugging requires **designed-in observability**:

| Layer | Tool | Provides |
|-------|------|----------|
| **Logging** | UART `print`, ESP-IDF log | Runtime traces |
| **Interactive** | GDB + probe | Breakpoints, memory inspect |
| **Tracing** | GPIO toggles, logic analyzer | Timing proof |
| **Physical** | SWD/JTAG | Halt/step MCU |
| **Host companion** | macOS Swift serial app | HIL protocol testing |

### SWD vs JTAG

| Protocol | Wires | Use |
|----------|-------|-----|
| **SWD** | SWDIO, SWCLK, GND | ARM Cortex-M default |
| **JTAG** | TDI, TDO, TCK, TMS | ESP32 USB-JTAG, boundary scan |

### Embedded Swift logging

| Approach | Cost | Notes |
|----------|------|-------|
| `print` over UART | Medium | Works in ESP-IDF path |
| ESP-IDF `ESP_LOGx` | Low | Tag + level filtering |
| Binary log codes | Minimal | Host decoder script in Swift |
| GPIO bit-bang trace | Tiny | One pin = one event |

---

## Hardware Overview

### ESP32-S3 DevKitC-1

Built-in **USB Serial/JTAG** — single cable:

```
PC ── USB-C ──► ESP32-S3 (JTAG + CDC serial)
```

### STM32 Nucleo

Integrated **ST-Link** — SWD on same PCB.

### External probe (STM32)

```
ST-Link          Target
SWDIO ─────────► SWDIO
SWCLK ─────────► SWCLK
GND   ─────────► GND
```

---

## ASCII Wiring

ESP32-S3 single-cable debug — no extra wiring.

Optional **debug GPIO** for scope triggers:

```
GPIO47 ──► Logic analyzer CH0  (mark ISR entry)
GPIO48 ──► LED                 (visual heartbeat)
```

---

## Memory & Register Notes

Inspect variables in GDB:

```bash
xtensa-esp32s3-elf-gdb build/app.elf
(gdb) target remote :3333
(gdb) break main
(gdb) continue
(gdb) print bootCount
```

For **Embedded Swift** symbols, ensure `-g` and avoid aggressive strip at link time during development.

---

## HAL Swift Example — Structured Logging

```swift
import ESPIDF

enum LogLevel: UInt8 {
    case error = 0, warn, info, debug
}

func log(_ level: LogLevel, _ message: StaticString) {
    #if DEBUG
    let prefix: StaticString = switch level {
    case .error: "[E]"
    case .warn:  "[W]"
    case .info:  "[I]"
    case .debug: "[D]"
    }
    print("\(prefix) \(message)\r\n")
    #endif
}

@main
struct DebugApp {
    static func main() {
        log(.info, "Boot OK")
        let sensor = readSensor()
        log(.debug, "Sensor read complete")
        if sensor < 0 {
            log(.error, "Sensor fault")
        }
    }

    static func readSensor() -> Int { 42 }
}
```

---

## Bare-Metal Swift — GPIO Trace Macro

```swift
let tracePin = GPIO(47, mode: .output)

@inline(__always)
func tracePulse() {
    tracePin.set(high: true)
    tracePin.set(high: false)
}

@_cdecl("uart_isr")
func uartISR() {
    tracePulse() // visible on logic analyzer
    // ... handle UART
}
```

---

## Host Swift — Serial Debug Monitor (macOS)

```swift
import Foundation
import SwiftUI

@main
struct SerialMonitorApp: App {
    var body: some Scene {
        WindowGroup {
            SerialMonitorView(portPath: "/dev/cu.usbmodem101")
        }
    }
}

struct SerialMonitorView: View {
    @StateObject private var monitor: SerialMonitor
    init(portPath: String) {
        _monitor = StateObject(wrappedValue: SerialMonitor(path: portPath))
    }
    var body: some View {
        ScrollView {
            Text(monitor.log).font(.system(.body, design: .monospaced))
        }
    }
}
```

Filter log lines, send commands, parse binary log codes — complements on-device `print`.

---

## GDB Session (ESP32-S3)

```bash
# Terminal 1: OpenOCD
openocd -f board/esp32s3-builtin.cfg

# Terminal 2: GDB
xtensa-esp32s3-elf-gdb build/firmware.elf
(gdb) target remote :3333
(gdb) monitor reset halt
(gdb) break app.main
(gdb) continue
```

With **Embedded Swift + ESP-IDF**, use `idf.py gdb` wrapper.

---

## Common Mistakes

| Mistake | Symptom | Fix |
|---------|---------|-----|
| Optimized-out variables in GDB | `<optimized out>` | Lower optimization for debug builds |
| Print in tight loop | Timing changes behavior | Rate-limit logs |
| Wrong OpenOCD config | Cannot halt | Match chip revision |
| USB charge-only cable | No JTAG port | Data-capable cable |
| Breakpoint in flash XIP | Unreliable | Use IRAM attribute for debug functions |
| Semihosting accidentally linked | Hang on print | Disable semihosting |

---

## Debugging Tips

- **Binary search** with GPIO toggles — narrow which subsystem fails.
- **Watchdog:** Log last checkpoint before reset; store in RTC memory.
- Compare **debug vs release** behavior early — `-Osize` can hide bugs.
- Use **Address Sanitizer** on host port of logic — [26-testing.md](./26-testing.md).

---

## Performance Tips

- Gate verbose logs behind `#if DEBUG`.
- Use **numeric log codes** in release; decode on host.
- RTT (if available on platform) beats UART for high-rate traces.
- Remove trace GPIO in production or disable via compile flag.

---

## Exercises

1. **GDB breakpoint:** Halt at `main`; step through LED init.
2. **Log levels:** Implement filter — host shows only `[E]` and `[W]`.
3. **GPIO trace:** Measure ISR duration with logic analyzer on trace pin.
4. **Host monitor:** SwiftUI app displays live serial log with search.
5. **Post-mortem:** Store panic reason in RTC memory; print on next boot.

---

## References

- [ESP-IDF JTAG Debugging](https://docs.espressif.com/projects/esp-idf/en/latest/esp32s3/api-guides/jtag-debugging/index.html)
- [probe-rs book](https://probe.rs/docs/)
- [OpenOCD ESP32 targets](https://docs.espressif.com/projects/esp-idf/en/latest/esp32s3/api-guides/jtag-debugging/using-debugger.html)
- [26-testing.md](./26-testing.md)

---

*Previous: [24-low-power.md](./24-low-power.md) · Next: [26-testing.md](./26-testing.md)*
