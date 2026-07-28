# Lesson 05: Embedded Architecture — PAC, HAL, and BSP

Production firmware is organized in layers. Understanding **PAC (Peripheral Access Crate)**, **HAL (Hardware Abstraction Layer)**, and **BSP (Board Support Package)** helps you navigate codebases, write portable drivers, and know where to fix bugs.

**Prerequisites:** [04-swiftpm.md](./04-swiftpm.md)  
**Next:** [06-register-programming.md](./06-register-programming.md)  
**See also:** [07-hal.md](./07-hal.md), [glossary.md](./glossary.md)

---

## Theory

### Layer Diagram

```
┌──────────────────────────────────────────────┐
│  Application                                  │
│  (your business logic, state machines)        │
├──────────────────────────────────────────────┤
│  Drivers                                      │
│  (sensor/display drivers using HAL protocols) │
├──────────────────────────────────────────────┤
│  BSP — Board Support Package                  │
│  (DevKit pin aliases, clock init, USB-UART)   │
├──────────────────────────────────────────────┤
│  HAL — Hardware Abstraction Layer             │
│  (GPIO, UART, SPI protocols + chip impl)      │
├──────────────────────────────────────────────┤
│  PAC — Peripheral Access Crate                │
│  (typed register definitions, raw MMIO)       │
├──────────────────────────────────────────────┤
│  Startup + Linker Script                      │
│  (reset vector, memory layout)                │
└──────────────────────────────────────────────┘
```

### Layer Responsibilities

| Layer | Owns | Does NOT own |
|-------|------|--------------|
| **PAC** | Register addresses, bit masks | Pin meaning, clock policy |
| **HAL** | Peripheral configuration API | Which pin is the LED |
| **BSP** | Board pin map, `Board::init()` | Sensor protocols |
| **App** | Application state | Register bit twiddling |

### Swift Mapping

| Rust term | Swift equivalent |
|-----------|------------------|
| PAC crate | `struct` register wrappers, code-generated from SVD |
| embedded-hal traits | Swift protocols (`DigitalOutputPin`, etc.) |
| BSP crate | `ESP32S3DevKit` module with pin constants |
| `Peripherals::take()` | One-time hardware init singleton pattern |

### Clock Tree

Before any peripheral works, **clocks** must be configured:

```
XTAL / RC oscillator
        │
        ▼
      PLL ──► CPU clock
        │
        ├──► AHB bus ──► SRAM, DMA
        │
        └──► APB peripherals ──► UART, TIM, GPIO
```

BSP `init()` typically:

1. Configure clock sources and PLL
2. Enable peripheral clock gates
3. Set flash wait states
4. Configure SysTick or timer for delays

### Dependency Direction

Dependencies flow **downward only**:

```
App → Drivers → BSP → HAL → PAC
```

Never import BSP from HAL. Never touch PAC from App (except bring-up exercises).

---

## Hardware Overview

### ESP32-S3 DevKitC-1 BSP Facts

| Signal | GPIO | Notes |
|--------|------|-------|
| User LED | GPIO2 (varies) | Active high |
| Boot button | GPIO0 | Strapping pin |
| USB-JTAG | Internal | No external probe |
| Default UART0 TX | GPIO43 | Console output |

### STM32 Nucleo-F411RE BSP Facts

| Signal | Pin | Notes |
|--------|-----|-------|
| User LED | PA5 | Active high |
| User button | PC13 | Active low |
| ST-Link UART | Virtual COM | PA2/PA3 |

### RP2040 Pico BSP Facts

| Signal | GPIO | Notes |
|--------|------|-------|
| On-board LED | GPIO25 | Active low |
| SWD | GPIO24/25 alt | Debug |

---

## Wiring Diagram

Architecture is software — but BSP maps to physical pins:

```
BSP Layer maps logical names to physical pins:

  board.led    ──►  GPIO2  (ESP32-S3)
  board.button ──►  GPIO0  (ESP32-S3)

Application code uses board.led — not raw GPIO numbers.
```

---

## Memory & Register Explanation

Each layer accesses memory differently:

| Layer | Access pattern |
|-------|----------------|
| PAC | `UnsafeMutablePointer<GPIO_Type>` at fixed address |
| HAL | Wraps PAC with `setHigh()` methods |
| BSP | Calls HAL with pin numbers from schematic |
| App | Calls `board.led.toggle()` |

**Singleton peripherals:** Many MCUs allow only one init of clock/UART. BSP enforces this:

```swift
struct ESP32S3Board {
    private static var initialized = false

    static func initOnce() -> ESP32S3Board? {
        guard !initialized else { return nil }
        initialized = true
        clockInit()
        return ESP32S3Board()
    }
}
```

---

## HAL-Style Swift Implementation

Full layer example — LED blink:

**PAC layer (simplified):**

```swift
struct GPIORegisters {
    var outSet: UInt32   // offset 0x10
    var outClear: UInt32 // offset 0x14
    // ...
}

func gpioBase() -> UnsafeMutablePointer<GPIORegisters> {
    UnsafeMutablePointer(bitPattern: 0x6000_4000)!
}
```

**HAL layer:**

