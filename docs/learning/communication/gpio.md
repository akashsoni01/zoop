# GPIO — General Purpose Input/Output

**GPIO** (General Purpose Input/Output) is the simplest digital communication between an MCU and the outside world: a pin is either driven **high** (~3.3 V) or **low** (~0 V), or configured as an **input** to read an external voltage.

**Prerequisites:** [08-gpio.md](../08-gpio.md), [07-hal.md](../07-hal.md)

---

## Theory

Each GPIO pin connects to a **pad** on the silicon, routed through an **I/O matrix** (ESP32-S3) or **alternate-function mux** (STM32). The CPU writes to **configuration registers** (direction, pull-up/down, drive strength) and **data registers** (set/clear/toggle).

| Mode | Behavior |
|------|----------|
| Output push-pull | Actively drives HIGH or LOW |
| Input floating | High impedance; undefined without external bias |
| Input pull-up | Weak internal resistor to VDD (~45 kΩ on ESP32-S3) |
| Input pull-down | Weak internal resistor to GND |
| Open-drain output | Can only pull LOW; HIGH requires external pull-up |

**Active-low** wiring is common: LED cathode to GPIO, anode through resistor to 3V3 — logic `0` turns the LED on.

On ESP32-S3, most pins are **3.3 V tolerant only**. Some strapping pins (GPIO0, GPIO3, GPIO45, GPIO46) affect boot mode — avoid pulling them incorrectly at reset.

---

## Timing Diagram (ASCII)

Output toggle at 1 kHz (software delay):

```
        ┌───┐   ┌───┐   ┌───┐
GPIO    │   │   │   │   │   │
    ────┘   └───┘   └───┘   └───
        |<->|
         1 ms period (500 µs high, 500 µs low)
```

Input with debouncing (button, ~20 ms filter):

```
Button (raw)  ──┐     ┌─┐     ┌──────────
                └─────┘ └─────┘
                     bounce

Debounced     ────────────────────────────
              (stable LOW after 20 ms)
```

---

## "Packet Format"

GPIO has no packets. Treat each pin as **1 bit** of state:

| Field | Size | Meaning |
|-------|------|---------|
| Level | 1 bit | 0 = LOW, 1 = HIGH |
| Direction | 1 bit | 0 = input, 1 = output |

Multi-pin **parallel GPIO** (e.g., 8-bit LCD data bus) sends one "byte" per write cycle — see [examples/lcd.md](../examples/lcd.md).

---

## Electrical Characteristics

| Parameter | ESP32-S3 (typical) | Notes |
|-----------|-------------------|-------|
| VIH | 0.75 × VDD | Input read as HIGH |
| VIL | 0.25 × VDD | Input read as LOW |
| VOH | ~3.3 V @ 20 mA | Depends on drive strength |
| Max source/sink | 40 mA per pin (abs max) | Stay ≤ 12 mA for reliability |
| Pull-up | ~45 kΩ internal | Optional |
| Rise time | ~ns–µs | Depends on load capacitance |

**Always** use a current-limiting resistor with LEDs (220 Ω–1 kΩ). A red LED at 3.3 V with 330 Ω draws ~10 mA.

---

## Rust HAL Sketch (esp-hal)

```rust
use esp_hal::gpio::{Level, Output, Input, Pull};
use esp_hal::delay::Delay;
use esp_hal::prelude::*;

// Ownership: `led` owns the pin peripheral; it cannot be duplicated.
// The type system prevents use-after-move of the GPIO block.

pub fn blink(mut led: Output<'static>, delay: &mut Delay) -> ! {
    loop {
        led.toggle();           // embedded-hal digital::ToggleableOutputPin
        delay.delay_millis(500);
    }
}

pub fn read_button(btn: &Input<'static>) -> bool {
    // Active-low button with internal pull-up
    btn.is_low()
}
```

**Compile notes (ESP32-S3):**

```bash
cargo build --release
espflash flash --monitor target/xtensa-esp32s3-none-elf/release/your-crate
```

Target: `xtensa-esp32s3-none-elf`. Enable `esp-hal` with appropriate features in `Cargo.toml`.

---

## Bare-Metal Sketch (Register-Level Concept)

```rust
// Illustrative — actual addresses from ESP32-S3 TRM
const GPIO_OUT_REG: *mut u32 = 0x6000_4004 as *mut u32;
const GPIO_ENABLE_REG: *mut u32 = 0x6000_4020 as *mut u32;
const LED_BIT: u32 = 1 << 2; // GPIO2

unsafe fn gpio2_output_enable() {
    GPIO_ENABLE_REG.write_volatile(GPIO_ENABLE_REG.read_volatile() | LED_BIT);
}

unsafe fn gpio2_toggle() {
    let v = GPIO_OUT_REG.read_volatile();
    GPIO_OUT_REG.write_volatile(v ^ LED_BIT);
}
```

Prefer HAL for portability — see [06-register-programming.md](../06-register-programming.md).

---

## Example Projects

| Project | GPIO Use |
|---------|----------|
| [examples/blink-led.md](../examples/blink-led.md) | Output toggle |
| [examples/button.md](../examples/button.md) | Input + pull-up |
| [projects/smart-switch.md](../projects/smart-switch.md) | Relay control |
| [projects/ble-beacon.md](../projects/ble-beacon.md) | Status LED |

---

## Common Mistakes

1. **No current-limiting resistor** on LED — damages pin or LED.
2. **Floating input** — reads random values; enable pull-up/down.
3. **Exceeding 3.3 V** on ESP32 pins — use level shifters for 5 V logic.
4. **Strapping pins at boot** — GPIO0 held LOW enters download mode.
5. **Busy-wait blink in ISR** — blocks interrupts; use timers ([10-timers.md](../10-timers.md)).

---

## Exercises

1. Blink an LED at 2 Hz using [08-gpio.md](../08-gpio.md) patterns.
2. Read a button and toggle LED on each press (debounce in software).
3. Drive two LEDs alternately without `delay` — use a timer interrupt.
4. Measure pin toggle rate with a logic analyzer; compare HAL vs register access.

---

## References

- [ESP32-S3 Technical Reference Manual](https://www.espressif.com/en/products/socs/esp32-s3) — GPIO chapter
- [embedded-hal digital traits](https://docs.rs/embedded-hal/latest/embedded_hal/digital/index.html)
- [esp-hal GPIO docs](https://docs.espressif.com/projects/rust/esp-hal/latest/esp32s3/esp_hal/gpio/index.html)
- Lesson: [08-gpio.md](../08-gpio.md)

---

*Next: [uart-usart.md](./uart-usart.md) | Index: [README.md](./README.md)*
