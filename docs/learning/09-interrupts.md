# Lesson 09: Interrupts — NVIC, ISRs, and Debouncing

**Interrupts** let hardware notify software immediately when events occur — button presses, timer ticks, UART data arrival — without constant polling.

**Prerequisites:** [08-gpio.md](./08-gpio.md)  
**Next:** [10-timers.md](./10-timers.md)  
**See also:** [01-rust-basics.md](./01-rust-basics.md), [glossary.md](./glossary.md)

---

## Theory

### Interrupt Terminology

| Term | Meaning |
|------|---------|
| **IRQ (Interrupt Request)** | Signal from peripheral to interrupt controller |
| **ISR (Interrupt Service Routine)** | Function called when interrupt fires |
| **NVIC (Nested Vectored Interrupt Controller)** | ARM interrupt controller — prioritizes and enables IRQs |
| **Interrupt Vector** | Table of ISR addresses indexed by IRQ number |
| **Priority** | Lower number = higher priority on ARM (usually) |
| **Mask / Enable** | Globally or per-IRQ disable/enable |
| **Critical Section** | Code region where interrupts are disabled |

ESP32 uses an **Interrupt Matrix** to route peripheral interrupts to CPU — conceptually similar to NVIC.

### Polling vs Interrupts

| Approach | CPU Usage | Latency | Complexity |
|----------|-----------|---------|------------|
| Polling | High (busy loop) | Variable | Low |
| Interrupts | Low (sleep until event) | Low (microseconds) | Medium |

Use interrupts when:

- Events are infrequent (buttons)
- Timing matters (UART byte received)
- Power saving needed (sleep between events)

---

## Hardware Overview

### ARM Cortex-M Interrupt Flow

```
1. Peripheral asserts IRQ line
2. NVIC checks enable + priority vs current execution
3. CPU finishes current instruction (unless NMI)
4. Hardware pushes registers to stack
5. PC set to vector[IRQn + 16]
6. ISR runs
7. Return from interrupt restores context
```

### ESP32-S3 Specifics

- Dual core — pin interrupt can route to PRO or APP CPU
- **GPIO interrupt** types: rising edge, falling edge, any edge, level high/low
- Rust HAL abstracts `bind_interrupt` patterns

### Button Bounce (Hardware Phenomenon)

Mechanical contacts bounce for 1–20 ms, generating multiple edges. **Debouncing** in software or hardware is mandatory.

---

## Wiring Diagram

Same as [08-gpio.md](./08-gpio.md) — interrupt configuration is software; wiring unchanged.

```
GPIO0 (button, active low, pull-up)
  │
  ├──[ SW ]── GND
  │
  └──► ESP32 GPIO interrupt input (falling edge on press)

GPIO2 (LED output)
  └──[ 220Ω ]──(LED)── GND
```

Optional hardware debounce:

```
GPIO0 ────[ 100nF cap to GND ]────[ SW ]──── GND
         (RC filter reduces bounce — not always sufficient alone)
```

---

## Memory & Register Explanation

### ISR Shared State Problem

Main loop and ISR both accessing `led_state` creates a **data race** without synchronization:

```rust
// UNSAFE PATTERN — data race between ISR and main
static mut LED_ON: bool = false;

// ISR reads/writes LED_ON
// Main also reads/writes LED_ON — undefined behavior in Rust without Sync/atomic
```

Solutions:

| Pattern | Use When |
|---------|----------|
| `AtomicBool`, `AtomicU32` | Simple flags, counters |
| `critical_section::Mutex<RefCell<T>>` | Short critical sections, `no_std` |
| `heapless::spsc::Queue` | ISR produces, main consumes |
| RTIC shared resources | Statically verified locking |

### ESP32 GPIO Interrupt Registers

| Register | Purpose |
|----------|---------|
| `GPIO_PINn_INT_TYPE` | Edge/level sensitivity |
| `GPIO_STATUS` | Interrupt status (read to clear) |
| `GPIO_PINn_REG` | Per-pin config |

---

## HAL Implementation

ESP32-S3 GPIO interrupt with critical section mutex:

```rust
#![no_std]
#![no_main]

use core::cell::RefCell;
use critical_section::Mutex;
use defmt::info;
use esp_hal::{
    clock::ClockControl,
    gpio::{Io, Level, Output, Pull, Input, Event},
    interrupt::software::SoftwareInterruptControl,
    peripherals::Peripherals,
};
use panic_halt as _;

/// Shared state — Mutex ensures exclusive access across ISR and main.
/// `RefCell` provides interior mutability; Mutex provides sync.
static BUTTON_PRESSED: Mutex<RefCell<bool>> = Mutex::new(RefCell::new(false));

#[esp_hal::macros::entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    let mut led = Output::new(io.pins.gpio2, Level::Low);

    // Configure button with interrupt on falling edge
    let mut button = io.pins.gpio0.into_input(Pull::Up);
    button.listen(Event::FallingEdge);

    // Bind interrupt handler — platform-specific macro
    // Handler must be 'static — lives for entire program
    button.set_interrupt_handler(gpio0_handler);

    let mut led_on = false;

    loop {
        // Check shared flag inside critical section
        let pressed = critical_section::with(|cs| {
            let flag = BUTTON_PRESSED.borrow_ref(cs);
            *flag
        });

        if pressed {
            critical_section::with(|cs| {
                *BUTTON_PRESSED.borrow_ref_mut(cs) = false;
            });

            led_on = !led_on;
            if led_on {
                led.set_high().ok();
            } else {
                led.set_low().ok();
            }
            info!("Toggled via interrupt");
        }

        // Sleep until interrupt — saves power
        core::hint::spin_loop();
    }
}

/// ISR — keep SHORT: set flag, clear interrupt status, return.
#[esp_hal::macros::handler]
fn gpio0_handler() {
    critical_section::with(|cs| {
        *BUTTON_PRESSED.borrow_ref_mut(cs) = true;
    });
    // HAL typically clears interrupt status in handler binding
}
```

