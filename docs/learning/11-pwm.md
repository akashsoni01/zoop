# Lesson 11: PWM — Pulse Width Modulation

**PWM (Pulse Width Modulation)** rapidly switches a digital pin between HIGH and LOW. By changing the **duty cycle** (fraction of time HIGH), you control average voltage — dimming LEDs or positioning servos.

**Prerequisites:** [10-timers.md](./10-timers.md)  
**Next:** [12-adc.md](./12-adc.md)  
**See also:** [08-gpio.md](./08-gpio.md), [13-dac.md](./13-dac.md)

---

## Theory

### PWM Parameters

| Parameter | Symbol | Unit | Effect |
|-----------|--------|------|--------|
| **Frequency** | f | Hz | How fast the pin toggles |
| **Period** | T = 1/f | s | One full ON+OFF cycle |
| **Duty cycle** | D | % | `(t_on / T) × 100` |
| **Resolution** | n bits | — | Discrete duty steps (2^n levels) |

### Why PWM Works for Analog Behavior

Human eyes and motor inertia **average** rapid pulses. 1 kHz PWM at 50% duty ≈ half supply voltage to a load (with appropriate filtering).

```
Duty 10%:  █░░░░░░░░░   (dim LED)
Duty 50%:  █████░░░░░   (medium)
Duty 90%:  █████████░   (bright)

Time ─────────────────────────────────►
```

### LED Dimming Frequency

- **< 100 Hz:** Visible flicker
- **100 Hz – 1 kHz:** Often acceptable
- **> 1 kHz:** Smooth for human vision; higher reduces efficiency slightly

### Servo Control (Intro)

Standard hobby servo (SG90) expects **50 Hz** frame with **1–2 ms** pulse width:

| Pulse Width | Angle (approx) |
|-------------|----------------|
| 1.0 ms | 0° |
| 1.5 ms | 90° |
| 2.0 ms | 180° |

---

## Hardware Overview

### ESP32-S3 — LEDC Peripheral

**LEDC (LED Control)** provides up to 8 channels with independent duty/frequency:

- High-speed or low-speed mode
- Up to 20-bit duty resolution (frequency dependent)
- Any GPIO via matrix

### STM32 — TIM PWM Mode

General-purpose timers (`TIM2`, etc.) support PWM on compare channels mapped to GPIO alternate functions.

### RP2040 — PWM Slice

8 PWM slices × 2 channels; 16 total outputs. Wrap value sets frequency.

---

## Wiring Diagram

### LED Dimming

```
ESP32-S3                    LED Circuit
┌──────────┐
│  GPIO4   ├──────[ 220Ω ]────(>|)──── GND
│          │      (PWM output)
│  GND     ├─────────────────────────── common
└──────────┘

Note: Use GPIO that supports LEDC channel — consult pin mux table.
```

### Servo Motor (External 5 V Supply Recommended)

```
                    ┌─────────────┐
                    │  Servo SG90 │
                    │             │
ESP32 GPIO5 (PWM)───┤ Signal (O)  │
                    │             │
Servo 5V ───────────┤ V+          │  ◄── Separate 5V supply (≥1A)
                    │             │
Common GND ─────────┤ GND         │  ◄── MUST share GND with ESP32
                    └─────────────┘

⚠️  Do not power servo from ESP32 3.3V pin — insufficient current.
⚠️  Signal is 3.3V — most servos accept it; use level shifter if unreliable.
```

---

## Memory & Register Explanation

### ESP32 LEDC Key Registers

| Register | Purpose |
|----------|---------|
| `LEDC_HSTimerx_CONF` | Timer prescaler, resolution |
| `LEDC_HSchx_CONF0` | Channel duty, timer binding |
| `LEDC_HSchx_DUTY` | Duty value (up to 20 bits) |
| `LEDC_HSchx_CONF1` | Update duty at period boundary |

Duty update pattern: write duty, set update bit, hardware applies at next period — avoids glitches.

### Frequency vs Resolution Trade-off

```
f_pwm = clock / (prescaler × (2^resolution))

Higher resolution → lower max frequency for same clock
```

---

## HAL Implementation

ESP32-S3 LED fade with `esp-hal`:

```rust
use esp_hal::ledc::{channel, timer, LowSpeed, LEDC, LSChannel, LSTimer};
use esp_hal::gpio::Io;
use esp_hal::peripherals::Peripherals;

#[esp_hal::macros::entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let clocks = /* init clocks */;
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    let ledc = LEDC::new(peripherals.LEDC, &clocks);
    let mut ledc_timer = ledc.get_timer::<LowSpeed>(LSTimer::Timer0);
    let mut ledc_channel = ledc.get_channel::<LowSpeed>(
        LSChannel::Channel0,
        io.pins.gpio4,
    );

    // 5 kHz, 10-bit resolution — good for LED dimming
    ledc_timer
        .configure(timer::config::TimerConfig::default().frequency(5.kHz()))
        .unwrap();
    ledc_channel
        .configure(channel::config::ChannelConfig::default())
        .unwrap();

    let max_duty = ledc_channel.max_duty();  // e.g., 1023 for 10-bit
    let mut direction: i32 = 1;
    let mut duty = 0u32;

    loop {
        ledc_channel.set_duty(duty).unwrap();
        ledc_channel.start().unwrap();

        duty = (duty as i32 + direction) as u32;
        if duty >= max_duty {
            duty = max_duty;
            direction = -1;
        } else if duty == 0 {
            direction = 1;
        }

        // Use timer tick from [10-timers.md](./10-timers.md) instead of busy-wait
        for _ in 0..10_000 { core::hint::spin_loop(); }
    }
}
```

