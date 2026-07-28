# Lesson 23 — RTIC (Real-Time Interrupt-driven Concurrency)

**Prerequisites:** [09-interrupts.md](./09-interrupts.md), [02-no-std.md](./02-no-std.md)

**Board focus:** [STM32](./boards/stm32.md), [RP2040](./boards/rp2040.md). ESP32-S3 notes where RTIC support differs.

---

## Theory

**RTIC** (Real-Time Interrupt-driven Concurrency, formerly `cortex-m-rtfm`) is a **`no_std` framework** for building real-time firmware using Rust's type system to prove **shared resource safety** at compile time.

Core ideas:

| Concept | Meaning |
|---------|---------|
| **Init** | One-time setup — returns resources split to tasks |
| **Task** | Function triggered by interrupt or software dispatch |
| **Shared resource** | Data accessed by multiple tasks with **priority ceiling** locking |
| **Local resource** | Owned by single task — zero locking overhead |
| **Monotonic** | Free-running timer for periodic scheduling |

### Priority rules

Higher numeric priority = **more urgent**. RTIC ensures:

- No priority inversion via **priority ceiling mutex**
- No data races — borrow checker + lock tokens at compile time
- Minimal runtime overhead — no dynamic allocator

### When to choose RTIC

| RTIC excels | Consider Embassy instead |
|-------------|--------------------------|
| Hard real-time ISR latency | Many async I/O waits |
| Deterministic microsecond jitter | TCP/USB stacks |
| Small firmware, no heap | Complex networking — [22-embassy.md](./22-embassy.md) |

RTIC targets **ARM Cortex-M** primarily. ESP32 (Xtensa/RISC-V) uses different concurrency paths — RTIC is not the default there.

---

## Hardware Overview

RTIC integrates with **NVIC** (Nested Vectored Interrupt Controller) on Cortex-M:

```
Hardware IRQ ──► RTIC task (priority N)
                      │
                      ├── locks Shared<T> (ceiling priority)
                      └── accesses Local<T> freely
```

Supported via **`cortex-m-rt`** + chip **PAC** — [boards/stm32.md](./boards/stm32.md).

---

## ASCII Wiring

Example: button interrupt + LED (STM32 Nucleo-F411RE):

```
Nucleo-F411RE
┌──────────────────┐
│ PA5 ──► User LED │  (on-board)
│ PC13 ◄─ Button   │  (user button, active low — verify board)
│ GND ─────────────│
└──────────────────┘
```

ESP32-S3 learners: implement equivalent logic with `esp-hal` interrupts or Embassy — patterns below transfer conceptually.

---

## Memory & Register Notes

### Static allocation only

RTIC stores resources in `.bss` / static — no `Box` required:

```rust
#[rtic::app(device = stm32f4xx_hal::stm32::stm32f411, peripherals = true, monotonics = rtic::app::Monotonics)]
mod app {
    #[shared]
    struct Shared {
        serial: Serial<USART2>,
    }

    #[local]
    struct Local {
        click_count: u32,
    }
}
```

### NVIC priorities

Configure in `init` or via HAL:

```rust
// Lower number on some docs = higher priority — STM32 uses 0 as highest subpriority
// Set SysTick lower priority than UART IRQ if UART must preempt
```

Document your chip's **priority bit width** (usually 4 bits on STM32).

---

## HAL Example — RTIC on STM32F411 (button + LED)

```rust
#![no_std]
#![no_main]

use panic_halt as _;
use stm32f4xx_hal::{
    gpio::{Edge, Input, Output, PushPull},
    pac,
    prelude::*,
    serial::{config::Config, Serial},
};

#[rtic::app(device = pac, peripherals = true, monotonics = rtic::app::Monotonics)]
mod app {
    use super::*;
    use rtic_monotonics::systick::prelude::*;

    #[monotonic(binds = SysTick, default = true)]
    type MyMono = Systick<1_000_000>; // 1 MHz tick

    #[shared]
    struct Shared {
        led: Output<PushPull>,
    }

    #[local]
    struct Local {}

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        let dp = ctx.device;
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(84.MHz()).freeze();

        let gpioc = dp.GPIOC.split();
        let button = gpioc.pc13.into_pull_up_input();
        button.set_interrupt(Edge::Falling);

        let gpioa = dp.GPIOA.split();
        let mut led = gpioa.pa5.into_push_pull_output();
        led.set_low();

        // Enable EXTI15_10 interrupt for PC13
        unsafe {
            cortex_m::peripheral::NVIC::unmask(pac::Interrupt::EXTI15_10);
        }

        MyMono::start(ctx.core.SYST, 84_000_000); // sysclk Hz

        (
            Shared { led },
            Local {},
            init::Monotonics(MyMono),
        )
    }

    #[task(binds = EXTI15_10, shared = [led])]
    fn button_irq(ctx: button_irq::Context) {
        ctx.shared.led.lock(|led| {
            led.toggle();
        });
    }

    #[idle]
    fn idle() -> ! {
        loop {
            cortex_m::asm::wfi(); // Wait for interrupt — low power
        }
    }
}
```

