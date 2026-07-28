# Lesson 07: Protocol-Oriented HAL

A **Hardware Abstraction Layer (HAL)** exposes peripheral operations through stable APIs. In Swift, HAL is naturally expressed as **protocols** — the equivalent of Rust's **embedded-hal** traits. Application and driver code depend on protocols, not chip-specific registers.

**Prerequisites:** [06-register-programming.md](./06-register-programming.md)  
**Next:** [08-gpio.md](./08-gpio.md)  
**See also:** [05-embedded-architecture.md](./05-embedded-architecture.md), [glossary.md](./glossary.md)

---

## Theory

### Why Protocol-Oriented HAL?

| Benefit | Explanation |
|---------|-------------|
| **Portability** | Swap ESP32 impl for STM32 without changing app |
| **Testability** | Mock protocols on macOS host |
| **Composition** | Generic algorithms work on any conforming type |
| **Zero cost** | Generic specialization = static dispatch, no vtable |

### Core Digital I/O Protocols

```swift
/// Errors shared across GPIO operations
public enum GPIOError: Error, Equatable {
    case invalidPin
    case notConfigured
    case hardwareFault
}

/// Single digital input pin
public protocol DigitalInputPin {
    func isHigh() -> Result<Bool, GPIOError>
    func isLow() -> Result<Bool, GPIOError>
}

public extension DigitalInputPin {
    func isLow() -> Result<Bool, GPIOError> {
        isHigh().map { !$0 }
    }
}

/// Single digital output pin
public protocol DigitalOutputPin {
    mutating func setHigh() -> Result<Void, GPIOError>
    mutating func setLow() -> Result<Void, GPIOError>
}

public extension DigitalOutputPin {
    mutating func toggle() -> Result<Void, GPIOError> {
        // Default impl requires readback — override if hardware supports
        fatalError("Override toggle() for your HAL")
    }
}
```

### Delay Protocol

```swift
public protocol DelayMs {
    mutating func delayMs(_ ms: UInt32)
}

public protocol DelayUs {
    mutating func delayUs(_ us: UInt32)
}
```

### Error Model

Embedded Swift favors small typed errors over `throws` with existential boxes:

```swift
public enum SPIError: Error {
    case busBusy
    case timeout
    case modeFault
}

public protocol SPIDevice {
    func transfer(_ buffer: inout [UInt8]) -> Result<Void, SPIError>
}
```

Use `Result` for recoverable errors; `fatalError` only for unrecoverable invariant violations.

### Generic Algorithms

Write once, specialize at compile time:

```swift
public func blink<P: DigitalOutputPin, D: DelayMs>(
    pin: inout P,
    delay: inout D,
    times: UInt32
) -> Result<Void, GPIOError> {
    for _ in 0..<times {
        _ = pin.setHigh()
        delay.delayMs(200)
        _ = pin.setLow()
        delay.delayMs(200)
    }
    return .success(())
}
```

No heap allocation, no dynamic dispatch when generic types are concrete.

### Mock Implementations for Testing

```swift
#if canImport(XCTest)
public struct MockOutputPin: DigitalOutputPin {
    public private(set) var level: Bool = false
    public var setHighCallCount: UInt32 = 0

    public mutating func setHigh() -> Result<Void, GPIOError> {
        level = true
        setHighCallCount += 1
        return .success(())
    }

    public mutating func setLow() -> Result<Void, GPIOError> {
        level = false
        return .success(())
    }
}
#endif
```

Run with `swift test` on macOS before flashing hardware.

---

## Hardware Overview

HAL protocols are chip-agnostic. Implementations bind to hardware:

| Protocol | ESP32-S3 impl | STM32F411 impl | RP2040 impl |
|----------|---------------|----------------|-------------|
| `DigitalOutputPin` | `ESP32GpioPin` | `Stm32GpioPin` | `Rp2040GpioPin` |
| `DelayMs` | `BusyLoopDelay` | `SysTickDelay` | `TimerDelay` |
| `BlockingUart` | `ESP32Uart` | `Stm32Uart` | `Rp2040Uart` |

Maturity note: you may need to write these implementations yourself for Embedded Swift — unlike Rust's mature `embedded-hal` ecosystem.

---

## Wiring Diagram

HAL abstracts wiring — application sees logical pins:

```
Application ──► DigitalOutputPin protocol ──► ESP32GpioPin ──► GPIO2 ──► LED

The protocol boundary is a software line, not a physical wire.
```

Physical wiring for GPIO exercises: [08-gpio.md](./08-gpio.md).

---

## Memory & Register Explanation

Protocol witnesses and generics compile away:

```swift
func demo<P: DigitalOutputPin>(pin: inout P) { ... }

let mut concrete = ESP32GpioPin(pin: 2)
demo(pin: &concrete)
// Compiler monomorphizes demo for ESP32GpioPin — same as writing pin-specific code
```

Avoid **`any DigitalOutputPin`** (existential) on MCU hot paths — introduces vtable and potential allocation.

Static buffers for bus transfers:

```swift
public protocol I2CBus {
    func writeRead(
        address: UInt8,
        write: UnsafeBufferPointer<UInt8>,
        read: UnsafeMutableBufferPointer<UInt8>
    ) -> Result<Void, I2CError>
}
```

---

## HAL-Style Swift Implementation

Complete mini-HAL module structure:

**Sources/EmbeddedHAL/Digital.swift** — protocols (shown above)

**Sources/EmbeddedHAL/BlockingDelay.swift:**

```swift
public struct BusyLoopDelay {
    let cyclesPerMs: UInt32

    public init(cpuHz: UInt32) {
        cyclesPerMs = cpuHz / 1000
    }
}

extension BusyLoopDelay: DelayMs {
    public mutating func delayMs(_ ms: UInt32) {
        for _ in 0..<ms {
            var c = cyclesPerMs
            while c > 0 {
                c &-= 1
            }
        }
    }
}
```

**Sources/ESP32HAL/GpioPin.swift:**

```swift
import EmbeddedHAL

public struct ESP32GpioPin: DigitalOutputPin {
    let pin: UInt8
    let mask: UInt32

    public init(pin: UInt8) {
        self.pin = pin
        self.mask = UInt32(1) << pin
        gpioConfigureOutput(pin: pin)
    }

    public mutating func setHigh() -> Result<Void, GPIOError> {
        gpioOutSet(mask: mask)
        return .success(())
    }

    public mutating func setLow() -> Result<Void, GPIOError> {
        gpioOutClear(mask: mask)
        return .success(())
    }
}

// PAC-level functions from Lesson 06
func gpioOutSet(mask: UInt32) { /* MMIO */ }
func gpioOutClear(mask: UInt32) { /* MMIO */ }
func gpioConfigureOutput(pin: UInt8) { /* MMIO + mux */ }
```

**Sources/App/main.swift:**

```swift
import EmbeddedHAL
import ESP32HAL

@main
struct App {
    static func main() {
        var led = ESP32GpioPin(pin: 2)
        var delay = BusyLoopDelay(cpuHz: 240_000_000)
        _ = blink(pin: &led, delay: &delay, times: 0xFFFF_FFFF)
    }
}
```

---

## Bare-Metal / MMIO Swift Notes

HAL sits directly above MMIO — implementations call register helpers:

```swift
@inline(__always)
func gpioOutSet(mask: UInt32) {
    UnsafeMutablePointer<UInt32>(bitPattern: 0x6000_4010)!.pointee = mask
}
```

Keep MMIO in BSP/PAC files; protocols never import raw addresses.

---

## Step-by-Step Explanation

### Step 1: Define Error Enums

One enum per subsystem — keep cases under ~8 for clarity.

### Step 2: Define Minimal Protocols

Start with `DigitalInputPin`, `DigitalOutputPin`, `DelayMs`.

### Step 3: Add Default Implementations

Use `extension` for derived methods like `isLow()`.

### Step 4: Implement for One Board

Prove the design with ESP32-S3 or STM32.

### Step 5: Write Generic Consumer

`blink`, `buttonWait`, etc. using protocol constraints.

### Step 6: Add Mock + Tests

Host tests validate logic without hardware.

### Step 7: Second Board Port

Implement same protocols for RP2040 — generic code should compile unchanged.

---

## Common Mistakes

| Mistake | Symptom | Fix |
|---------|---------|-----|
| Fat protocols | Hard to implement | Split into focused protocols |
| Existential types on MCU | Bloat, slow | Use generics |
| Ignoring Result errors | Silent failures | Log or propagate |
| Leaking MMIO into protocol | Not portable | Hide in impl |
| No mock types | Can't unit test | Add MockOutputPin |
| `toggle()` without readback | Wrong on open-drain | Document electrical mode |

---

## Debugging Tips

1. **Test on host first** — `swift test` catches logic bugs.
2. **Log protocol calls** — wrap impl with counting decorator struct.
3. **Compare mock vs hardware** — same generic function, two backends.
4. **Type-check generics** — explicit types at call site when compiler errors are cryptic.
5. **One protocol at a time** — bring up output before input.

---

## Performance Tips

| Tip | Detail |
|-----|--------|
| Generics over existentials | Static dispatch |
| `@inline(__always)` on MMIO in impl | Not on protocol witness |
| Batch operations protocol | `setMultiple(mask:)` if hardware supports |
| Avoid Result in inner ISR loop | Use bool or status code |
| Specialized delay impl | SysTick beats busy-loop for accuracy |

---

## Exercises

### Exercise 1: Input Protocol

Implement `DigitalInputPin` for ESP32 with pull-up config.

### Exercise 2: Generic Chase

Write `chasePattern` cycling through three `DigitalOutputPin` pins.

### Exercise 3: Mock Test

Test that `blink(..., times: 3)` calls `setHigh` exactly 3 times.

### Exercise 4: Second Board

Port `ESP32GpioPin` to a stub `Stm32GpioPin` with same protocol.

### Exercise 5: Decorator

Implement `LoggingPin<P: DigitalOutputPin>` wrapping any output pin with UART logs.

---

## References

- [Protocol-Oriented Programming (Swift)](https://docs.swift.org/swift-book/documentation/the-swift-programming-language/protocols/)
- [Embedded Rust embedded-hal](https://docs.rs/embedded-hal/latest/embedded_hal/)
- [Lesson 08 — GPIO](./08-gpio.md)
- [Lesson 04 — SwiftPM multi-target layout](./04-swiftpm.md)
- [Embedded Rust HAL lesson](../learning/07-hal.md)

---

*Previous: [06-register-programming.md](./06-register-programming.md) | Next: [08-gpio.md](./08-gpio.md)*
