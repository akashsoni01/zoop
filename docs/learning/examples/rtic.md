# Example: RTIC (Real-Time Interrupt-driven Concurrency)

**Goal:** Schedule tasks on ARM (STM32/RP2040) with zero-cost mutexes.

**Prerequisites:** [09-interrupts.md](../09-interrupts.md)

**Note:** RTIC targets **ARM Cortex-M** primarily. ESP32-S3 uses **Embassy** or FreeRTOS — study RTIC for portable patterns when using Nucleo/ Pico boards per [README](../README.md).

---

## Rust Sketch (RP2040)

```rust
#[app(device = rp_pico, peripherals = true)]
mod app {
    use rtic::app;

    #[shared]
    struct Shared { counter: u32 }

    #[local]
    struct Local { led: Pin<Output> }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        blink::spawn().ok();
        (Shared { counter: 0 }, Local { led: /* ... */ }, init::Monotonics())
    }

    #[task(shared = [counter])]
    async fn blink(mut ctx: blink::Context) {
        loop {
            ctx.shared.counter.lock(|c| *c += 1).await;
            Mono::delay(500.millis()).await;
        }
    }
}
```

RTIC **priority ceiling** prevents deadlock — mutex locks are task-aware.

---

## When to Use

| Framework | Best for |
|-----------|----------|
| Embassy | ESP32-S3, async I/O |
| RTIC | ARM bare-metal, strict ISR latency |
| FreeRTOS | ESP-IDF interop |

*Index: [README.md](./README.md)*
