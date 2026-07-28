# Example: Real-Time Clock (RTC)

**Goal:** Keep time across resets with ESP32-S3 RTC + NTP sync.

**Prerequisites:** [04-cargo.md](../04-cargo.md)

---

## Rust Sketch

```rust
use esp_hal::rtc::Rtc;

fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let rtc = Rtc::new(peripherals.LPWR);

    // After Wi-Fi NTP sync (see wifi-client.md):
    // rtc.set_time(datetime);

    loop {
        let now = rtc.current_time();
        defmt::info!("{:?}", now);
        delay.delay_secs(1);
    }
}
```

RTC domain survives **deep sleep** — see [deep-sleep.md](./deep-sleep.md).

*Next: [deep-sleep.md](./deep-sleep.md)*