### embedded-hal PWM Trait

```rust
use embedded_hal::pwm::SetDutyCycle;

fn set_brightness<P: SetDutyCycle>(pwm: &mut P, percent: u8) -> Result<(), P::Error> {
    let max = pwm.max_duty_cycle();
    let duty = (max as u32 * percent as u32 / 100) as P::Duty;
    pwm.set_duty_cycle(duty)
}
```

STM32:

```rust
let mut pwm = timer4.pwm_hz(pin, 1.kHz(), &clocks);
pwm.set_duty(timer4.ch1, pwm.get_max_duty() / 2);
```

---

## Bare-Metal Implementation

Software PWM (bit-bang) — educational only:

```rust
/// Software PWM — CPU intensive, timing sensitive.
/// `duty` out of `resolution` steps.
fn soft_pwm(pin_set: fn(), pin_clear: fn(), duty: u32, resolution: u32) {
    for step in 0..resolution {
        if step < duty {
            pin_set();
        } else {
            pin_clear();
        }
        // Tight timing loop — jitter likely without disabling interrupts
        for _ in 0..100 { core::hint::spin_loop(); }
    }
}
```

Hardware PWM strongly preferred for servos and dimming.

ESP32 LEDC bare-metal (sketch):

```rust
unsafe fn ledc_set_duty(channel: u32, duty: u32) {
    const LEDC_BASE: u32 = 0x6001_9000;
    // Write HSch duty register for channel
    // Set duty_start bit in conf1
    let duty_reg = (LEDC_BASE + 0x00 + channel * 0x14) as *mut u32;
    core::ptr::write_volatile(duty_reg, duty);
}
```

Consult TRM for exact offsets — HAL strongly recommended.

---

## Step-by-Step Explanation

### Step 1: Choose Timer Channel and GPIO

Verify pin supports PWM alternate function (LEDC channel mapping on ESP32).

### Step 2: Configure Frequency and Resolution

LED: 1–10 kHz, 8–10 bit. Servo: 50 Hz, ~16–20 bit for 1 µs resolution.

### Step 3: Set Initial Duty

0% for LED off; 7.5% (1.5 ms / 20 ms) for servo center.

### Step 4: Update Duty Smoothly

Ramp in small steps with timer-based delays for fade effect.

### Step 5: Verify with Scope or Phone Camera

Phone camera reveals flicker if frequency too low.

### Step 6: Servo Safety

Limit pulse width to 1–2 ms; don't stall servo against mechanical stop.

---

## Common Mistakes

| Mistake | Symptom | Fix |
|---------|---------|-----|
| PWM frequency too low | Visible flicker | Increase to ≥ 1 kHz for LEDs |
| Wrong duty range | LED always off/on | Use `max_duty()` from HAL |
| Servo on 3.3V power | Brownout, jitter | External 5 V supply |
| No common GND | Erratic servo | Tie all GND together |
| Updating duty mid-period | Glitch pulse | Use hardware latch/update |
| GPIO not muxed to LEDC | No output | Configure alternate function |

---

## Debugging Tips

1. **Scope on PWM pin** — verify frequency and duty cycle.
2. **Start at 50% duty** — easiest to detect with multimeter (average ~1.65 V).
3. **Servo without load first** — listen for smooth movement.
4. **Log duty values via `defmt`** — confirm software ramp logic.
5. **Check LEDC clock source** — wrong clock = wrong frequency.

---

## Performance Tips

| Tip | Detail |
|-----|--------|
| Hardware PWM always | Frees CPU |
| Batch duty updates | Once per frame, not every loop iteration |
| Match resolution to need | 8-bit enough for LEDs — saves register complexity |
| DMA + PWM for audio | Advanced — ESP32 I2S preferred over PWM for sound |
| Spread spectrum (advanced) | Reduce EMI on long wires |

---

## Exercises

### Exercise 1: Brightness Levels

Implement 10 discrete brightness levels via PWM duty lookup table.

### Exercise 2: Servo Sweep

Sweep servo 0°→180°→0° over 4 seconds using 50 Hz PWM.

### Exercise 3: RGB LED

Three PWM channels mix colors on common-cathode RGB LED.

### Exercise 4: Frequency Experiment

Try 100 Hz vs 5 kHz — photograph LED to show flicker difference.

### Exercise 5: Portable PWM

Write `fade<P: SetDutyCycle>(pwm: &mut P, steps: u32)` using embedded-hal.

---

## References

- [ESP32-S3 TRM — LED PWM Controller](https://www.espressif.com/sites/default/files/documentation/esp32-s3_technical_reference_manual_en.pdf)
- [embedded-hal SetDutyCycle](https://docs.rs/embedded-hal/1.0.0/embedded_hal/pwm/trait.SetDutyCycle.html)
- [STM32 RM — PWM mode](https://www.st.com/resource/en/reference_manual/rm0383-stm32f411-advanced-armbased-32bit-mcus-stmicroelectronics.pdf)
- [RP2040 Datasheet — PWM](https://datasheets.raspberrypi.com/rp2040/rp2040-datasheet.pdf)
- [Lesson 13 — DAC](./13-dac.md)

---

*Previous: [10-timers.md](./10-timers.md) | Next: [12-adc.md](./12-adc.md)*
