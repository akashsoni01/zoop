# Lesson 23 — Real-Time Patterns (Interrupt-Driven Concurrency)

**Prerequisites:** [09-interrupts.md](./09-interrupts.md), [02-no-stdlib.md](./02-no-stdlib.md)

**Board focus:** [STM32](./boards/stm32.md), [RP2040](./boards/rp2040.md). ESP32-S3 uses interrupt matrix (not ARM NVIC).

**Maturity note:** This lesson is the **RTIC equivalent** for Embedded Swift — **priority-based interrupt-driven concurrency** without a full async runtime. RTIC itself is Rust/Cortex-M specific; Swift embedded uses **manual critical sections**, **IRQ deferral queues**, and **priority ceilings**. On ESP32 (Xtensa/RISC-V), patterns transfer conceptually but NVIC-specific APIs differ.

---

## Theory

Real-time firmware coordinates **interrupt service routines (ISRs)** and **main-loop / task** code safely. Goals:

| Goal | Technique |
|------|-----------|
| **Low ISR latency** | Minimal work in ISR |
| **No data races** | Critical sections, actors, lock-free queues |
| **Priority ordering** | Higher urgency IRQ preempts lower |
| **Deterministic timing** | Hardware timers, not `sleep` guesses |

### Core concepts (RTIC analog)

| RTIC concept | Embedded Swift equivalent |
|--------------|---------------------------|
| Init | `@main` setup — split resources |
| Task (ISR) | `@_cdecl("irq_handler")` or HAL IRQ callback |
| Shared resource | `actor` or `Mutex` with critical section |
| Local resource | Per-module `private var` only ISR touches |
| Monotonic timer | Hardware timer — [10-timers.md](./10-timers.md) |

### When to choose interrupt-driven vs async

| Interrupt-driven excels | Consider Swift Concurrency instead |
|-------------------------|-------------------------------------|
| Sub-millisecond ISR latency | TCP/USB/BLE stacks |
| Button/encoder edge response | Many I/O waits |
| Small firmware, no heap | Complex networking — [22-swift-concurrency.md](./22-swift-concurrency.md) |

---

## Hardware Overview

### ARM Cortex-M (STM32, RP2040)

**NVIC** (Nested Vectored Interrupt Controller):

```
Hardware IRQ ──► ISR (priority N)
                      │
                      ├── critical section lock
                      └── enqueue event for main loop
```

### ESP32-S3

**Interrupt matrix** — peripheral sources routed to CPU exceptions. Wi-Fi/BT IRQs mostly opaque inside binary stack.

---

## ASCII Wiring

Button interrupt + LED (STM32 Nucleo-F411RE):

```
Nucleo-F411RE
┌──────────────────┐
│ PA5 ──► User LED │
│ PC13 ◄─ Button   │  (active low)
│ GND ─────────────│
└──────────────────┘
```

ESP32-S3 equivalent: GPIO0 button, GPIO48 LED — [boards/esp32-s3.md](./boards/esp32-s3.md).

---

## Memory & Register Notes

### Static allocation for IRQ queues

```swift
// Fixed-capacity queue — safe for ISR → main communication
struct IRQQueue {
    private var buffer: [UInt8] = Array(repeating: 0, count: 64)
    private var head: Int = 0
    private var tail: Int = 0

    mutating func pushFromISR(_ byte: UInt8) -> Bool {
        let next = (tail + 1) % buffer.count
        guard next != head else { return false }
        buffer[tail] = byte
        tail = next
        return true
    }

    mutating func pop() -> UInt8? {
        guard head != tail else { return nil }
        let byte = buffer[head]
        head = (head + 1) % buffer.count
        return byte
    }
}
```

Mark ISR entry points clearly; document reentrancy rules.

---

## HAL Swift Example — Button IRQ + LED (ESP-IDF)

