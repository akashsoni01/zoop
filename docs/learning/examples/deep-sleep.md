# Example: Deep Sleep

**Goal:** Wake from timer or GPIO after microamp sleep.

**Prerequisites:** [rtc.md](./rtc.md), firmware power docs

---

## Rust Sketch

```rust
use esp_hal::rtc_cntl::{Rtc, sleep::TimerWakeupSource, WakeSource};

fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let mut rtc = Rtc::new(peripherals.LPWR);

    // Wake every 60 seconds
    let timer = TimerWakeupSource::new(60_000_000); // µs
    rtc.sleep_enable(&[WakeSource::Timer(timer)]);

    // Before sleep: disconnect Wi-Fi, flush NVS
    rtc.enter_deep_sleep();

    // --- wakes here after 60 s ---
    loop {}
}
```

Measure current with multimeter in series — expect **~10 µA** with proper design.

See [projects/battery-monitor.md](../projects/battery-monitor.md).

*Next: [wifi-client.md](./wifi-client.md)*
