# Lesson 10: Timers — Hardware Timing and Delays

Accurate timing is essential for debouncing, scheduling, timeouts, and PWM. **Hardware timers** count clock cycles independently of the CPU — replacing inaccurate busy-loop delays.

**Prerequisites:** [09-interrupts.md](./09-interrupts.md)  
**Next:** [11-pwm.md](./11-pwm.md)  
**See also:** [12-adc.md](./12-adc.md), [glossary.md](./glossary.md)

---

## Theory

### Why Not Busy-Wait?

```swift
// Bad for production — CPU burns power, timing drifts with interrupts
func busyWait(ms: UInt32) {
    for _ in 0..<(ms * 240_000) { }
}
```

Problems:

- CPU cannot sleep or do other work
- ISR jitter affects duration
- Constant depends on clock speed and optimization

### Hardware Timer Basics

| Term | Meaning |
|------|---------|
| **Prescaler** | Divides input clock before counter |
| **Period / ARR** | Auto-reload value — counter resets at this count |
| **Compare (CCR)** | Match value for output compare / PWM |
| **Tick** | One counter increment |

```
Timer frequency = CPU_CLK / (prescaler + 1) / (period + 1)

Example: 80 MHz / 80 / 1000 = 1 kHz (1 ms tick)
```

### Timer Modes

| Mode | Use |
|------|-----|
| **One-shot** | Single delay then stop |
| **Periodic** | Repeated interrupts at fixed rate |
| **Input capture** | Measure pulse width |
| **Output compare** | Toggle pin or trigger PWM |

### SysTick (ARM Cortex-M)

Built-in 24-bit countdown timer — common for RTOS tick and `delayMs`:

```swift
// Conceptual SysTick setup for 1 ms tick at 84 MHz
// reload = 84000 - 1
```

### Swift Time Types

On embedded, avoid `Date` and `Duration` if Foundation unavailable. Use raw milliseconds:

```swift
struct Milliseconds: Equatable {
    var rawValue: UInt32
}

static var systemMs: ManagedAtomic<UInt32> = ManagedAtomic(0)

// Timer ISR increments systemMs
```

---

## Hardware Overview

### ESP32-S3 Timers

- Multiple timer groups (TIMG0, TIMG1)
- 64-bit counters, configurable prescalers
- Used for RTOS tick in ESP-IDF (C) — Swift ports wrap similarly

### STM32 TIM2

- 32-bit general-purpose timer on APB1
- Prescaler, auto-reload, update interrupt

### RP2040 Timer

- 64-bit microsecond timer at fixed 1 MHz
- `time_us_64()` style access in SDK — Swift wraps via MMIO

---

## Wiring Diagram

Timers are internal — no external wiring for basic tick:

```
CPU Clock ──► Prescaler ──► Counter ──► Compare match ──► IRQ
                                              │
                                              └──► (optional) GPIO toggle

External wiring only if routing timer output to a pin (PWM — Lesson 11).
```

Optional lab: route timer output compare to GPIO header; observe square wave on scope.

---

## Memory & Register Explanation

Timer registers are MMIO — typical layout (STM32-style):

| Offset | Register | Purpose |
|--------|----------|---------|
| 0x00 | CR1 | Enable, direction |
| 0x10 | DIER | Interrupt enable |
| 0x24 | SR | Status flags (clear on write) |
| 0x28 | EGR | Generate update |
| 0x2C | CCMR | Channel mode |
| 0x34 | CCER | Channel enable |
| 0x84 | PSC | Prescaler |
| 0x88 | ARR | Auto-reload / period |

```swift
struct TimerRegisters {
    var cr1: UInt32
    var reserved: (UInt32, UInt32, UInt32)
    var dier: UInt32
    // ... padded to match reference manual
    var psc: UInt32
    var arr: UInt32
}
```

---

## HAL-Style Swift Implementation

```swift
public protocol PeriodicTimer {
    mutating func start(periodMs: UInt32) -> Result<Void, TimerError>
    mutating func stop() -> Result<Void, TimerError>
    func elapsedMs() -> UInt32
}

public enum TimerError: Error {
    case invalidPeriod
    case alreadyRunning
    case hardwareFault
}

// Global tick — incremented in ISR
static var tickMs: ManagedAtomic<UInt32> = ManagedAtomic(0)

@_cdecl("TIM2_IRQHandler")
func tim2Handler() {
    tickMs.wrappingIncrement(ordering: .relaxed)
    timerClearUpdateInterrupt()
}

struct Stm32PeriodicTimer: PeriodicTimer {
    mutating func start(periodMs: UInt32) -> Result<Void, TimerError> {
        guard periodMs > 0 else { return .failure(.invalidPeriod) }
        timerConfigure(prescaler: 8399, period: periodMs - 1)  // 84 MHz example
        timerEnableUpdateInterrupt()
        nvicEnableTIM2()
        return .success(())
    }

    mutating func stop() -> Result<Void, TimerError> {
        timerDisable()
        return .success(())
    }

    func elapsedMs() -> UInt32 {
        tickMs.load(ordering: .relaxed)
    }
}

// Delay using system tick
struct SysTickDelay: DelayMs {
    mutating func delayMs(_ ms: UInt32) {
        let start = tickMs.load(ordering: .relaxed)
        while tickMs.load(ordering: .relaxed) &- start < ms {
            waitForInterrupt()
        }
    }
}
```

