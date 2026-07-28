# Lesson 01: Swift Basics for Embedded

Swift on embedded systems uses the same core language as iOS and macOS, but the **constraints** differ: no Foundation by default, limited heap, no ARC for many freestanding targets, and every byte of RAM matters. This lesson covers the language features you will encounter daily in firmware.

**Prerequisites:** Basic programming experience in any language.  
**Next:** [02-embedded-swift.md](./02-embedded-swift.md)  
**See also:** [glossary.md](./glossary.md), [07-hal.md](./07-hal.md)

---

## Theory

### Why Swift for Embedded?

| Advantage | Explanation |
|-----------|-------------|
| **Type safety** | Optionals eliminate null pointer crashes; enums model hardware states |
| **Value semantics** | Structs copy predictably — no hidden sharing like C pointers |
| **Protocol-oriented design** | HAL traits map naturally to Swift protocols |
| **Modern concurrency** | async/await and actors (where supported) structure concurrent code |
| **Growing embedded support** | Apple Embedded Swift targets microcontrollers directly |

Swift does **not** eliminate all firmware bugs — logic errors and hardware mistakes still happen — but it eliminates many pointer and type confusion bugs common in C firmware.

### Embedded vs Host Swift

| Feature | Host (iOS/macOS) | Embedded (freestanding) |
|---------|------------------|---------------------------|
| Foundation | Available | Not available |
| Heap / ARC | Automatic for classes | Limited or disabled |
| Concurrency | Full Swift Concurrency | Subset; custom executor |
| I/O | Files, URLSession | Registers, GPIO, UART |
| Error handling | `throws` + Foundation errors | `Result` + custom enums |
| Logging | `os_log`, `print` | UART, semihosting |

### Value Types vs Reference Types

| Kind | Examples | Memory | Embedded preference |
|------|----------|--------|---------------------|
| **Value (struct, enum)** | `GPIOConfig`, `PinState` | Stack or inline | **Preferred** — predictable, no ARC |
| **Reference (class, actor)** | `SensorDriver` (host) | Heap + ARC | Avoid on MCU unless static singleton |

```swift
// Value type — copied, no reference counting overhead
struct PinConfig {
    var number: UInt8
    var mode: PinMode
}

// Reference type — ARC overhead; often unavailable in freestanding mode
class NetworkManager {
    var connection: Connection?
}
```

On embedded targets, prefer **structs and enums** for driver state. Use classes on the host (iOS companion apps) where ARC is fully supported.

### Optionals

`Optional` (written `T?`) represents a value that may be absent — safer than null pointers.

```swift
enum PinError: Error {
    case invalidPin
    case hardwareFault
}

func readButton() -> Bool? {
    // Returns nil if hardware not initialized
    guard isInitialized else { return nil }
    return gpioRead(pin: 0) == .low
}

// Unwrapping patterns
if let pressed = readButton(), pressed {
    toggleLED()
}

// Guard for early exit
guard let level = readPin(2) else {
    return
}
```

### Result Type

`Result<Success, Failure>` models operations that succeed or fail — ideal for HAL methods.

```swift
enum GPIOError: Error {
    case pinBusy
    case invalidConfiguration
}

protocol DigitalOutputPin {
    mutating func setHigh() -> Result<Void, GPIOError>
    mutating func setLow() -> Result<Void, GPIOError>
}

func blink<P: DigitalOutputPin>(pin: inout P) -> Result<Void, GPIOError> {
    pin.setHigh()
    delay(ms: 500)
    return pin.setLow()
}
```

### Protocols (HAL Foundation)

Protocols define capabilities without specifying implementation — the Swift equivalent of Rust traits or C interfaces.

```swift
protocol DigitalInputPin {
    func isHigh() -> Result<Bool, GPIOError>
    func isLow() -> Result<Bool, GPIOError>
}

protocol DigitalOutputPin {
    mutating func setHigh() -> Result<Void, GPIOError>
    mutating func setLow() -> Result<Void, GPIOError>
    mutating func toggle() -> Result<Void, GPIOError>
}
```

