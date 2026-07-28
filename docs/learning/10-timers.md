# Lesson 10: Timers — SysTick, Hardware Timers, and Delays

Accurate timing is essential for debouncing, scheduling, PWM, and timeouts. This lesson covers **SysTick**, general-purpose hardware timers, busy-wait pitfalls, and periodic interrupts.

**Prerequisites:** [09-interrupts.md](./09-interrupts.md)  
**Next:** [11-pwm.md](./11-pwm.md)  
**See also:** [09-interrupts.md](./09-interrupts.md), [03-memory-layout.md](./03-memory-layout.md)

---

## Theory

### Why Not Busy-Wait?

```rust
for _ in 0..500_000 { core::hint::spin_loop(); }  // "Delay"
```

Problems:

- Cycle count varies with CPU frequency, compiler opts, cache
- CPU burns power doing nothing
- Blocks all other work

**Hardware timers** count at a known clock rate independent of CPU load (mostly).

### Timer Types

| Timer | Platform | Typical Use |
|-------|----------|-------------|
| **SysTick** | ARM Cortex-M | 1 ms OS tick, HAL delays |
| **GPTimer** | ESP32-S3 | General purpose, 64-bit, alarms |
| **LEDC timer** | ESP32 | PWM timebase ([11-pwm.md](./11-pwm.md)) |
| **TIM2–TIM5** | STM32 | 32-bit general timers |
| **Timer** | RP2040 | 64-bit, lazy alarm chaining |

### Timer Modes

| Mode | Behavior |
|------|----------|
| **One-shot** | Count to match value, stop, optionally interrupt |
| **Periodic (auto-reload)** | Reset and repeat — system tick |
| **Input capture** | Timestamp external edge |
| **Output compare** | Toggle pin at match value |

---

## Hardware Overview

### ESP32-S3 Timer Subsystem

- Multiple **GPTimer** groups (Timer Group 0, 1, etc.)
- Clock source: APB (typically 80 MHz) with prescaler
- Alarms generate interrupts at compare match

```
Timer clock = APB_CLK / prescaler
Period = (alarm_value + 1) / timer_clock
```

### ARM SysTick

Built into every Cortex-M core:

```
SysTick counts down from LOAD to 0 at CPU clock / (LOAD+1) divider
CTRL register: ENABLE, TICKINT, CLKSOURCE
```

STM32 HAL delay often uses SysTick at 1 kHz.

### RP2040 Timer

Single 64-bit counter; alarms are **lazy** — hardware schedules next interrupt efficiently.

---

## Wiring Diagram

Timers are internal — no external wiring for basic lessons.

Optional external verification:

```
Timer output compare pin ──► Logic analyzer CH0
                             (observe square wave at expected frequency)
```

For LED heartbeat:

```
GPIO2 ──[220Ω]──(LED)── GND    (toggled every 500 ms by timer ISR)
```

---

## Memory & Register Explanation

### Timer Registers (Generic Pattern)

| Register | Purpose |
|----------|---------|
| `CTRL` | Enable, interrupt enable, mode |
| `LOAD` / `AUTO_RELOAD` | Period value |
| `COUNT` / `COUNTER` | Current count |
| `STATUS` | Interrupt pending (often write-1-to-clear) |

ESP32 GPTimer example addresses in TRM section "Timer Group".

### Timekeeping in Software

```rust
use core::sync::atomic::{AtomicU32, Ordering};

/// Milliseconds since boot — incremented in timer ISR.
/// Atomic: safe to read from main without critical section on single-core.
static MS_TICK: AtomicU32 = AtomicU32::new(0);

pub fn millis() -> u32 {
    MS_TICK.load(Ordering::Relaxed)
}

// In timer ISR:
// MS_TICK.fetch_add(1, Ordering::Relaxed);
```

`Ordering::Relaxed` sufficient for monotonic tick counter on single-core MCU.

---

## HAL Implementation

ESP32-S3 periodic 1 ms tick with GPTimer:

```rust
use esp_hal::timer::timg::TimerGroup;
use esp_hal::peripherals::Peripherals;
use core::sync::atomic::{AtomicU32, Ordering};

static MS: AtomicU32 = AtomicU32::new(0);

#[esp_hal::macros::entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let clocks = /* ... clock init ... */;

    let timg0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    let mut timer0 = timg0.timer0;

    // Configure 1 ms period — API varies by esp-hal version; consult docs
    timer0.set_alarm_active(true);
    timer0.listen();

    timer0.set_interrupt_handler(timer0_isr);

    let mut led = /* ... GPIO init ... */;
    let mut last_toggle = 0u32;

    loop {
        let now = MS.load(Ordering::Relaxed);

        // Non-blocking 500 ms blink using tick counter
        if now.wrapping_sub(last_toggle) >= 500 {
            last_toggle = now;
            led.toggle().ok();
        }

        // Could use wait_for_interrupt() for lower power
    }
}

#[esp_hal::macros::handler]
fn timer0_isr() {
    MS.fetch_add(1, Ordering::Relaxed);
    // Clear interrupt flag via HAL
}
```

STM32 SysTick delay:

```rust
use cortex_m::peripheral::SYST;
use stm32f4xx_hal::delay::Delay;

let mut delay = Delay::new(cp.SYST, &clocks);
delay.delay_ms(500u32);  // Blocks — OK for init, not for concurrent tasks
```