Application — stopwatch:

```swift
@main
struct TimerApp {
    static func main() {
        var timer = Stm32PeriodicTimer()
        _ = timer.start(periodMs: 1)  // 1 ms tick

        var led = ESP32GpioOutputPin(pin: 2, initial: .low)
        var lastToggle = timer.elapsedMs()

        while true {
            let now = timer.elapsedMs()
            if now &- lastToggle >= 500 {
                _ = led.toggle()
                lastToggle = now
            }
        }
    }
}
```

---

## Bare-Metal / MMIO Swift Notes

STM32 TIM2 bare setup (conceptual addresses):

```swift
let TIM2_BASE: UInt = 0x4000_0000
let RCC_APB1ENR: UInt = 0x4002_383C  // Enable TIM2 clock

func tim2Init1msTick() {
    // Enable TIM2 clock
    reg(RCC_APB1ENR).pointee |= 1 << 0

    let tim = UnsafeMutablePointer<TimerRegisters>(bitPattern: TIM2_BASE)!
    tim.pointee.psc = 8399       // 84 MHz / 8400 = 10 kHz
    tim.pointee.arr = 9          // 10 kHz / 10 = 1 kHz = 1 ms
    tim.pointee.dier = 1         // Update interrupt enable
    tim.pointee.cr1 = 1          // Counter enable
}

@_cdecl("TIM2_IRQHandler")
func tim2Isr() {
    tickMs.wrappingIncrement(ordering: .relaxed)
    reg(TIM2_BASE + 0x10).pointee &= ~1  // Clear UIF in SR — check TRM
}
```

---

## Step-by-Step Explanation

### Step 1: Enable Timer Clock

Peripherals are clock-gated — enable in RCC (STM32) or PERIP_CLK (ESP32).

### Step 2: Calculate Prescaler and Period

Target tick rate from CPU frequency. Document your math.

### Step 3: Configure Update Interrupt

Enable UI interrupt in DIER equivalent.

### Step 4: Install ISR

Increment global millisecond counter.

### Step 5: Start Counter

Set enable bit in CR1.

### Step 6: Replace Busy-Wait

Refactor [08-gpio.md](./08-gpio.md) debounce to use `SysTickDelay`.

### Step 7: Verify with Logic Analyzer

Optional: output compare to GPIO for visible frequency.

---

## Common Mistakes

| Mistake | Symptom | Fix |
|---------|---------|-----|
| Timer clock not enabled | Counter frozen | Enable RCC bit |
| Wrong prescaler math | Wrong tick rate | Recalculate |
| Forgot clear interrupt flag | Immediate re-IRQ | Clear status in ISR |
| 16-bit overflow | Short max period | Use 32-bit timer |
| Read ARR before shadow update | Glitch | Force update event |
| Unsigned wrap confusion | Long delays fail | Use `&-` for elapsed |

---

## Debugging Tips

1. **Toggle GPIO in timer ISR briefly** — verify frequency with scope.
2. **Log tickMs** — should increment steadily.
3. **Halt in debugger** — read counter register live.
4. **Compare with known-good C example** — register values match?
5. **Measure LED blink** — 500 ms toggle = 1 Hz blink.

---

## Performance Tips

| Tip | Detail |
|-----|--------|
| One system tick timer | Share tickMs across app |
| WFI in delay loops | Sleep between tick checks |
| 32-bit timers for long intervals | Avoid wrap issues |
| Avoid floating point in calc | Integer prescaler math |
| Coalesce timer ISRs | One tick for many software timers |

---

## Exercises

### Exercise 1: 1 ms Tick

Configure 1 ms system tick; log uptime every second.

### Exercise 2: Replace Busy-Wait

Port GPIO blink from busy-wait to tick-based timing.

### Exercise 3: Stopwatch

Button starts/stops incrementing counter displayed via UART.

### Exercise 4: Prescaler Calculation

Given 240 MHz CPU, compute prescaler/ARR for 10 ms tick.

### Exercise 5: Software Timer Array

Implement four virtual timers off one hardware 1 ms tick.

---

## References

- [STM32 RM0383 — General-purpose timers](https://www.st.com/resource/en/reference_manual/rm0383-stm32f411-advanced-armbased-32bit-mcus-stmicroelectronics.pdf)
- [ESP32-S3 TRM — Timer Group](https://www.espressif.com/sites/default/files/documentation/esp32-s3_technical_reference_manual_en.pdf)
- [RP2040 Datasheet — Timer](https://datasheets.raspberrypi.com/rp2040/rp2040-datasheet.pdf)
- [Lesson 11 — PWM](./11-pwm.md)
- [Embedded Rust timers](../learning/10-timers.md)

---

*Previous: [09-interrupts.md](./09-interrupts.md) | Next: [11-pwm.md](./11-pwm.md)*