Generic functions constrain on protocols for zero-cost abstraction:

```swift
func waitForPress<P: DigitalInputPin>(_ pin: P) {
    while (pin.isLow() ?? false) == false {
        delay(ms: 10)
    }
}
```

### Error Handling for Embedded

Avoid heavy exception-style patterns. Use small `Error` enums:

```swift
enum PeripheralError: Error {
    case timeout
    case busFault
    case invalidParameter
}

func i2cWrite(address: UInt8, data: [UInt8]) -> Result<Void, PeripheralError> {
    guard !data.isEmpty else { return .failure(.invalidParameter) }
    // ... hardware access ...
    return .success(())
}
```

---

## Hardware Overview

This is a language lesson — no wiring required. All code examples are written so they **compile conceptually** for an ESP32-S3, but can run on the host for practice.

The ESP32-S3 has limited RAM (~512 KB SRAM). Accidentally copying large arrays on every loop iteration wastes stack space — value semantics help you see copies explicitly.

| Resource | ESP32-S3 | Why it matters for Swift |
|----------|----------|--------------------------|
| SRAM | 512 KB | Stack for struct locals |
| Flash | 8–16 MB | Code + constants (`StaticString`) |
| Heap | Optional, limited | Avoid class allocation on MCU |

---

## Wiring Diagram

Not applicable for this lesson. For your first hardware exercise, see [08-gpio.md](./08-gpio.md).

---

## Memory & Register Explanation

Embedded Swift interacts with memory in three primary locations:

```
┌──────────────────────────────────────────────┐
│  Flash (ROM) — program code, constants         │
│  __TEXT, __CONST                               │
├──────────────────────────────────────────────┤
│  RAM — stack (function locals), .data, .bss  │
│  Stack grows ↓    Heap grows ↑ (if enabled)    │
├──────────────────────────────────────────────┤
│  MMIO Registers — peripheral control         │
│  Fixed addresses (e.g., 0x6000_4000 GPIO)      │
└──────────────────────────────────────────────┘
```

Value types on the stack are **automatically reclaimed** when scope ends — no GC, no ARC for structs. See [03-memory-layout.md](./03-memory-layout.md) for linker-level detail.

---

## HAL-Style Swift Implementation

HAL code relies on protocols and value semantics:

```swift
/// Pin wrapper — struct owns configuration state
struct OutputPin<P: PinIdentifier> {
    private var pin: P
    private var level: PinLevel

    mutating func setHigh() -> Result<Void, GPIOError> {
        level = .high
        return hardwareWrite(pin: pin, level: .high)
    }
}

func blinkOnce<P: PinIdentifier>(pin: inout OutputPin<P>) -> Result<Void, GPIOError> {
    _ = pin.setHigh()
    delay(ms: 500)
    return pin.setLow()
}
```

Key observations:

- `mutating func` — struct methods that change state require `mutating`
- `Result<Void, GPIOError>` — explicit error path, no exceptions on embedded
- Generics (`P: PinIdentifier`) — compile-time polymorphism with no runtime vtable on constrained calls

---

## Bare-Metal / MMIO Swift Notes

Bare-metal code uses `UnsafePointer` and `UnsafeMutablePointer` for register access:

```swift
let gpioOutReg = UnsafeMutablePointer<UInt32>(
    bitPattern: 0x6000_4004
)!

func togglePin(mask: UInt32) {
    // Memory ordering matters — see Lesson 06
    let current = gpioOutReg.pointee
    gpioOutReg.pointee = current ^ mask
}
```

`mask: UInt32` is a value type — passed by copy, no reference counting. Register access requires unsafe pointers because the compiler cannot verify hardware side effects. Wrap unsafe code in small functions and document invariants.

---

## Step-by-Step Explanation

### Step 1: Learn Value Semantics

Structs copy on assignment. Understand when copies happen:

```swift
var configA = PinConfig(number: 2, mode: .output)
var configB = configA   // Copy — independent values
configB.number = 4
// configA.number is still 2
```

### Step 2: Use Optionals for Missing State

Replace sentinel values (-1, NULL) with optionals:

```swift
var lastReading: Int? = nil

func updateSensor(value: Int) {
    lastReading = value
}
```

### Step 3: Model Errors with Enums

One enum per subsystem keeps error types small and stack-friendly:

```swift
enum UARTError: Error {
    case framingError
    case overrun
    case notInitialized
}
```

### Step 4: Define HAL Protocols Early

Start with `DigitalInputPin` and `DigitalOutputPin` before board-specific code. See [07-hal.md](./07-hal.md).

### Step 5: Prefer `StaticString` for Logging

Heap-free string literals for UART output:

```swift
func log(_ message: StaticString) {
    uartWrite(message)
}

log("System started")
```

### Step 6: Understand `mutating` on Structs

Methods that modify struct properties must be marked `mutating`. Call sites need `var`:

```swift
var led = OutputPin(...)
led.setHigh()   // OK — var binding
```

### Step 7: Generics for Portability

Write algorithms once, specialize at compile time for each board.

---

## Common Mistakes

| Mistake | Symptom | Fix |
|---------|---------|-----|
| Using classes on MCU | Link errors, heap allocation | Prefer structs |
| Force-unwrap (`!`) everywhere | Runtime trap on nil | Use `guard let` / `if let` |
| Ignoring `Result` errors | Silent hardware failures | Handle or propagate errors |
| Copying large structs in loops | Stack overflow | Pass by reference or use `inout` |
| String concatenation in loops | Heap churn on host; fails on MCU | Use `StaticString` |
| Missing `mutating` keyword | Compile error on struct methods | Add `mutating` |

---

## Debugging Tips

1. **Build for macOS first** — validate logic with `swift test` on host before cross-compiling.
2. **Print enum cases** — `Switch` on errors to see which failure path runs.
3. **Enable warnings** — `-warnings-as-errors` catches unused Results.
4. **Use LLDB** — `po variableName` inspects struct fields on host builds.
5. **Static assertions** — `static assert` patterns via compile-time checks where available.

---

## Performance Tips

| Tip | Detail |
|-----|--------|
| Prefer structs over classes | No ARC retain/release on hot paths |
| Use `UInt32` for register values | Matches hardware width |
| Mark hot paths `@inline(__always)` sparingly | Only after profiling |
| Avoid protocol existentials (`any P`) on MCU | Use generics for static dispatch |
| Stack allocate fixed buffers | `[UInt8; 64]` style over dynamic arrays |

---

## Exercises

### Exercise 1: PinConfig Struct

Create a `PinConfig` struct with `number`, `mode`, and `pull`. Add a method `isValid() -> Bool`.

### Exercise 2: Result-Based API

Implement `setPin(_ pin: UInt8, high: Bool) -> Result<Void, GPIOError>` with at least three error cases. Write unit tests on macOS.

### Exercise 3: Protocol Consumer

Write a generic function `readUntilPress<P: DigitalInputPin>(_ pin: P) -> Result<Int, GPIOError>` that counts loop iterations until the pin reads low.

### Exercise 4: Optional vs Sentinel

Refactor a C-style API using `-1` as error into Swift optionals.

### Exercise 5: Error Propagation

Chain three `Result`-returning operations with `flatMap` or `guard case .success`.

---

## References

- [The Swift Programming Language](https://docs.swift.org/swift-book/)
- [Swift Evolution — Embedded Swift](https://github.com/apple/swift-evolution)
- [Lesson 02 — Embedded Swift](./02-embedded-swift.md)
- [Lesson 07 — HAL Protocols](./07-hal.md)
- [Embedded Rust equivalent](../learning/01-rust-basics.md) (comparison)

---

*Previous: [00-roadmap.md](./00-roadmap.md) | Next: [02-embedded-swift.md](./02-embedded-swift.md)*
