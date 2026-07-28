# Lesson 02: Embedded Swift (Freestanding Mode)

**Embedded Swift** is Apple's subset of Swift designed to run on microcontrollers **without** an operating system, Objective-C runtime, or Foundation framework. It is the Swift equivalent of Rust's `no_std` — sometimes called **freestanding** mode.

**Prerequisites:** [01-swift-basics.md](./01-swift-basics.md)  
**Next:** [03-memory-layout.md](./03-memory-layout.md)  
**See also:** [04-swiftpm.md](./04-swiftpm.md), [glossary.md](./glossary.md)

---

## Theory

### What Is Freestanding Swift?

Freestanding Swift compiles firmware that:

- Boots directly from reset vector (no OS)
- Links against a minimal runtime (or none)
- Accesses hardware via MMIO (Memory-Mapped I/O)
- Avoids dynamic allocation unless you explicitly enable it

```
┌─────────────────────────────────────────┐
│  Application (your Swift code)          │
├─────────────────────────────────────────┤
│  HAL / BSP (board-specific Swift/C)     │
├─────────────────────────────────────────┤
│  Startup / linker script (assembly)     │
├─────────────────────────────────────────┤
│  Hardware (MCU)                         │
└─────────────────────────────────────────┘
```

### What You Keep vs Lose

| Feature | Full Swift (host) | Embedded Swift (freestanding) |
|---------|-------------------|-------------------------------|
| Structs, enums, protocols | ✅ | ✅ |
| Generics | ✅ | ✅ (with limits) |
| Optionals, Result | ✅ | ✅ |
| `String`, `Array` (heap) | ✅ | ❌ or limited |
| Foundation | ✅ | ❌ |
| ARC (Automatic Reference Counting) | ✅ for classes | ❌ / minimal |
| Swift Concurrency | ✅ full runtime | ⚠️ subset / custom executor |
| Reflection, mirrors | ✅ | ❌ |
| Objective-C interop | ✅ | ❌ |
| Dynamic casting (`as?`) | ✅ | Limited |
| `@main` entry | ✅ | ✅ (with embedded attribute) |
| `print` to console | ✅ | ❌ — use UART |

### ARC and Ownership on Embedded

On iOS, **ARC** automatically manages class lifetimes. On MCU firmware:

- **Prefer structs** — no reference counting at all
- **Classes are often unavailable** or require custom runtime support
- **Static allocation** — global and static variables live for program lifetime
- **`UnsafePointer`** — explicit memory ownership for MMIO and buffers

```swift
// Embedded-friendly: all stack or static, no ARC
struct TimerState {
    var tickCount: UInt32
    var periodMs: UInt32
}

// Static buffer — lives in .bss, zero-initialized at boot
var uartBuffer: (UInt8, UInt8, UInt8, UInt8, UInt8,
                 UInt8, UInt8, UInt8, UInt8, UInt8,
                 UInt8, UInt8, UInt8, UInt8, UInt8,
                 UInt8, UInt8, UInt8, UInt8, UInt8,
                 UInt8, UInt8, UInt8, UInt8, UInt8,
                 UInt8, UInt8, UInt8, UInt8, UInt8,
                 UInt8, UInt8, UInt8, UInt8, UInt8,
                 UInt8, UInt8, UInt8, UInt8, UInt8,
                 UInt8, UInt8, UInt8, UInt8, UInt8,
                 UInt8, UInt8, UInt8, UInt8, UInt8,
                 UInt8, UInt8, UInt8, UInt8, UInt8,
                 UInt8, UInt8, UInt8, UInt8, UInt8,
                 UInt8, UInt8, UInt8, UInt8) = (0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0)
```

Use fixed-size tuples or `UnsafeMutableBufferPointer` to fixed storage instead of `[UInt8]` when heap is disabled.

### Entry Point and Panic Handling

Embedded programs never return from `main`. The entry point initializes hardware and enters an infinite loop:

