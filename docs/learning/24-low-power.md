# Lesson 24 — Low Power: Sleep, Deep Sleep, Wake Sources

**Prerequisites:** [09-interrupts.md](./09-interrupts.md), [10-timers.md](./10-timers.md), [20-wifi.md](./20-wifi.md) (if using radio sleep)

**Board focus:** [ESP32-S3](./boards/esp32-s3.md), [nrf52.md](./boards/nrf52.md) (ULP excellence).

---

## Theory

Battery-powered firmware must **minimize average current**. MCUs offer tiered sleep modes trading **wake latency** vs **power savings**.

### Generic sleep hierarchy

| Mode | CPU | RAM | Peripherals | Wake source | Current (typical) |
|------|-----|-----|-------------|-------------|-------------------|
| **Run** | On | On | On | — | tens of mA |
| **Sleep / WFI** | Off | On | Partial | Any IRQ | ~1–10 mA |
| **Light sleep** | Off | On | Slowed/gated | Timer, GPIO | hundreds of µA |
| **Deep sleep** | Off | Partial/off | Off | RTC, GPIO, ULP | tens of µA |
| **Hibernate / off** | Off | Off | Off | Reset pin | nA (RTC domain) |

Exact names vary by vendor — always read the TRM for your chip.

### Wake sources

| Source | Example use |
|--------|-------------|
| **GPIO** | Button press |
| **RTC timer** | Hourly sensor sample |
| **UART start bit** | Wake on serial command |
| **ULP coprocessor** | Espressif ultra-low-power core watches ADC |
| **BLE connection event** | Maintained connection schedule |

### Current budget math

```
Battery life (hours) ≈ Capacity (mAh) / Average current (mA)

Example: 1000 mAh coin cell, 50 µA average
  → 1000 / 0.05 = 20,000 h ≈ 2.3 years (idealized)
```

Budget **peak TX current** (Wi-Fi/BLE) separately — brownout if supply cannot deliver.

---

## Hardware Overview

### ESP32-S3 power modes

| Mode | API (ESP-IDF) | Notes |
|------|---------------|-------|
| **Modem sleep** | Wi-Fi/BLE controller sleep | Requires clock sync with AP |
| **Light sleep** | CPU paused, RTC running | Fast wake (~1 ms) |
| **Deep sleep** | Most digital off | **~7 µA** class with ULP; RAM loss unless RTC memory kept |

**RTC slow memory** retains data across deep sleep — store boot counter:

```rust
#[link_section = ".rtc.data"]
static mut BOOT_COUNT: u32 = 0;
```

### nRF52840

**System ON idle** vs **System OFF** — sub-µA with RAM loss in OFF. **GPIOTE** sense for pin wake — [boards/nrf52.md](./boards/nrf52.md).

### STM32 Stop / Standby

**Stop mode** keeps SRAM; **Standby** minimal retention — wake via WKUP pins.

---

## ASCII Wiring

Deep sleep wake on **EXT0** button (ESP32-S3):

```
        ┌─── 10 kΩ pull-up ─── 3V3
        │
GPIO0 ──┴──► Button ──► GND   (RTC-capable pin — check TRM)

Measure current: multimeter in series with 3V3 supply
  OR Nordic PPK2 / Joulescope for µA resolution
```

Avoid USB-powered measurement noise — use battery + LDO for accurate µA readings.

---

## Memory & Register Notes

### ESP32-S3 sleep registers (conceptual)

| Register / API | Purpose |
|----------------|---------|
| `RTC_CNTL_STATE0` | Sleep entry control |
| `RTC_GPIO` | Hold levels during sleep |
| `esp_sleep_enable_ext0_wakeup` | Pin level wake |
| `esp_sleep_enable_timer_wakeup` | µs timer wake |

### Clock gating

Peripherals left enabled leak current — **disable UART/SPI clocks** before sleep:

```rust
// esp-idf: esp_sleep_pd_config(ESP_PD_DOMAIN_RTC_PERIPH, ESP_PD_OPTION_OFF);
```

### Rust ownership across sleep

Deep sleep **resets CPU** — only RTC memory persists. Structure firmware as:

```
boot → read wake cause → work burst → configure wake → enter deep sleep
```

Not "pause and resume" like laptop sleep.

---

## HAL Example — Deep sleep timer wake (esp-idf-svc)

```rust
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::sys::{esp_sleep_enable_timer_wakeup, esp_deep_sleep_start};
use core::time::Duration;

fn main() {
    esp_idf_svc::sys::link_patches();

    // Log wake cause (timer, gpio, etc.) — first line after boot
    let cause = esp_idf_svc::hal::reset::ResetReason::get();
    log::info!("Reset reason: {:?}", cause);

    // Do work: read sensor, queue transmission...

    // Wake again in 60 seconds (microseconds argument)
    let us = Duration::from_secs(60).as_micros() as u64;
    unsafe {
        esp_sleep_enable_timer_wakeup(us);
        esp_deep_sleep_start();
    }
    // never returns
}
```