STM32 with `cortex-m-rt`:

```rust
use cortex_m::interrupt;

static DEBOUNCE: Mutex<RefCell<u32>> = Mutex::new(RefCell::new(0));

#[interrupt]
fn EXTI15_10() {
    critical_section::with(|cs| {
        *DEBOUNCE.borrow_ref_mut(cs) += 1;
    });
}
```

### Software Debounce in Main Loop

```rust
const DEBOUNCE_MS: u32 = 50;

struct Debouncer {
    last_change_ms: u32,
    stable: bool,
}

impl Debouncer {
    fn update(&mut self, raw: bool, now_ms: u32) -> Option<bool> {
        if raw != self.stable {
            if now_ms - self.last_change_ms >= DEBOUNCE_MS {
                self.stable = raw;
                return Some(self.stable);
            }
        } else {
            self.last_change_ms = now_ms;
        }
        None
    }
}
```

Requires millisecond tick from [10-timers.md](./10-timers.md).

---

## Bare-Metal Implementation

Conceptual ARM EXTI enable (STM32-style):

```rust
const SYSCFG_BASE: u32 = 0x4001_3800;
const EXTI_BASE: u32 = 0x4001_3C00;
const NVIC_ISER: u32 = 0xE000_E100;

unsafe fn enable_exti_line13() {
    // 1. Connect PC13 to EXTI line 13 via SYSCFG EXTICR
    // 2. Configure EXTI for falling edge
    // 3. Unmask EXTI line
    // 4. Enable NVIC IRQ for EXTI15_10

    let iser = NVIC_ISER as *mut u32;
    write_volatile(iser, read_volatile(iser) | (1 << (40 - 32))); // IRQ 40
}

#[no_mangle]
pub extern "C" fn EXTI15_10() {
    // Read EXTI_PR to see which line fired
    // Clear pending bit by writing 1
    // Set atomic flag
}
```

On ESP32, raw GPIO interrupt setup spans many registers — strongly prefer HAL for correctness.

---

## Step-by-Step Explanation

### Step 1: Configure GPIO Input (Same as Polling)

Pull-up, input mode — [08-gpio.md](./08-gpio.md).

### Step 2: Select Interrupt Trigger

- Button active low → **falling edge** on press
- Release → rising edge (optional separate handler)

### Step 3: Write Minimal ISR

Set a flag only — no `defmt` in ISR initially (logging can be slow).

### Step 4: Enable Interrupt in NVIC/Matrix

HAL function or PAC register write.

### Step 5: Process Flag in Main Loop

Toggle LED, debounce timestamps, clear flag atomically.

### Step 6: Add Debouncing

Time-based debounce in main loop using timer tick.

### Step 7: Test with Logic Analyzer

Verify single edge per press after debounce.

---

## Common Mistakes

| Mistake | Symptom | Fix |
|---------|---------|-----|
| Long ISR | Missed interrupts, jitter | Defer work to main |
| `defmt` in ISR | Overrun, crashes | Log in main only |
| Data race on shared state | Random behavior | Atomics or Mutex |
| Wrong edge polarity | Interrupt on release | Match active level |
| No debounce | Multiple toggles per press | 20–50 ms debounce |
| Forgetting to clear pending | Interrupt storm | Clear status register |
| Stack overflow in nested IRQ | HardFault | Increase stack, shorten ISRs |

---

## Debugging Tips

1. **Count ISR fires** with `AtomicU32` — compare to button presses.
2. **Toggle debug pin in ISR** — scope shows ISR latency.
3. **Disable interrupts temporarily** — isolate polling vs IRQ issues.
4. **`panic-probe` + breakpoint in ISR** — use sparingly.
5. **Check priority inversion** — high-rate IRQ starving others.

---

## Performance Tips

| Tip | Rationale |
|-----|-----------|
| Keep ISR < 10 µs | Real-time responsiveness |
| Use `WFI` sleep in main | Wake on any interrupt |
| Atomic flags over Mutex when possible | Lower overhead |
| Batch GPIO interrupt status read | One ISR for multiple pins |
| Hardware debounce for noisy environments | Reduces CPU load |

---

## Exercises

### Exercise 1: ISR Counter

Log `AtomicU32` interrupt count vs accepted button events after debounce.

### Exercise 2: Both Edges

Fire interrupt on press and release; measure duration between edges.

### Exercise 3: Critical Section Timing

Measure max critical section duration — should stay sub-microsecond for simple flags.

### Exercise 4: STM32 Port

Implement equivalent EXTI handler on Nucleo PC13 button.

### Exercise 5: Queue Pattern

Use `heapless::spsc::Queue` — ISR pushes events, main pops and processes.

---

## References

- [Cortex-M Generic User Guide — Exceptions](https://developer.arm.com/documentation/dui0552/latest/)
- [ESP32-S3 TRM — Interrupt Matrix](https://www.espressif.com/sites/default/files/documentation/esp32-s3_technical_reference_manual_en.pdf)
- [critical-section crate](https://docs.rs/critical-section/)
- [The Embedded Rust Book — Interrupts](https://docs.rust-embedded.org/book/concurrency/index.html)
- [Lesson 10 — Timers](./10-timers.md)

---

*Previous: [08-gpio.md](./08-gpio.md) | Next: [10-timers.md](./10-timers.md)*