```swift
protocol DigitalOutputPin {
    mutating func setHigh() -> Result<Void, GPIOError>
    mutating func setLow() -> Result<Void, GPIOError>
}

struct GpioPin: DigitalOutputPin {
    let mask: UInt32

    mutating func setHigh() -> Result<Void, GPIOError> {
        gpioBase().pointee.outSet = mask
        return .success(())
    }

    mutating func setLow() -> Result<Void, GPIOError> {
        gpioBase().pointee.outClear = mask
        return .success(())
    }
}
```

**BSP layer:**

```swift
struct ESP32S3DevKit {
    var led: GpioPin
    var bootButton: GpioInputPin

    init?() {
        guard clockInit() else { return nil }
        gpioEnable(pin: 2, mode: .output)
        gpioEnable(pin: 0, mode: .inputPullUp)
        led = GpioPin(mask: 1 << 2)
        bootButton = GpioInputPin(mask: 1 << 0)
    }
}
```

**Application:**

```swift
@main
struct App {
    static func main() {
        guard var board = ESP32S3DevKit() else {
            fatalError("Board init failed")
        }
        while true {
            _ = board.led.setHigh()
            delay(ms: 500)
            _ = board.led.setLow()
            delay(ms: 500)
        }
    }
}
```

---

## Bare-Metal / MMIO Swift Notes

Bring-up sometimes skips HAL and uses PAC directly — acceptable for learning, not for production:

```swift
// Direct PAC access — Lesson 06 covers safety rules
func bareMetalBlink() {
    let mask: UInt32 = 1 << 2
    let gpio = gpioBase()
    while true {
        gpio.pointee.outSet = mask
        busyWait()
        gpio.pointee.outClear = mask
        busyWait()
    }
}
```

Migrate to HAL once registers are understood.

---

## Step-by-Step Explanation

### Step 1: Identify Layers in Your Project

Open your firmware repo and label each file PAC/HAL/BSP/App.

### Step 2: Draw Clock Init Flow

Trace from reset to first GPIO toggle — list every init function.

### Step 3: Define BSP Pin Constants

```swift
enum BoardPin {
    static let led: UInt8 = 2
    static let button: UInt8 = 0
}
```

### Step 4: Implement HAL Protocol

Start with `DigitalOutputPin` — see [07-hal.md](./07-hal.md).

### Step 5: Write App Against BSP Only

Application should never hardcode GPIO numbers.

### Step 6: Port Test

Change `BoardPin.led` constant only — app code unchanged if layers are correct.

### Step 7: Document Dependencies

Ensure SwiftPM targets enforce layer direction.

---

## Common Mistakes

| Mistake | Symptom | Fix |
|---------|---------|-----|
| App writes registers directly | Unportable code | Route through HAL |
| HAL knows board pin names | Circular deps | Move names to BSP |
| Skip clock init | GPIO dead | Call BSP init first |
| Multiple peripheral init | Hard fault | Enforce initOnce pattern |
| Leaking PAC types in public API | Tight coupling | Expose protocols only |
| Giant god-module BSP | Unmaintainable | Split by peripheral |

---

## Debugging Tips

1. **Layer printf** — log at each init stage to find where boot stops.
2. **Compare with vendor C SDK** — verify clock config matches Espressif/ST example.
3. **Schematic vs BSP** — confirm GPIO numbers match PCB.
4. **Strip layers** — if HAL fails, test PAC directly to isolate bug.
5. **Dependency graph** — `swift package describe` validates structure.

---

## Performance Tips

| Tip | Detail |
|-----|--------|
| Inline PAC accessors | Hot path register reads |
| Batch clock enables | One write to clock enable register |
| Const pin masks at compile time | Avoid runtime shifts |
| Avoid virtual dispatch in ISR | Generics over protocols in hot paths |
| Cache peripheral base pointers | Don't recompute addresses |

---

## Exercises

### Exercise 1: Layer Diagram

Draw PAC/HAL/BSP/App for your board from scratch without looking at code.

### Exercise 2: BSP for Nucleo

Define `STM32NucleoF411` BSP with `led` and `button` using HAL protocols.

### Exercise 3: Init Guard

Implement `initOnce()` that returns optional second call.

### Exercise 4: Dependency Audit

List three violations of layer direction in a hypothetical bad codebase and fix them.

### Exercise 5: Clock Trace

Read ESP32-S3 or STM32 clock chapter; write five bullet points on what BSP must configure.

---

## References

- [ESP32-S3 TRM — Clocks](https://www.espressif.com/sites/default/files/documentation/esp32-s3_technical_reference_manual_en.pdf)
- [STM32 RM0383 — Reset and Clock Control](https://www.st.com/resource/en/reference_manual/rm0383-stm32f411-advanced-armbased-32bit-mcus-stmicroelectronics.pdf)
- [Lesson 06 — Register Programming](./06-register-programming.md)
- [Lesson 07 — HAL Protocols](./07-hal.md)
- [Embedded Rust architecture equivalent](../learning/05-embedded-architecture.md)

---

*Previous: [04-swiftpm.md](./04-swiftpm.md) | Next: [06-register-programming.md](./06-register-programming.md)*
