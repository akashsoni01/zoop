# Lesson 24 — Low Power: Sleep, Deep Sleep, Wake Sources

**Prerequisites:** [09-interrupts.md](./09-interrupts.md), [10-timers.md](./10-timers.md), [20-wifi.md](./20-wifi.md) (if using radio sleep)

**Board focus:** [ESP32-S3](./boards/esp32-s3.md), [nrf52.md](./boards/nrf52.md).

**Maturity note:** Power APIs are **C-centric in ESP-IDF** — wrap from Embedded Swift. Bare-metal Swift can enter **light sleep** via register writes on C6 PoCs; deep sleep + Wi-Fi modem sleep requires ESP-IDF. **Host Swift** apps don't apply — this lesson is MCU-focused.

---

## Theory

Battery-powered firmware must **minimize average current** through tiered sleep modes.

### Sleep hierarchy

| Mode | CPU | RAM | Wake source | Current (typical) |
|------|-----|-----|-------------|-------------------|
| **Run** | On | On | — | tens of mA |
| **Sleep / WFI** | Off | On | Any IRQ | ~1–10 mA |
| **Light sleep** | Off | On | Timer, GPIO | hundreds of µA |
| **Deep sleep** | Off | Partial | RTC, GPIO | tens of µA |
| **Hibernate** | Off | Off | Reset | nA |

### Wake sources

| Source | Example |
|--------|---------|
| **GPIO** | Button press |
| **RTC timer** | Hourly sensor sample |
| **UART start bit** | Wake on serial command |
| **ULP coprocessor** | Espressif ADC threshold |
| **BLE connection event** | Maintained connection schedule |

### Current budget

```
Battery life (hours) ≈ Capacity (mAh) / Average current (mA)

1000 mAh / 0.05 mA = 20,000 h ≈ 2.3 years (idealized)
```

Budget **peak TX current** (Wi-Fi/BLE) separately — brownout if supply cannot deliver.

---

## Hardware Overview

### ESP32-S3 power modes

| Mode | Notes |
|------|-------|
| **Modem sleep** | Wi-Fi/BLE controller sleep — needs AP sync |
| **Light sleep** | CPU paused, RTC running — ~1 ms wake |
| **Deep sleep** | Most digital off — **~7 µA** class; RAM loss unless RTC memory |

**RTC slow memory** retains data across deep sleep:

```swift
@Section(".rtc.data")
var bootCount: UInt32 = 0
```

### nRF52840

System ON idle vs System OFF — sub-µA in OFF — [boards/nrf52.md](./boards/nrf52.md).

---

## ASCII Wiring

Deep sleep wake on **EXT0** button (ESP32-S3):

```
        ┌─── 10 kΩ pull-up ─── 3V3
        │
GPIO0 ◄─┴─── Button ─── GND   (press = LOW → wake)
```

Ensure strapping pin GPIO0 behavior at reset is acceptable — [boards/esp32-s3.md](./boards/esp32-s3.md).

---

## Memory & Register Notes

### ESP32-S3 sleep registers (conceptual)

| Register / API | Purpose |
|----------------|---------|
| `RTC_CNTL_STATE0` | Sleep enable |
| `RTC_GPIO` | Hold GPIO state during sleep |
| `esp_sleep_enable_ext0()` | GPIO wake config |
| `esp_deep_sleep_start()` | Enter deep sleep |

Store persistent config in **RTC memory** or **NVS flash**.

---

## HAL Swift Example — Deep Sleep Counter (ESP-IDF)

```swift
import ESPIDF

@Section(".rtc.data")
var bootCount: UInt32 = 0

@main
struct DeepSleepApp {
    static func main() {
        bootCount &+= 1
        print("Boot count: \(bootCount)\r\n")

        // Wake on GPIO0 low (button)
        ESPSleep.enableExt0Wake(pin: 0, level: 0)
        ESPSleep.enableTimerWake(seconds: 60) // backup wake

        print("Entering deep sleep — press button or wait 60s\r\n")
        ESPSleep.deepSleep()
        // Does not return — chip resets on wake
    }
}
```

---

## Bare-Metal Swift Sketch

```swift
import MMIO

let rtc = RTC_CNTL(baseAddress: 0x6000_8000)

func enterLightSleep(wakeUs: UInt64) {
    rtc.timer.set(wakeUs)
    rtc.state0.sleep_en.set(true)
    waitForInterrupt() // WFI
}

func enterDeepSleep() {
    rtc.state0.sleep_en.set(true)
    rtc.state0.sleep_sel.set(0x2) // deep sleep — check TRM
    // Flush UART, disable peripherals
    halt()
}
```

---

## Light Sleep with Wi-Fi Modem Sleep

```swift
// ESP-IDF path — modem sleep between MQTT publishes
func sensorLoop() async {
    while true {
        let reading = readSensor()
        try? await mqtt.publish(reading)
        ESPWiFi.modemSleep(true)
        try? await Task.sleep(for: .seconds(60))
        ESPWiFi.modemSleep(false)
    }
}
```

---

## Common Mistakes

| Mistake | Symptom | Fix |
|---------|---------|-----|
| Wake source not configured | Never wakes | Enable ext0/timer/GPIO |
| Floating wake pin | Random wake | Pull-up/down |
| UART not flushed before sleep | Garbled boot log | `uart.wait_tx_done()` |
| Wi-Fi always active | mA instead of µA | Modem sleep between uploads |
| Deep sleep + lost RAM vars | Zeroed globals | Use RTC memory section |
| Brownout on Wi-Fi TX | Reset during send | Bulk cap on supply |

---

## Debugging Tips

- Measure with **multimeter** (µA range) or **Joulescope**.
- Log wake cause: `esp_sleep_get_wakeup_cause()`.
- GPIO hold: `gpio_hold_en` during sleep for stable LED off.
- Compare light vs deep sleep current on same board.

---

## Performance Tips

- **Duty cycle:** Sense 1 s / sleep 59 s beats continuous sampling.
- **BLE connection interval** — longer interval = lower average current.
- Disable **USB-JTAG** in production if not needed.
- Use **ULP** for analog threshold wake without full CPU — ESP-IDF ULP coprocessor.

---

## Exercises

1. **Boot counter:** Increment RTC variable each deep sleep wake.
2. **Button wake:** Print wake cause (timer vs GPIO).
3. **Current measure:** Document run vs light vs deep sleep current.
4. **Modem sleep:** Wi-Fi MQTT publish every 5 min with modem sleep between.
5. **Host irrelevant:** Document power budget spreadsheet for your project.

---

## References

- [ESP-IDF Sleep Modes](https://docs.espressif.com/projects/esp-idf/en/latest/esp32s3/api-reference/system/sleep_modes.html)
- [ESP32-S3 TRM — RTC](https://www.espressif.com/en/products/socs/esp32-s3)
- [boards/esp32-s3.md](./boards/esp32-s3.md)
- [20-wifi.md](./20-wifi.md)

---

*Previous: [23-realtime-patterns.md](./23-realtime-patterns.md) · Next: [25-debugging.md](./25-debugging.md)*