```swift
@main
struct App {
    static func main() {
        hardwareInit()
        log("Embedded Swift started")

        while true {
            blinkTask()
        }
    }
}
```

When something impossible happens, call a **panic handler** instead of unwinding:

```swift
func fatalError(_ message: StaticString) -> Never {
    log("FATAL: ")
    log(message)
    // Disable interrupts, blink SOS, or halt
    while true { }
}
```

There is no stack unwinding on most embedded targets — panics are **abort-only**.

### Maturity Honesty (2025–2026)

| Target | Status |
|--------|--------|
| ESP32-S3 | Community projects; expect API churn |
| STM32 (Cortex-M) | Experimental Swift ports |
| RP2040 | Research / hobby projects |
| Apple Silicon (host tests) | Fully supported — use for unit tests |

Check [swift-embedded-examples](https://github.com/apple/swift-embedded-examples) for current supported boards before buying hardware.

---

## Hardware Overview

For your first freestanding project, use **ESP32-S3 DevKitC-1**:

| Feature | Detail |
|---------|--------|
| CPU | Dual Xtensa LX7 @ 240 MHz |
| USB | Built-in USB-JTAG + serial |
| Boot | ROM bootloader + external flash |
| LED | On-board (GPIO varies by revision) |

Alternative: **STM32 Nucleo-F411RE** for ARM-centric Embedded Swift experiments.

---

## Wiring Diagram

No external wiring for the first freestanding flash — use the on-board LED.

```
ESP32-S3 DevKit (minimal first flash)
┌─────────────────┐
│  USB-C ─────────┼──► Power + programming + serial monitor
│  On-board LED   │    (GPIO2 or WS2812 depending on board)
│  BOOT / RESET   │
└─────────────────┘
```

External wiring begins in [08-gpio.md](./08-gpio.md).

---

## Memory & Register Explanation

Freestanding Swift produces ELF binaries with standard sections:

| Section | Content | Swift examples |
|---------|---------|----------------|
| `.text` | Machine code | Compiled functions |
| `.rodata` | Constants | `StaticString` literals |
| `.data` | Initialized statics | `var counter = 0` at global scope |
| `.bss` | Zero-init statics | `var buffer: [UInt8; N]` static |
| `.stack` | Function locals | Struct instances on stack |

**No heap section** unless you link a custom allocator — keep it that way initially.

Reset sequence:

1. CPU reads reset vector from flash
2. Startup code copies `.data`, zeroes `.bss`
3. Calls Swift runtime init (minimal)
4. Calls `main`

---

## HAL-Style Swift Implementation

Minimal freestanding app structure using HAL:

```swift
import ESP32HAL   // Hypothetical board package

@main
struct BlinkApp {
    static func main() {
        let board = ESP32S3Board()
        var led = board.pin2.asOutput(initial: .low)

        var delay = MillisecondDelay()

        while true {
            _ = led.setHigh()
            delay.wait(500)
            _ = led.setLow()
            delay.wait(500)
        }
    }
}
```

The HAL hides clock init, GPIO mux, and pin configuration. Your application uses protocols from [07-hal.md](./07-hal.md).

---

## Bare-Metal / MMIO Swift Notes

Without any HAL — direct register access:

```swift
// ESP32-S3 GPIO register addresses (simplified)
enum GPIORegister {
    static let base: UInt = 0x6000_4000
    static let outSet: UInt = base + 0x10   // GPIO_OUT_W1TS
    static let outClear: UInt = base + 0x14 // GPIO_OUT_W1TC
}

@inline(__always)
func regWrite(_ address: UInt, _ value: UInt32) {
    UnsafeMutablePointer<UInt32>(bitPattern: address)!.pointee = value
}

func blinkBareMetal() {
    let pinMask: UInt32 = 1 << 2  // GPIO2

    while true {
        regWrite(GPIORegister.outSet, pinMask)
        busyWait(cycles: 24_000_000)  // ~500ms at 48MHz example
        regWrite(GPIORegister.outClear, pinMask)
        busyWait(cycles: 24_000_000)
    }
}

func busyWait(cycles: UInt32) {
    var remaining = cycles
    while remaining > 0 {
        remaining &-= 1
    }
}
```

All hardware access is explicit. The compiler optimizes the loop if not marked volatile.

---

## Step-by-Step Explanation

### Step 1: Install Toolchain

Install Swift from Xcode or swift.org. Obtain Embedded Swift-capable toolchain per project README.

### Step 2: Clone Example Project

```bash
git clone https://github.com/apple/swift-embedded-examples.git
cd swift-embedded-examples
# Follow board-specific subdirectory instructions
```

### Step 3: Build for Target Triple

```bash
swift build -c release \
  --triple arm-none-eabi   # or xtensa-esp32s3-none-elf
```

Exact flags vary — see [04-swiftpm.md](./04-swiftpm.md).

### Step 4: Flash Binary

Use esptool (ESP32), OpenOCD (STM32), or picotool (RP2040).

### Step 5: Open Serial Monitor

```bash
# macOS example
screen /dev/cu.usbmodem* 115200
```

### Step 6: Verify Boot Message

You should see your `log("...")` output or observe LED blink.

### Step 7: Strip Host Dependencies

If porting host code, remove `import Foundation`, `String`, and class-based APIs one at a time until it links.

---

## Common Mistakes

| Mistake | Symptom | Fix |
|---------|---------|-----|
| `import Foundation` in firmware | Link error | Remove; use freestanding types |
| Using `String` in loops | Crash or link failure | Use `StaticString`, `[UInt8]` |
| Class-based architecture | ARC link errors | Refactor to structs |
| Missing startup file | Crash at boot | Include board startup.s / linker script |
| Wrong target triple | Illegal instructions | Match board CPU exactly |
| Assuming latest stable Swift works | Build failure | Use Embedded Swift nightly if required |

---

## Debugging Tips

1. **Link map inspection** — find unexpected sections bloating flash.
2. **Semihosting** — ARM debug prints via OpenOCD (development only).
3. **Minimal repro** — strip to blink-only when debugging boot issues.
4. **Compare with C example** — verify hardware works before blaming Swift.
5. **Host unit tests** — test algorithms on macOS; flash only hardware glue.

---

## Performance Tips

| Tip | Detail |
|-----|--------|
| `-Osize` for release | Minimize flash usage |
| Avoid protocol existentials | Static dispatch via generics |
| Inline small MMIO helpers | `@inline(__always)` on register accessors |
| No heap = deterministic timing | Good for real-time control loops |
| Place hot code in IRAM (ESP32) | Linker section attributes when supported |

---

## Exercises

### Exercise 1: Host vs Embedded Diff

List ten APIs you use in iOS development that are unavailable in freestanding Swift.

### Exercise 2: StaticString Logger

Implement `log(_ message: StaticString)` writing to a stub UART function. Test on macOS by printing to stdout.

### Exercise 3: Panic Handler

Write a `fatalError` that toggles an LED in a loop before halting — visual panic indicator.

### Exercise 4: Size Audit

Build a minimal blink and record `.text` + `.data` size. Compare after adding one `String` (if it links).

### Exercise 5: Port a Struct Driver

Take a struct-based driver from [01-swift-basics.md](./01-swift-basics.md) and confirm it compiles with freestanding flags.

---

## References

- [Swift Forums — Embedded](https://forums.swift.org/c/development/embedded/43)
- [swift-embedded-examples](https://github.com/apple/swift-embedded-examples)
- [Embedded Swift Vision (Apple)](https://github.com/apple/swift-evolution/blob/main/proposals/0413-embedded-swift.md)
- [Lesson 03 — Memory Layout](./03-memory-layout.md)
- [Embedded Rust no_std equivalent](../learning/02-no-std.md)

---

*Previous: [01-swift-basics.md](./01-swift-basics.md) | Next: [03-memory-layout.md](./03-memory-layout.md)*
