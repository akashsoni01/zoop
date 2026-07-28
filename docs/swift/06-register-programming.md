# Lesson 06: Register Programming — MMIO, Volatile, and Bitfields

Microcontroller peripherals are controlled by **registers** — fixed memory addresses where each bit or field has hardware meaning. This lesson teaches safe **MMIO (Memory-Mapped I/O)** access patterns in Embedded Swift.

**Prerequisites:** [05-embedded-architecture.md](./05-embedded-architecture.md)  
**Next:** [07-hal.md](./07-hal.md)  
**See also:** [08-gpio.md](./08-gpio.md), [glossary.md](./glossary.md)

---

## Theory

### What Is MMIO?

Unlike normal RAM, **MMIO** addresses trigger hardware actions when read or written:

```
CPU write to 0x6000_4010 = 0x0000_0004
         │
         ▼
GPIO hardware sets pin 2 HIGH
```

The compiler must **not** optimize away or reorder these accesses — hence **volatile** semantics.

### Register Map Example (ESP32-S3 GPIO)

| Offset | Name | Access | Purpose |
|--------|------|--------|---------|
| 0x00 | GPIO_OUT | R/W | Output level |
| 0x10 | GPIO_OUT_W1TS | W | Set bits (write 1 to set) |
| 0x14 | GPIO_OUT_W1TC | W | Clear bits (write 1 to clear) |
| 0x20 | GPIO_ENABLE | R/W | Output enable |
| 0x3C | GPIO_IN | R | Input level |

**W1TS / W1TC** (Write 1 To Set / Clear) allow atomic bit manipulation without read-modify-write races.

### Bitfields

A 32-bit register often packs multiple fields:

```
GPIO_ENABLE register (conceptual):
┌────────────────────────────────────────┐
│ 31 ... 22 │ 21 ... 0                    │
│  reserved │ EN per pin (1=output enable)│
└────────────────────────────────────────┘
```

Access patterns:

```swift
// Set pin 2 as output: enable bit 2
let enableMask: UInt32 = 1 << 2
reg.pointee.enable |= enableMask

// Clear pin 2 enable (make input)
reg.pointee.enable &= ~enableMask
```

### Volatile Semantics in Swift

Swift lacks a `volatile` keyword like C. Use:

1. **`UnsafeMutablePointer`** with careful usage
2. **Compiler barriers** via `@_semantics("optimize.sil.none")` (internal)
3. **Wrapper struct** with documented read/write side effects
4. **C volatile shim** for critical sequences

```swift
struct VolatileUInt32 {
    private let address: UInt

    init(address: UInt) {
        self.address = address
    }

    var value: UInt32 {
        get {
            UnsafeMutablePointer<UInt32>(bitPattern: address)!.pointee
        }
        set {
            UnsafeMutablePointer<UInt32>(bitPattern: address)!.pointee = newValue
        }
    }
}
```

Document that every access has hardware side effects. Do not cache reads the compiler could elide across multiple logical operations without reloading.

### Read-Modify-Write Hazards

```swift
// RMW — unsafe if ISR also modifies same register
var val = reg.pointee.out
val |= mask
reg.pointee.out = val

// Prefer atomic set/clear registers when available
reg.pointee.outSet = mask   // W1TS — no read needed
```

ISRs and main code sharing a register need **critical sections** or hardware atomic aliases.

---

## Hardware Overview

### ESP32-S3 GPIO Base Address

- GPIO controller: `0x6000_4000`
- IO_MUX (pin function select): `0x6000_9000`

### STM32F411 GPIOA

- Base: `0x4002_0000`
- MODER (mode), ODR (output data), IDR (input data), BSRR (bit set/reset)

### RP2040 SIO (GPIO)

- Base: `0xD000_0000`
- Direct `GPIO_OUT`, `GPIO_OE` (output enable), `GPIO_IN`

Always verify addresses in the current **Technical Reference Manual (TRM)** — errata happen.

---

## Wiring Diagram

Register programming is invisible on the wire — but GPIO registers control physical pins:

```
Register write ──► GPIO pad ──► Pin header ──► LED / scope probe

Example: GPIO_OUT_W1TS bit 2 = 1
         │
         └──► GPIO2 header pin goes HIGH (~3.3 V)
```

Use a multimeter on the pin to verify register writes during bring-up.

---

## Memory & Register Explanation

MMIO addresses are **not** stored in RAM — your code holds constants:

```swift
enum GPIO {
    static let base: UInt = 0x6000_4000
    static let outSet: UInt = base + 0x10
    static let outClear: UInt = base + 0x14
}
```

These compile to immediate values in `.text` — no RAM consumed for the address itself.

**Memory ordering:** On ARM, peripheral accesses may need barriers before/after if mixing with DMA or other cores. ESP32-S3 dual-core: protect shared GPIO with spinlocks or assign pins per core.

---

## HAL-Style Swift Implementation

Typed register block (PAC-style):

