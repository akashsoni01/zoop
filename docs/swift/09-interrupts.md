# Lesson 09: Interrupts — IRQs, ISRs, and Actors

**Interrupts** let hardware notify your firmware immediately when events occur — button presses, timer ticks, UART bytes — without constant polling. In Swift, combine **Interrupt Service Routines (ISRs)** with **actors** and atomic state for safe concurrency.

**Prerequisites:** [08-gpio.md](./08-gpio.md)  
**Next:** [10-timers.md](./10-timers.md)  
**See also:** [glossary.md](./glossary.md), [Embedded Rust interrupts](../learning/09-interrupts.md)

---

## Theory

### Interrupt Concepts

| Term | Meaning |
|------|---------|
| **IRQ (Interrupt Request)** | Hardware signal requesting CPU attention |
| **ISR (Interrupt Service Routine)** | Short handler function running on interrupt |
| **NVIC (Nested Vectored Interrupt Controller)** | ARM interrupt router — priorities, nesting |
| **Vector table** | Table of ISR entry addresses at boot |
| **Critical section** | Code with interrupts masked |

### Interrupt Flow

```
Hardware event (button press)
        │
        ▼
NVIC asserts IRQ
        │
        ▼
CPU finishes current instruction
        │
        ▼
Push context, jump to ISR
        │
        ▼
ISR runs (keep SHORT)
        │
        ▼
Return to main code
```

### ISR Rules

1. **Keep ISRs short** — set flag, clear hardware flag, exit
2. **No blocking** — no delays, no mutex waits
3. **Minimal work** — defer to main loop or actor
4. **Shared state** — use atomics or critical sections

### Swift Concurrency Mapping

| Rust pattern | Swift equivalent |
|--------------|------------------|
| Embassy async | async/await + custom executor ([README](./README.md)) |
| RTIC shared resources | Actors + static atomics |
| `Mutex<RefCell<T>>` | Actor isolation or `ManagedAtomic` |
| Critical section crate | `disableInterrupts()` / `enableInterrupts()` |

On Embedded Swift, full Swift Concurrency runtime may be limited — **static atomics + main-loop dispatch** is the reliable baseline:

```swift
// ISR sets atomic flag; main loop reacts
static var buttonEventPending: ManagedAtomic<Bool> = ManagedAtomic(false)

// Called from ISR — must be safe
func gpioIsrHandler() {
    buttonEventPending.store(true, ordering: .relaxed)
    gpioClearInterruptFlag()
}
```

### Actor for Deferred Processing

On host or when executor available:

```swift
actor ButtonDebouncer {
    private var lastEventMs: UInt32 = 0

    func handlePress(nowMs: UInt32) -> Bool {
        if nowMs - lastEventMs > 50 {
            lastEventMs = nowMs
            return true
        }
        return false
    }
}
```

Main loop (or async Task) calls the actor — never from ISR directly.

---

## Hardware Overview

### ESP32-S3 GPIO Interrupt

- Any GPIO can generate edge/level interrupt
- Interrupt matrix routes to CPU
- Clear interrupt status in ISR to avoid re-entry

### STM32 EXTI

- **EXTI (External Interrupt)** lines map pins to NVIC
- Configure rising/falling edge on SYSCFG+EXTI registers

### Button Debounce Context

Mechanical buttons bounce 1–20 ms — ISR may fire multiple times per press. Debounce in main loop using timestamps from [10-timers.md](./10-timers.md).

---

## Wiring Diagram

Same as [08-gpio.md](./08-gpio.md) — interrupt configuration is software:

```
Button ──► GPIO0 ──► EXTI / GPIO interrupt matrix ──► NVIC ──► ISR

No wiring change from polling lesson — only firmware differs.
```

Optional: scope or logic analyzer on GPIO to see bounce edges triggering ISR.

---

## Memory & Register Explanation

### Vector Table (ARM Cortex-M)

```
Address 0x0000_0000 (or offset):
  Word 0: Initial SP
  Word 1: Reset handler
  Word 2: NMI handler
  ...
  Word N: GPIO IRQ handler address
```

Startup assembly installs handlers. Swift ISR functions need C calling convention:

```swift
@_cdecl("GPIO0_IRQHandler")
func gpio0IrqHandler() {
    gpioIsrHandler()
}
```

### Shared Static State

```swift
// Lives in .bss — visible to ISR and main
static var eventCount: ManagedAtomic<UInt32> = ManagedAtomic(0)
```

Document memory ordering when reading atomics in main loop.

---

## HAL-Style Swift Implementation

