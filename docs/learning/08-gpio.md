# Lesson 08: GPIO — Digital Input and Output

**GPIO (General Purpose Input/Output)** pins are the foundation of embedded interaction — LEDs, buttons, relays, and digital sensors all start here.

**Prerequisites:** [07-hal.md](./07-hal.md)  
**Next:** [09-interrupts.md](./09-interrupts.md)  
**See also:** [06-register-programming.md](./06-register-programming.md), [11-pwm.md](./11-pwm.md)

---

## Theory

### GPIO Modes

| Mode | Direction | Use Case |
|------|-----------|----------|
| **Output push-pull** | Out | Drive LED high/low |
| **Output open-drain** | Out | I²C, shared bus pull-up |
| **Input floating** | In | External pull resistor required |
| **Input pull-up** | In | Button to GND (active low) |
| **Input pull-down** | In | Button to VCC (active high) |
| **Analog** | — | ADC input ([12-adc.md](./12-adc.md)) |
| **Alternate function** | — | UART, SPI hardware peripheral |

### Electrical Basics

- **Active high:** LED on when pin is HIGH (3.3 V)
- **Active low:** LED on when pin is LOW (common on STM32 Nucleo)
- **Logic levels:** ESP32/STM32/RP2040 are **3.3 V** — do not connect 5 V signals directly

Current through LED: `I = (Vgpio - Vf_led) / R`. Target ~5–10 mA.

---

## Hardware Overview

### ESP32-S3 DevKitC-1

- Built-in RGB LED may be on GPIO48 (WS2812) or separate GPIO2 LED depending on revision
- Boot button: GPIO0 (strapping pin — avoid holding at boot)
- 45 GPIO pins, not all exposed on DevKit headers

### STM32 Nucleo-F411RE (Comparison)

- User LED: PA5 (active high)
- User button: PC13 (active low, internal pull-up on some boards)

### RP2040 Pico (Comparison)

- On-board LED: GPIO25 (active low on Pico W variant differs)
- 30 multi-function GPIO pins

---

## Wiring Diagram

### LED + Button Circuit (Breadboard)

```
                    ESP32-S3 DevKit
                  ┌─────────────────┐
                  │                 │
    3V3 ──────────┤ 3V3             │
                  │                 │
                  │ GPIO2 ────┬─────┼──► To LED anode (+)
                  │           │     │
                  │           [ 220Ω ]     (current limit)
                  │           │     │
                  │           └──(>|)────┤ GND (cathode)
                  │                 │
                  │ GPIO0 ────┬─────┤ (input, internal pull-up enabled in software)
                  │           │     │
                  │          [SW]     │ tactile button
                  │           │     │
    GND ──────────┤ GND ──────┴─────┤
                  └─────────────────┘

Button wiring (active low):
  GPIO0 ──── button ──── GND
  (internal pull-up keeps pin HIGH when open)
  (pressed → pin pulled LOW)
```

### Multi-LED Example

```
GPIO2 ──[220Ω]──(LED1)── GND
GPIO4 ──[220Ω]──(LED2)── GND
GPIO5 ──[220Ω]──(LED3)── GND
```

Always share common GND between board and breadboard.

---

## Memory & Register Explanation

GPIO state lives in MMIO registers — see [06-register-programming.md](./06-register-programming.md).

| Operation | ESP32-S3 Register | STM32 Register |
|-----------|-------------------|----------------|
| Set output high | `GPIO_OUT_W1TS` | `BSRR` low half |
| Set output low | `GPIO_OUT_W1TC` | `BSRR` high half |
| Read input | `GPIO_IN` | `IDR` |
| Enable output | `GPIO_ENABLE` | `MODER` |

HAL configures these once; application code calls `set_high()` / `is_low()`.

---

## HAL Implementation

Complete ESP32-S3 blink + button toggle:

```rust
#![no_std]
#![no_main]

use defmt::info;
use embedded_hal::digital::{InputPin, OutputPin};
use esp_hal::{
    clock::ClockControl,
    delay::Delay,
    gpio::{Io, Level, Input, Output, Pull},
    peripherals::Peripherals,
};
use panic_halt as _;

#[esp_hal::macros::entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    // `mut led` — owned locally; mutable for OutputPin methods
    let mut led = Output::new(io.pins.gpio2, Level::Low);

    // Button with internal pull-up — `Input` owns pin config
    let button = Input::new(io.pins.gpio0, Pull::Up);

    let mut delay = Delay::new(&clocks);
    let mut led_state = false;

    loop {
        // `&button` — immutable borrow; reading doesn't need mut
        if button.is_low().unwrap() {
            // Debounce delay — better: timer or interrupt ([09-interrupts.md](./09-interrupts.md))
            delay.delay_ms(50);

            if button.is_low().unwrap() {
                led_state = !led_state;
                if led_state {
                    led.set_high().unwrap();
                    info!("LED ON");
                } else {
                    led.set_low().unwrap();
                    info!("LED OFF");
                }
                // Wait for release
                while button.is_low().unwrap() {
                    delay.delay_ms(10);
                }
            }
        }
        delay.delay_ms(10);
    }
}
```

STM32 equivalent:

```rust
let mut led = gpioa.pa5.into_push_pull_output();
let button = gpioc.pc13.into_pull_up_input();

loop {
    if button.is_low().unwrap_or(false) {
        led.toggle().ok();
        // debounce...
    }
}
```

---

## Bare-Metal Implementation

