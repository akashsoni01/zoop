# Lesson 22 — Embassy: Async Embedded Rust

**Prerequisites:** [09-interrupts.md](./09-interrupts.md), [02-no-std.md](./02-no-std.md), basic Rust `async`/`await`

**Board focus:** [ESP32-S3](./boards/esp32-s3.md), [boards/stm32.md](./boards/stm32.md), [boards/rp2040.md](./boards/rp2040.md)

---

## Theory

**Async** (asynchronous) programming lets firmware **wait for events without blocking the CPU** in a busy loop. Instead of:

```rust
while !flag { /* spin */ }  // wastes power, blocks other work
```

you write:

```rust
event.wait().await;  // executor runs other tasks meanwhile
```

**Embassy** is an embedded async ecosystem for Rust providing:

| Component | Role |
|-----------|------|
| **embassy-executor** | Run `async fn` tasks on `no_std` |
| **embassy-time** | Timers, `Timer::after_secs(1).await` |
| **embassy-sync** | Async mutex, channel, signal |
| **embassy-hal** crates | Platform init (embassy-nrf, embassy-stm32, esp-hal integration) |
| **embassy-net** | TCP/IP stack (smoltcp-based) |
| **embassy-usb** | USB device classes — [19-usb.md](./19-usb.md) |

### Executor model

An **executor** polls `Task` futures when they yield. On embedded, typically **one executor thread** + **interrupts wake tasks**:

```
Main thread: executor.run() polls tasks
IRQ:         set WAKER → executor schedules task
```

No OS threads required — fits `no_std`.

### vs RTIC / blocking

| Approach | Best for |
|----------|----------|
| **Embassy async** | Many I/O-bound tasks, networking, USB |
| **RTIC** | Strict priority scheduling, minimal overhead — [23-rtic.md](./23-rtic.md) |
| **Blocking + IRQ** | Simple projects, smallest conceptual surface |

You can combine patterns carefully (async task + RTIC shared resources) but start with one framework.

---

## Hardware Overview

Embassy is **platform-agnostic** — HAL bindings provide:

| Platform | Crate / integration |
|----------|---------------------|
| **ESP32-S3** | `esp-hal` + `embassy-executor` feature |
| **STM32** | `embassy-stm32` — extensive chip support |
| **RP2040** | `embassy-rp` |
| **nRF52** | `embassy-nrf` |

Each maps **hardware events** (UART RX ready, timer expire, DMA done) to **wakers**.

---

## ASCII Wiring

No special wiring — reuse any prior lesson hardware. Async LED blink:

```
GPIO48 ──► LED ──► GND   (ESP32-S3 DevKitC-1 onboard LED pin — verify schematic)
```

---

## Memory & Register Notes

### Task storage

Each `#[embassy_executor::task]` future is stored statically:

```rust
// Executor allocates task slots at compile time — no dynamic spawn heap by default
#[embassy_executor::task]
async fn blink_task(mut led: Output<'static>) {
    loop {
        led.toggle();
        Timer::after_millis(500).await;
    }
}
```

### Stack size

Async tasks use **separate stacks** (configurable). Deep recursion in tasks can overflow — prefer flat state machines — [27-design-patterns.md](./27-design-patterns.md).

### Wakers and `'static`

All resources referenced across `.await` must be `'static` — use `StaticCell`, `&'static mut`, or allocate at boot once.

---

## HAL Example — Embassy on ESP32-S3

```rust
#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Timer, Duration};
use esp_hal::{
    clock::ClockControl,
    gpio::{Io, Level, Output},
    peripherals::Peripherals,
    prelude::*,
};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
    let led = Output::new(io.pins.gpio48, Level::Low);

    // Spawn independent async task
    spawner.spawn(blink(led)).ok();

    // Main task can do other async work
    loop {
        Timer::after(Duration::from_secs(5)).await;
        // heartbeat log via defmt
    }
}

#[embassy_executor::task]
async fn blink(mut led: Output<'static>) {
    loop {
        led.toggle();
        Timer::after_millis(500).await;
    }
}
```

`Cargo.toml` features:

```toml
[dependencies]
esp-hal = { version = "0.21", features = ["esp32s3", "async", "embassy-time"] }
embassy-executor = { version = "0.6", features = ["arch-spin", "executor-thread"] }
embassy-time = { version = "0.3", features = ["generic-queue-8"] }
```

---

## Embassy on STM32 (embassy-stm32)

```rust
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_time::Timer;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    let mut led = Output::new(p.PA5, Level::Low, Speed::Low);

    loop {
        led.toggle();
        Timer::after_millis(500).await;
    }
}
```

See [boards/stm32.md](./boards/stm32.md) for clock config.

---

## Embassy on RP2040 (embassy-rp)

```rust
use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, Output};
use embassy_time::Timer;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let mut led = Output::new(p.PIN_25, Level::Low);

    loop {
        led.toggle();
        Timer::after_millis(500).await;
    }
}
```

---

## Async UART / SPI pattern

```rust
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;

static UART: Mutex<CriticalSectionRawMutex, UartDriver> = Mutex::new(/* ... */);

#[embassy_executor::task]
async fn reader_task() {
    let mut buf = [0u8; 64];
    loop {
        let mut uart = UART.lock().await;
        let n = uart.read_async(&mut buf).await;
        drop(uart); // release before processing
        process(&buf[..n]);
    }
}
```

`Mutex::lock().await` yields instead of spinning — critical for fairness.

---

## Step-by-Step

1. Add **embassy-executor** + **embassy-time** to existing HAL project.
2. Convert `loop { delay; toggle }` to async blink.
3. Split into **two tasks** — blink + serial reader.
4. Add **`embassy-sync` channel** between ISR-deferred producer and async consumer.
5. Try **`embassy-net`** TCP echo on ESP32 when Wi-Fi stable — [20-wifi.md](./20-wifi.md).
6. Profile stack high-water with `defmt` or executor features.

---

## Common Mistakes

| Mistake | Symptom | Fix |
|---------|---------|-----|
| Blocking in async task | Other tasks starve | Use `.await` APIs, not `thread::sleep` |
| Missing timer driver | Panic at boot | Enable `embassy-time` driver feature for chip |
| `'static` violation | Won't compile | StaticCell / spawn at init |
| Mutex held across `.await` | Deadlock | Release lock before await |
| Too many tasks / small stacks | Random faults | Increase stack, reduce tasks |
| IRQ waker not set | Task never wakes | Check HAL async binding |

---

## Debugging Tips

- **`defmt` timestamps** with `embassy-time` — correlate events.
- Count **executor poll iterations** in debug builds.
- Use **`embassy-futures::select`** to race timeout vs channel receive.
- GDB breakpoints in task entry — [25-debugging.md](./25-debugging.md).
- Compare behavior with **blocking version** first.

---

## Performance Tips

- Async shines when **multiple I/O sources** wait concurrently.
- For **tight bit-bang timing**, use PIO/hardware or RTIC ISR — not async.
- Prefer **`embassy-sync` channels** over busy polling flags.
- Enable **compiler LTO** for smaller executor overhead.
- On ESP32, Wi-Fi task integration — follow esp-wifi embassy examples for tuned priorities.

---

## Exercises

1. **Two-task dashboard:** Task A reads button; Task B blinks LED rate based on channel messages.
2. **Timeout UART line:** Read until `\n` or 100 ms idle using `select`.
3. **STM32 port:** Same blink on Nucleo with `embassy-stm32`.
4. **Measure latency:** GPIO toggle on async channel receive — scope vs IRQ baseline.
5. **USB + blink:** Run [19-usb.md](./19-usb.md) CDC concurrently with LED task.

---

## References

- [Embassy Book](https://embassy.dev/book/)
- [embassy-executor docs](https://docs.embassy.dev/embassy-executor/)
- [esp-hal Embassy integration](https://docs.esp-rs.org/esp-hal/)
- [23-rtic.md](./23-rtic.md) — alternative concurrency
- [27-design-patterns.md](./27-design-patterns.md) — state machines in async

---

*Previous: [21-bluetooth.md](./21-bluetooth.md) · Next: [23-rtic.md](./23-rtic.md)*