```swift
import EmbeddedHAL

/// GPIO interrupt manager — HAL layer
struct GpioInterrupt {
    let pin: UInt8

    func enableRisingEdge() -> Result<Void, GPIOError> {
        gpioConfigureInterrupt(pin: pin, edge: .rising)
        nvicEnableIRQ(irq: gpioIrqNumber(pin: pin), priority: 2)
        return .success(())
    }
}

// Application
static var toggleLedRequest = ManagedAtomic(false)

@_cdecl("GPIO0_IRQHandler")
func gpio0Handler() {
    toggleLedRequest.store(true, ordering: .relaxed)
    gpioClearInterruptStatus(pin: 0)
}

@main
struct InterruptApp {
    static func main() {
        var led = ESP32GpioOutputPin(pin: 2, initial: .low)
        var ledOn = false

        _ = GpioInterrupt(pin: 0).enableRisingEdge()

        while true {
            if toggleLedRequest.exchange(false, ordering: .acquire) {
                // Debounce with timer — simplified busy wait here
                delayMs(50)
                if gpioPinLow(0) {
                    ledOn.toggle()
                    if ledOn { _ = led.setHigh() }
                    else { _ = led.setLow() }
                }
            }
            // Sleep until interrupt — WFI instruction
            waitForInterrupt()
        }
    }
}
```

---

## Bare-Metal / MMIO Swift Notes

Enable GPIO interrupt on ESP32-S3 (simplified):

```swift
enum GPIOInt {
    static let status: UInt = 0x6000_4040
    static let statusClear: UInt = 0x6000_4044
    static let pinIntEn: UInt = 0x6000_4048
}

func enableGpioInterrupt(pin: UInt8) {
    let mask = UInt32(1) << pin
    reg(GPIOInt.pinIntEn).pointee |= mask
    // Configure pin for rising edge in separate reg — see TRM
}

@_cdecl("GPIO_IRQHandler")
func gpioIrq() {
    let status = reg(GPIOInt.status).pointee
    if (status & (1 << 0)) != 0 {
        toggleLedRequest.store(true, ordering: .relaxed)
        reg(GPIOInt.statusClear).pointee = (1 << 0)
    }
}
```

---

## Step-by-Step Explanation

### Step 1: Verify Polling Works

Complete [08-gpio.md](./08-gpio.md) first — confirms hardware.

### Step 2: Configure Pin for Interrupt

Edge type: usually **falling edge** for active-low button with pull-up.

### Step 3: Install Vector Handler

Link ISR symbol in vector table via startup code.

### Step 4: Enable NVIC IRQ

Set priority — lower number = higher priority on ARM.

### Step 5: ISR Sets Flag Only

No `delayMs` in ISR — ever.

### Step 6: Main Loop Processes Flag

Debounce, toggle LED, clear state.

### Step 7: Optional WFI

`waitForInterrupt()` reduces power between events.

---

## Common Mistakes

| Mistake | Symptom | Fix |
|---------|---------|-----|
| Long ISR | Missed interrupts, jitter | Set flag only |
| Forgot clear interrupt status | Infinite re-trigger | Clear in ISR |
| Debounce in ISR | Blocking | Debounce in main |
| Race on non-atomic shared var | Corrupted state | Use ManagedAtomic |
| Wrong IRQ priority | Starvation | Plan priority table |
| Calling actor from ISR | Undefined behavior | Signal main loop only |

---

## Debugging Tips

1. **ISR counter** — increment atomic in ISR; verify fires in main.
2. **LED toggle in ISR** — temporary debug only — proves ISR runs.
3. **Check pending flag register** — hardware still pending after ISR?
4. **Logic analyzer** — correlate edges with ISR count.
5. **LLDB** — breakpoint in main, not ISR, for routine debugging.

---

## Performance Tips

| Tip | Detail |
|-----|--------|
| Short ISR | < 1 µs ideal for fast peripherals |
| Priority grouping | Critical sensors > background |
| WFI in idle main | Saves power |
| Direct register clear | Faster than HAL call in ISR |
| Avoid Swift heavy types in ISR | No String, no allocation |

---

## Exercises

### Exercise 1: ISR Counter

Log `eventCount` every second — compare with button presses.

### Exercise 2: Rising vs Falling

Try both edges; explain which works for your button circuit.

### Exercise 3: Actor Debounce (Host)

Implement `ButtonDebouncer` actor; unit test on macOS with simulated timestamps.

### Exercise 4: Two IRQ Sources

Button + timer tick — prioritize timer if both fire.

### Exercise 5: Critical Section

Implement `withInterruptsDisabled { }` wrapping a non-atomic counter increment.

---

## References

- [ARM Cortex-M4 Generic User Guide — NVIC](https://developer.arm.com/documentation/dui0553/latest/)
- [ESP32-S3 TRM — Interrupt Matrix](https://www.espressif.com/sites/default/files/documentation/esp32-s3_technical_reference_manual_en.pdf)
- [Swift Atomics Package](https://github.com/apple/swift-atomics)
- [Lesson 10 — Timers](./10-timers.md)
- [Embedded Rust interrupts](../learning/09-interrupts.md)

---

*Previous: [08-gpio.md](./08-gpio.md) | Next: [10-timers.md](./10-timers.md)*