```swift
import ESPIDF

// Shared between ISR and main — protect with critical section
var buttonPressed = false

@_cdecl("gpio_isr_handler")
func gpioISRHandler(_ arg: UnsafeMutableRawPointer?) {
    // Minimal ISR: set flag only
    buttonPressed = true
}

@main
struct InterruptApp {
    static func main() {
        let led = GPIO(48, mode: .output)
        let button = GPIO(0, mode: .input, pull: .up)

        button.installISR(handler: gpioISRHandler, edge: .falling)

        while true {
            if buttonPressed {
                buttonPressed = false
                led.toggle()
                print("Button pressed\r\n")
            }
            // WFI sleep until next IRQ — [24-low-power.md](./24-low-power.md)
            waitForInterrupt()
        }
    }
}
```

---

## Bare-Metal Swift — Priority Ceiling Pattern

Illustrative lock token pattern (RTIC-style):

```swift
struct SharedSensorData {
    private var value: Int32 = 0
    private var ceilingPriority: UInt8 = 3
}

struct LockToken<Priority: RawRepresentable> where Priority.RawValue == UInt8 {}

func withLock<P>(_ data: inout SharedSensorData,
                 priority: P,
                 _ body: (inout Int32) -> Void)
where P: RawRepresentable, P.RawValue == UInt8 {
    // Raise base priority to ceiling, run body, restore
    disableInterruptsBelow(data.ceilingPriority)
    body(&data.value)
    restoreInterrupts()
}
```

On production code, use HAL critical section macros until Swift gains compile-time priority tokens.

---

## STM32-Style Vector (illustrative bare-metal)

```swift
import MMIO

var uartRxQueue = IRQQueue()

@_cdecl("USART2_IRQHandler")
func usart2IRQHandler() {
    let usart = USART2(baseAddress: 0x4000_4400)
    while usart.sr.rxne.get() {
        let byte = UInt8(truncatingIfNeeded: usart.dr.get())
        _ = uartRxQueue.pushFromISR(byte)
    }
}
```

---

## Common Mistakes

| Mistake | Symptom | Fix |
|---------|---------|-----|
| Heavy work in ISR | Jitter, missed bytes | Defer to main loop |
| Non-reentrant HAL call in ISR | Deadlock | Document HAL IRQ safety |
| Unprotected shared state | Random corruption | Critical section or lock-free queue |
| Wrong IRQ priority | Starvation | Button > UART > background |
| Floating-point in ISR | Slow context save | Avoid on Cortex-M without FPU lazy stacking |
| `print` in ISR | Hang / crash | Set flag; print in main |

---

## Debugging Tips

- Toggle **debug GPIO** at ISR entry — measure latency on scope.
- Count **overruns** in IRQ queue — resize or process faster.
- Use **logic analyzer** on button + LED — verify debounce timing.
- STM32: **probe-rs** halt and inspect NVIC registers — [25-debugging.md](./25-debugging.md).

---

## Performance Tips

- **Defer everything** except clear hardware flag and enqueue.
- Use **DMA** to shrink interrupt rate — [14-dma.md](./14-dma.md).
- Assign **NVIC priorities** explicitly at init — don't rely on defaults.
- **Debouncing** in software timer task, not GPIO ISR storm.

---

## Exercises

1. **Debounce:** Button IRQ sets flag; timer task debounces 20 ms before toggling LED.
2. **UART queue:** ISR enqueues bytes; main loop echoes — [15-uart.md](./15-uart.md).
3. **Priority demo:** Two timers at different rates; verify preemption with GPIO toggles.
4. **Lock-free ring:** Implement SPSC ring buffer; unit test on host — [26-testing.md](./26-testing.md).
5. **ESP32 matrix:** Map GPIO interrupt source; log which peripheral fired.

---

## References

- [09-interrupts.md](./09-interrupts.md)
- [ARM Cortex-M NVIC documentation](https://developer.arm.com/documentation/dui0497/a/the-cortex-m3-processor/nested-vectored-interrupt-controller)
- [ESP32-S3 TRM — Interrupt Matrix](https://www.espressif.com/en/products/socs/esp32-s3)
- [22-swift-concurrency.md](./22-swift-concurrency.md)

---

*Previous: [22-swift-concurrency.md](./22-swift-concurrency.md) · Next: [24-low-power.md](./24-low-power.md)*