### Non-Blocking Delay Pattern

```rust
struct Timer {
    deadline_ms: u32,
}

impl Timer {
    fn new(duration_ms: u32) -> Self {
        Self {
            deadline_ms: millis().wrapping_add(duration_ms),
        }
    }

    fn expired(&self) -> bool {
        millis().wrapping_sub(self.deadline_ms) < u32::MAX / 2
        // Wrapping arithmetic trick for deadline comparison
    }
}
```

---

## Bare-Metal Implementation

ARM SysTick 1 ms init (conceptual):

```rust
const SYSTICK_BASE: u32 = 0xE000_E010;
const SYST_CSR: u32 = SYSTICK_BASE;
const SYST_RVR: u32 = SYSTICK_BASE + 4;
const SYST_CVR: u32 = SYSTICK_BASE + 8;

/// Initialize SysTick for 1 kHz tick at `cpu_hz`.
unsafe fn systick_init_1khz(cpu_hz: u32) {
    let reload = cpu_hz / 1000 - 1;
    core::ptr::write_volatile(SYST_RVR as *mut u32, reload);
    core::ptr::write_volatile(SYST_CVR as *mut u32, 0);
    // ENABLE | TICKINT | CLKSOURCE (processor clock)
    core::ptr::write_volatile(SYST_CSR as *mut u32, 0b111);
}

#[no_mangle]
pub extern "C" fn SysTick() {
    MS.fetch_add(1, Ordering::Relaxed);
}
```

ESP32 GPTimer bare-metal requires configuring prescaler, alarm value, and interrupt matrix — dozens of registers. Use HAL unless profiling demands otherwise.

---

## Step-by-Step Explanation

### Step 1: Determine Timer Clock Source

Read TRM / clock tree — often APB frequency after prescalers.

### Step 2: Calculate Prescaler and Period

```
desired_hz = 1000  (1 kHz)
timer_clk = 80_000_000
prescaler = timer_clk / 1_000_000 - 1  → 1 MHz tick
alarm = 1000 - 1  → 1 ms period
```

### Step 3: Enable Timer Interrupt

Bind ISR, set priority, unmask in NVIC.

### Step 4: Maintain Global Tick Counter

Atomic increment in ISR.

### Step 5: Replace Busy-Wait Delays

Use `millis()` comparisons in main loop.

### Step 6: Verify with LED or Logic Analyzer

500 ms toggle should match within timer resolution.

---

## Common Mistakes

| Mistake | Symptom | Fix |
|---------|---------|-----|
| Wrong clock frequency assumption | Timing off by factor | Measure or read RCC config |
| 32-bit tick overflow confusion | Rare deadline bugs | Use wrapping arithmetic |
| Blocking delay in ISR | System hang | Never delay in ISR |
| Timer not started | No interrupts | Set ENABLE bit |
| Forgetting clear interrupt flag | Immediate re-entry | Write-1-to-clear status |
| SysTick conflict | Double tick rate | One owner of SysTick |

---

## Debugging Tips

1. **Toggle GPIO in timer ISR** — verify frequency on scope.
2. **Log `MS` every second** — detect missed ticks.
3. **Compare `millis()` against wall clock** — long-term drift check.
4. **Disable optimizations temporarily** — isolate compiler timing changes.
5. **Read COUNT register in debugger** — confirm timer running.

---

## Performance Tips

| Tip | Benefit |
|-----|---------|
| One system tick (1 ms) | Shared timebase for all tasks |
| Coalesce periodic work | One timer ISR schedules many soft deadlines |
| Use `WFI` between work | Lower power until tick or other IRQ |
| 64-bit timer for timestamps | Avoid wrap issues (RP2040, ESP32 GPTimer) |
| Hardware timer for PWM | CPU free ([11-pwm.md](./11-pwm.md)) |

---

## Exercises

### Exercise 1: Stopwatch

Button starts/stops a counter displayed via `defmt` every second.

### Exercise 2: Multiple Deadlines

Three different periodic tasks (100 ms, 250 ms, 1 s) from one 1 ms tick.

### Exercise 3: Measure Busy-Wait Error

Compare busy loop vs timer delay at different optimization levels.

### Exercise 4: Timer Capture

Measure pulse width on input pin using input capture mode (if supported).

### Exercise 5: Sleep Mode

Use `WFI` or ESP light sleep between ticks; measure current draw difference.

---

## References

- [ESP32-S3 TRM — Timer Group](https://www.espressif.com/sites/default/files/documentation/esp32-s3_technical_reference_manual_en.pdf)
- [ARM SysTick documentation](https://developer.arm.com/documentation/dui0662/latest/Hardware/System-Control/SysTick-timer)
- [STM32F411 RM — General-purpose timers](https://www.st.com/resource/en/reference_manual/rm0383-stm32f411-advanced-armbased-32bit-mcus-stmicroelectronics.pdf)
- [RP2040 Datasheet — Timer](https://datasheets.raspberrypi.com/rp2040/rp2040-datasheet.pdf)
- [Lesson 09 — Interrupts](./09-interrupts.md)

---

*Previous: [09-interrupts.md](./09-interrupts.md) | Next: [11-pwm.md](./11-pwm.md)*