---

## esp-hal light sleep (no_std direction)

```rust
// Check esp-hal sleep API for your version — pattern:
// esp_hal::rtc_cntl::Rtc::new(...);
// rtc.sleep_enable_timer(1_000_000); // 1 s
// rtc.enter_light_sleep();
// continues here after wake — RAM retained
```

API evolves — cross-check [esp-hal documentation](https://docs.esp-rs.org/esp-hal/).

---

## GPIO wake example (ESP-IDF)

```rust
use esp_idf_svc::sys::{
    esp_sleep_enable_ext0_wakeup,
    esp_deep_sleep_start,
    esp_sleep_get_wakeup_cause,
    esp_sleep_wakeup_cause_t_ESP_SLEEP_WAKEUP_EXT0,
};

fn main() {
    // Configure GPIO0 as RTC pin, wake on low level
    unsafe {
        esp_sleep_enable_ext0_wakeup(0, 0); // pin 0, level 0
    }

    if unsafe { esp_sleep_get_wakeup_cause() } == esp_sleep_wakeup_cause_t_ESP_SLEEP_WAKEUP_EXT0 {
        log::info!("Woke from button!");
    }

    // ... work ...

    unsafe { esp_deep_sleep_start(); }
}
```

---

## Step-by-Step

1. **Baseline current:** Measure run-mode mA with multimeter.
2. **WFI idle:** RTIC/Embassy idle with `wfi` — drop to lower mA — [23-rtic.md](./23-rtic.md).
3. **Light sleep** between periodic samples — verify wake on timer.
4. **Deep sleep** with RTC timer — confirm boot reason logging.
5. Add **GPIO wake** on button; debounce in software after wake.
6. If using **Wi-Fi**, enable modem sleep — [20-wifi.md](./20-wifi.md).
7. Document **energy per task** in spreadsheet.

---

## Common Mistakes

| Mistake | Symptom | Fix |
|---------|---------|-----|
| USB serial left open | High sleep current | Disconnect or power gate adapter |
| Floating wake pin | Random wake | Pull-up/down + RTC hold |
| Wrong pin not RTC-capable | No EXT0 wake | Check TRM RTC IO map |
| Peripherals not deinit | 1–5 mA sleep | Gate clocks, enter reset |
| Brownout on TX spike | Reset loops | Bulk cap, better LDO |
| Expect RAM after deep sleep | Corrupt state | Use RTC memory or NVS |
| Debugger attached | Never sleeps | Detach for power test |

---

## Debugging Tips

- **Wake cause register** first line every boot.
- **GPIO hold** during sleep for stable CS lines to external flash.
- **PPK2** power profiler — correlate spikes with code phases.
- Log **time spent awake** vs sleep duration ratio.
- Compare **sdkconfig** power options (`CONFIG_PM_ENABLE`, etc.).

---

## Performance Tips

- **Batch work:** Wake → sense → TX → sleep (minimum awake window).
- **Slow clock** for RTC timers — less drift acceptable for hourly samples.
- **ULP/PCD** (Programmable Counter Detector) on ESP for anomaly wake.
- BLE: lengthen **connection interval** during idle — [21-bluetooth.md](./21-bluetooth.md).
- Replace **polling loops** with interrupt + sleep — largest firmware win.

---

## Exercises

1. **Boot counter:** Persist increment in RTC memory across deep sleep.
2. **Button-only wake:** Device sleeps until button; toggle LED briefly; sleep again.
3. **Energy budget:** Calculate theoretical battery life for 1 s awake / 59 s sleep cycle.
4. **Modem sleep:** Wi-Fi connected; measure difference modem sleep on/off.
5. **nRF comparison:** Same counter on nRF52840 System OFF — [boards/nrf52.md](./boards/nrf52.md).

---

## References

- [ESP32-S3 Sleep Modes (ESP-IDF)](https://docs.espressif.com/projects/esp-idf/en/latest/esp32s3/api-reference/system/sleep_modes.html)
- [ESP32 power consumption measurement app note](https://www.espressif.com/en/support/documents/technical-documents)
- [nRF52 Power Management](https://infocenter.nordicsemi.com/)
- [boards/esp32-s3.md](./boards/esp32-s3.md) — RTC memory map
- [23-rtic.md](./23-rtic.md) — WFI in idle

---

*Previous: [23-rtic.md](./23-rtic.md) · Next: [25-debugging.md](./25-debugging.md)*