```rust
const GPIO_BASE: u32 = 0x6000_4000;
const GPIO_ENABLE: u32 = GPIO_BASE + 0x20;
const GPIO_OUT_W1TS: u32 = GPIO_BASE + 0x10;
const GPIO_OUT_W1TC: u32 = GPIO_BASE + 0x14;
const GPIO_IN: u32 = GPIO_BASE + 0x3C;
const IO_MUX_GPIO: u32 = 0x6000_9000;

const PIN_LED: u32 = 2;
const PIN_BTN: u32 = 0;
const MASK_LED: u32 = 1 << PIN_LED;
const MASK_BTN: u32 = 1 << PIN_BTN;

unsafe fn gpio_init() {
    // Mux GPIO2 and GPIO0 as IO function
    core::ptr::write_volatile((IO_MUX_GPIO + PIN_LED * 4) as *mut u32, 1);
    core::ptr::write_volatile((IO_MUX_GPIO + PIN_BTN * 4) as *mut u32, 1);

    // Enable output on LED pin only
    let en = core::ptr::read_volatile(GPIO_ENABLE as *const u32);
    core::ptr::write_volatile(GPIO_ENABLE as *mut u32, en | MASK_LED);

    // Enable input on button — ESP32: clear enable bit for input
    let en = core::ptr::read_volatile(GPIO_ENABLE as *const u32);
    core::ptr::write_volatile(GPIO_ENABLE as *mut u32, en & !MASK_BTN);
    // Note: pull-up configured via separate register — HAL handles this
}

unsafe fn led_on()  { core::ptr::write_volatile(GPIO_OUT_W1TS as *mut u32, MASK_LED); }
unsafe fn led_off() { core::ptr::write_volatile(GPIO_OUT_W1TC as *mut u32, MASK_LED); }

unsafe fn button_pressed() -> bool {
    let inputs = core::ptr::read_volatile(GPIO_IN as *const u32);
    (inputs & MASK_BTN) == 0  // Active low
}

fn main_bare() -> ! {
    unsafe {
        gpio_init();
        let mut on = false;
        loop {
            if button_pressed() {
                on = !on;
                if on { led_on(); } else { led_off(); }
                while button_pressed() {}  // wait release
            }
            busy_wait();
        }
    }
}

fn busy_wait() {
    for _ in 0..100_000 { core::hint::spin_loop(); }
}
```

---

## Step-by-Step Explanation

### Step 1: Identify Pins

Check board schematic. Note LED polarity and button strapping constraints (GPIO0 on ESP32 affects boot mode).

### Step 2: Wire Breadboard

Connect LED with resistor. Connect button between GPIO and GND. Verify GND common.

### Step 3: Initialize Clocks and GPIO HAL

Follow template project structure — clocks before GPIO.

### Step 4: Configure Output Pin

Set initial level (usually LOW to avoid LED flash at boot).

### Step 5: Configure Input with Pull-Up

Internal pull-up (~45 kΩ on ESP32) usually sufficient for buttons.

### Step 6: Poll Button in Loop

Simplest approach — works for learning; interrupts improve responsiveness ([09-interrupts.md](./09-interrupts.md)).

### Step 7: Toggle LED on Press

Add software debounce (20–50 ms) to filter mechanical bounce.

---

## Common Mistakes

| Mistake | Symptom | Fix |
|---------|---------|-----|
| No current-limiting resistor | Dead LED or damaged GPIO | Add 220 Ω–1 kΩ |
| Wrong pin number | Nothing happens | Check schematic |
| Floating input | Erratic button reads | Enable pull-up/down |
| Active low vs high confusion | Inverted behavior | Match `is_low()` / `is_high()` |
| GPIO0 held at boot (ESP32) | Boot failure | Release button before reset |
| 5 V on 3.3 V pin | Damaged MCU | Level shifter or 3.3 V logic |

---

## Debugging Tips

1. **Toggle pin with multimeter** — verify voltage changes.
2. **Slow blink on boot** — confirms code runs before button logic.
3. **`defmt` log raw pin read** — see bounce noise.
4. **Swap LED orientation** — won't light if reversed.
5. **Logic analyzer** — visualize bounce duration (typically 1–20 ms).

---

## Performance Tips

| Tip | Detail |
|-----|--------|
| Use interrupts for buttons | Frees CPU ([09-interrupts.md](./09-interrupts.md)) |
| Batch GPIO via port writes | Toggle multiple pins in one register write |
| Avoid polling in tight loop | Add small delay or WFI sleep |
| Direct register access in ISR | Only if profiling shows HAL too slow |

---

## Exercises

### Exercise 1: Blink Rate

Modify blink to 2 Hz. Measure with phone camera slow-mo or logic analyzer.

### Exercise 2: Three LEDs

Control three LEDs with three GPIO pins; chase pattern.

### Exercise 3: Hold-to-Dim Preview

Button held > 1 s turns LED on; short press toggles (prepares for [11-pwm.md](./11-pwm.md)).

### Exercise 4: Bare-Metal Port

Rewrite HAL blink in bare-metal PAC/raw register style.

### Exercise 5: Portable Driver

Write `ButtonLed` struct using `InputPin` + `OutputPin` traits; test on two boards if available.

---

## References

- [ESP32-S3 TRM — GPIO](https://www.espressif.com/sites/default/files/documentation/esp32-s3_technical_reference_manual_en.pdf)
- [embedded-hal OutputPin](https://docs.rs/embedded-hal/1.0.0/embedded_hal/digital/trait.OutputPin.html)
- [Lesson 06 — Register Programming](./06-register-programming.md)
- [STM32 Nucleo User Manual](https://www.st.com/resource/en/user_manual/um1724-stm32-nucleo64-boards-mb1136-stmicroelectronics.pdf)

---

*Previous: [07-hal.md](./07-hal.md) | Next: [09-interrupts.md](./09-interrupts.md)*