```swift
struct GPIORegisterBlock {
    var out: UInt32           // 0x00
    var reserved0: (UInt32, UInt32, UInt32, UInt32)
    var outSet: UInt32        // 0x10
    var outClear: UInt32      // 0x14
    var enable: UInt32        // 0x20
    // ... padded to match hardware layout
}

final class GPIOController {
    private let registers: UnsafeMutablePointer<GPIORegisterBlock>

    init(base: UInt = 0x6000_4000) {
        registers = UnsafeMutablePointer(bitPattern: base)!
    }

    func setOutputEnabled(pin: UInt8) {
        let mask = UInt32(1) << pin
        registers.pointee.enable |= mask
    }

    func setHigh(pin: UInt8) {
        registers.pointee.outSet = UInt32(1) << pin
    }

    func setLow(pin: UInt8) {
        registers.pointee.outClear = UInt32(1) << pin
    }

    func isInputHigh(pin: UInt8) -> Bool {
        let inputs = registers.pointee.out  // Use GPIO_IN offset in real code
        return (inputs & (UInt32(1) << pin)) != 0
    }
}
```

Use `final class` sparingly on embedded — a `struct` wrapping the pointer is often better:

```swift
struct GPIOController {
    var registers: UnsafeMutablePointer<GPIORegisterBlock>
    // mutating methods...
}
```

---

## Bare-Metal / MMIO Swift Notes

Minimal blink — no types, raw pointers:

```swift
let OUT_SET = UnsafeMutablePointer<UInt32>(bitPattern: 0x6000_4010)!
let OUT_CLR = UnsafeMutablePointer<UInt32>(bitPattern: 0x6000_4014)!
let ENABLE  = UnsafeMutablePointer<UInt32>(bitPattern: 0x6000_4020)!

func gpioInitPin2Output() {
    // Enable output on pin 2
    ENABLE.pointee = ENABLE.pointee | (1 << 2)
}

func blinkRaw() {
    let mask: UInt32 = 1 << 2
    while true {
        OUT_SET.pointee = mask
        busyWait(ms: 500)
        OUT_CLR.pointee = mask
        busyWait(ms: 500)
    }
}
```

**Safety invariants** (document in comments):

1. Addresses valid for this chip revision
2. Clock to GPIO peripheral enabled
3. IO_MUX configured for GPIO function
4. No other code modifies same pins concurrently

---

## Step-by-Step Explanation

### Step 1: Find Register Map

Open TRM chapter for your peripheral. Note base address and offsets.

### Step 2: Verify with Debugger

Halt CPU, manually write to OUT_SET in LLDB — confirm pin toggles.

### Step 3: Define Swift Struct Layout

Match offsets exactly. Use padding tuples for reserved gaps.

### Step 4: Enable Clock and Mux

GPIO registers are inert until clocks and pin mux are configured — BSP responsibility.

### Step 5: Use W1TS/W1TC

Prefer set/clear registers over RMW on `OUT`.

### Step 6: Wrap in HAL Methods

Expose `setHigh(pin:)` hiding register details from application.

### Step 7: Add Input Read Path

Read `GPIO_IN` for buttons — see [08-gpio.md](./08-gpio.md).

---

## Common Mistakes

| Mistake | Symptom | Fix |
|---------|---------|-----|
| Wrong base address | Hard fault or no effect | Check TRM |
| Struct layout mismatch | Wrong bits toggled | Match offsets with padding |
| Cached register read | Stale input value | Re-read before each decision |
| RMW race with ISR | Glitchy outputs | W1TS/W1TC or critical section |
| Forgot IO_MUX | GPIO unresponsive | Configure pin function |
| Writing read-only field | Unpredictable behavior | Check access permissions in TRM |

---

## Debugging Tips

1. **LLDB memory write** — `memory write 0x60004010 0x4` tests hardware path.
2. **Logic analyzer** — confirm pin toggles match register writes.
3. **Read-back** — after configure, read register and compare expected mask.
4. **SVD/codegen** — generate PAC from vendor SVD XML to avoid manual offsets.
5. **Single-step** — verify init sequence order in debugger.

---

## Performance Tips

| Tip | Detail |
|-----|--------|
| W1TS/W1TC for single pins | One store instruction |
| Batch via `OUT` write | Multiple pins same value simultaneously |
| `@inline(__always)` on accessors | Reduce call overhead in loops |
| Avoid func calls in tight ISR | Inline or open-code |
| Cache pointer, not register values | Pointer in struct; reload each access |

---

## Exercises

### Exercise 1: Register Map Table

For your MCU, copy GPIO register table from TRM into your notes with offsets.

### Exercise 2: Volatile Wrapper

Implement `VolatileUInt32` with get/set and use it for OUT_SET.

### Exercise 3: Bitfield Extract

Write `func extractField(value: UInt32, shift: UInt8, width: UInt8) -> UInt32`.

### Exercise 4: RMW vs Atomic

Toggle pin using RMW, then rewrite using W1TS/W1TC. Compare assembly output if possible.

### Exercise 5: Input Read

Configure pin 0 as input with pull-up; read GPIO_IN and log state over UART.

---

## References

- [ESP32-S3 TRM — GPIO & IO_MUX](https://www.espressif.com/sites/default/files/documentation/esp32-s3_technical_reference_manual_en.pdf)
- [STM32 RM0383 — GPIO](https://www.st.com/resource/en/reference_manual/rm0383-stm32f411-advanced-armbased-32bit-mcus-stmicroelectronics.pdf)
- [RP2040 Datasheet — SIO](https://datasheets.raspberrypi.com/rp2040/rp2040-datasheet.pdf)
- [Lesson 07 — HAL](./07-hal.md)
- [Embedded Rust register lesson](../learning/06-register-programming.md)

---

*Previous: [05-embedded-architecture.md](./05-embedded-architecture.md) | Next: [07-hal.md](./07-hal.md)*