`shared.led.lock()` uses **priority ceiling** — compile-time checked.

---

## Software Task + Monotonic Timer

```rust
#[task(local = [count])]
fn periodic(mut ctx: periodic::Context) {
    *ctx.local.count += 1;
    // schedule next invocation
    periodic::spawn_after(500.millis()).ok();
}

#[init]
fn init(...) -> (...) {
    periodic::spawn_after(1.secs()).ok();
    ...
}
```

Monotonic scheduling without busy-wait — [10-timers.md](./10-timers.md).

---

## Bare-Metal Comparison

Without RTIC, you'd manually:

```rust
static mut LED_STATE: bool = false; // UNSAFE — data race if ISR + main both access

#[interrupt]
fn EXTI15_10() {
    unsafe { LED_STATE = !LED_STATE; } // UB if main also reads
}
```

RTIC replaces this with **provably safe** locks or local ownership.

---

## Step-by-Step

1. Create **`cortex-m-quickstart`** or STM32 RTIC template project.
2. **`cargo add rtic`** + chip HAL + `rtic-monotonics`.
3. Implement **init** — LED + button EXTI.
4. Add **ISR task** toggling LED via `shared`.
5. Add **monotonic periodic task** — blink rate independent of button.
6. Measure **ISR latency** with GPIO toggle + scope.
7. Compare stack usage vs manual mutex with `cargo size`.

---

## Common Mistakes

| Mistake | Symptom | Fix |
|---------|---------|-----|
| Blocking in high-priority ISR | Missed deadlines | Defer work to lower task |
| `lock()` across long work | Jitter | Minimize critical section |
| Wrong EXTI line binding | ISR never fires | Match `binds =` to vector |
| Spawn from wrong context | Compile error | Use `spawn` only where RTIC allows |
| ESP32 RTIC attempt | Crate unsupported | Use Embassy/esp-hal on ESP |
| Floating button pin | Spurious IRQ | Enable pull-up/down |

---

## Debugging Tips

- **`cargo build` errors** often explain priority conflicts — read fully.
- Use **`defmt`** in tasks — know which ISR ran — [25-debugging.md](./25-debugging.md).
- Probe **stack overflow** with `_stack_start` symbols + GDB watchpoint.
- Temporarily **disable WFI** in idle if debugger detaches.
- Logic analyzer on ISR pin toggle — measure worst-case latency.

---

## Performance Tips

- Keep ISRs **< 10 µs** when possible — defer parsing to software tasks.
- Use **`#[task(priority = N)]`** explicitly for scheduling intent.
- **Local resources** avoid mutex cost entirely.
- **`wfi` in idle** saves power vs spin — [24-low-power.md](./24-low-power.md).
- RTIC has **zero-cost** abstractions vs manual critical sections — trust compiler.

---

## Exercises

1. **Debounce:** Timer task confirms button stable 20 ms before toggle.
2. **Producer/consumer:** UART ISR pushes bytes to `HeaplessQueue`; main task drains (shared).
3. **Priority demo:** High-priority task preempts low — GPIO trace proves order.
4. **RP2040:** Port to `rp2040-hal` if RTIC support available in your version.
5. **Compare Embassy:** Same button/LED in [22-embassy.md](./22-embassy.md) — note code size.

---

## References

- [RTIC Book](https://rtic.rs/)
- [rtic crate docs](https://docs.rs/rtic/)
- [cortex-m-quickstart](https://github.com/rust-embedded/cortex-m-quickstart)
- [boards/stm32.md](./boards/stm32.md)
- [09-interrupts.md](./09-interrupts.md)

---

*Previous: [22-embassy.md](./22-embassy.md) · Next: [24-low-power.md](./24-low-power.md)*
