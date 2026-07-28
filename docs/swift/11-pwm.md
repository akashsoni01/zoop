# Lesson 11: PWM — Pulse Width Modulation

**PWM (Pulse Width Modulation)** rapidly switches a digital pin between HIGH and LOW. By varying **duty cycle** (fraction of period HIGH), you control average voltage — dim LEDs, set servo angles, or approximate analog output.

**Prerequisites:** [10-timers.md](./10-timers.md)  
**Next:** [12-adc.md](./12-adc.md)  
**See also:** [13-dac.md](./13-dac.md), [glossary.md](./glossary.md)

---

## Theory

### PWM Waveform

```
        ┌──────┐      ┌──────┐
        │      │      │      │
    ────┘      └──────┘      └────
        |<-- period -->|
        |<- duty ->|

Duty cycle = (HIGH time / period) × 100%
```

| Parameter | Symbol | Typical range |
|-----------|--------|---------------|
| Frequency | f | 100 Hz – 20 kHz (LED), 50 Hz (servo) |
| Duty cycle | D | 0% – 100% |
| Resolution | bits | 8–16 bits depending on timer |

### Applications

| Application | Frequency | Duty range |
|-------------|-----------|------------|
| LED dimming | 1–5 kHz (flicker-free) | 0–100% |
| Servo (SG90) | 50 Hz (20 ms period) | 1–2 ms pulse width |
| DC motor speed | 20 kHz+ | 0–100% |
| Audio (crude) | 44 kHz+ | PWM-DAC |

### Timer PWM Mode

Most MCUs drive PWM from timer **output compare** — hardware toggles GPIO without CPU:

```
Timer counter: 0 → ARR → 0 → ARR ...
                      ↑
Compare match (CCR) → pin goes HIGH or LOW
```

### Swift API Shape

```swift
public protocol PwmOutput {
    mutating func setFrequency(hz: UInt32) -> Result<Void, PwmError>
    mutating func setDutyPercent(_ percent: UInt8) -> Result<Void, PwmError>
    mutating func enable() -> Result<Void, PwmError>
    mutating func disable() -> Result<Void, PwmError>
}
```

Use `UInt8` percent (0–100) on embedded — avoid `Float` unless FPU present.

---

## Hardware Overview

### ESP32-S3 LEDC

- **LEDC (LED Control)** peripheral — 8 channels, up to 20-bit duty
- Independent frequency per timer group
- Preferred for LED dimming on ESP32

### STM32 TIM3 Channel 1

- Route TIM3_CH1 to GPIO pin via alternate function
- 16-bit counter — duty resolution depends on frequency

### RP2040 PWM Slice

- 8 PWM slices, 2 channels each
- Programmable divider and wrap

### Servo SG90

- Power: 5 V (may need external supply — not from 3.3 V GPIO)
- Signal: 3.3 V PWM usually OK
- 1.0 ms ≈ 0°, 1.5 ms ≈ 90°, 2.0 ms ≈ 180°

---

## Wiring Diagram

### LED PWM

```
ESP32-S3 GPIO4 ──[220Ω]──(LED)── GND

Software: LEDC channel 0 on GPIO4
Frequency ~5 kHz, duty 0-100%
```

### Servo SG90

```
                    ┌─────────────┐
    5V (external) ──┤ V+ (red)    │
    GPIO5 (PWM) ────┤ Signal (org)│  SG90
    GND ────────────┤ GND (brown) │
                    └─────────────┘

Common GND between servo supply and MCU required.
```

---

## Memory & Register Explanation

ESP32 LEDC registers (simplified):

| Register | Purpose |
|----------|---------|
| LEDC_CONF0 | Timer config |
| LEDC_HSTIMERx | High-speed timer |
| LEDC_CHx_CONF | Channel bind to GPIO |
| LEDC_CHx_DUTY | Duty value |
| LEDC_CHx_CONF1 | Update duty |

Duty update may require latch bit — read TRM for "duty start" sequence.

STM32: `TIMx_CCR1` holds compare value; `TIMx_ARR` holds period.

---

## HAL-Style Swift Implementation

```swift
public enum PwmError: Error {
    case invalidFrequency
    case invalidDuty
    case channelBusy
}

public protocol PwmOutput {
    mutating func configure(frequencyHz: UInt32, dutyPercent: UInt8) -> Result<Void, PwmError>
    mutating func setDutyPercent(_ percent: UInt8) -> Result<Void, PwmError>
}

struct Esp32LedPwm: PwmOutput {
    let gpio: UInt8
    let channel: UInt8

    mutating func configure(frequencyHz: UInt32, dutyPercent: UInt8) -> Result<Void, PwmError> {
        guard dutyPercent <= 100 else { return .failure(.invalidDuty) }
        guard frequencyHz >= 100 && frequencyHz <= 40000 else {
            return .failure(.invalidFrequency)
        }
        ledcBindChannel(channel: channel, gpio: gpio)
        ledcSetFrequency(timer: 0, hz: frequencyHz)
        ledcSetDuty(channel: channel, percent: dutyPercent)
        ledcStart(channel: channel)
        return .success(())
    }

    mutating func setDutyPercent(_ percent: UInt8) -> Result<Void, PwmError> {
        guard percent <= 100 else { return .failure(.invalidDuty) }
        ledcSetDuty(channel: channel, percent: percent)
        return .success(())
    }
}

// LED fade example
@main
struct PwmFadeApp {
    static func main() {
        var pwm = Esp32LedPwm(gpio: 4, channel: 0)
        _ = pwm.configure(frequencyHz: 5000, dutyPercent: 0)

        var increasing = true
        var duty: UInt8 = 0

        while true {
            _ = pwm.setDutyPercent(duty)
            delayMs(20)

            if increasing {
                if duty >= 100 { increasing = false }
                else { duty &+= 1 }
            } else {
                if duty == 0 { increasing = true }
                else { duty &-= 1 }
            }
        }
    }
}
```

Servo angle helper:

```swift
struct ServoAngle {
    var degrees: UInt8  // 0-180

    func pulseMicroseconds() -> UInt16 {
        // Linear map 0° → 1000 µs, 180° → 2000 µs
        let minUs: UInt16 = 1000
        let maxUs: UInt16 = 2000
        let range = maxUs - minUs
        return minUs + UInt16(degrees) * range / 180
    }
}
```

---

## Bare-Metal / MMIO Swift Notes

STM32 TIM3 PWM on PA6 (AF2):

```swift
func tim3PwmInit(frequencyHz: UInt32, dutyPercent: UInt8) {
    enableGpioAF(port: .a, pin: 6, af: 2)
    enableTim3Clock()

    let tim = tim3Registers()
    // 84 MHz timer clock example
    tim.pointee.psc = 0
    tim.pointee.arr = (84_000_000 / frequencyHz) - 1
    let ccr = (UInt32(dutyPercent) * (tim.pointee.arr + 1)) / 100
    tim.pointee.ccr1 = ccr

    // PWM mode 1, preload enable, channel enable — see TRM
    tim.pointee.ccmr1 = 0x0060
    tim.pointee.ccer = 0x0001
    tim.pointee.cr1 = 0x0081
}
```

---

## Step-by-Step Explanation

### Step 1: Choose Timer Channel and Pin

Check alternate function table — not all pins support PWM.

### Step 2: Enable Clocks and GPIO AF

Configure pin as alternate function, not plain output.

### Step 3: Set Frequency via ARR and Prescaler

Higher frequency → lower duty resolution.

### Step 4: Set Initial Duty

Start at 0% for LED (off).

### Step 5: Enable PWM Output

Start timer; verify waveform on scope.

### Step 6: Fade Loop

Increment/decrement duty in main loop using timer tick delays.

### Step 7: Servo Extension

Fix frequency at 50 Hz; vary pulse width 1–2 ms.

---

## Common Mistakes

| Mistake | Symptom | Fix |
|---------|---------|-----|
| Wrong alternate function | No output | Check AF table |
| Frequency too low for LED | Visible flicker | Use > 1 kHz |
| Servo on 3.3 V power | Weak movement | External 5 V supply |
| No common GND | Erratic servo | Tie grounds |
| Duty > resolution | Quantized steps | Increase ARR |
| Updating duty without latch | Glitch | Follow TRM update sequence |

---

## Debugging Tips

1. **Scope on PWM pin** — verify frequency and duty.
2. **Start at 50% duty** — easiest to see square wave.
3. **Multimeter DC mode** — average voltage ≈ 3.3 V × duty.
4. **Compare LED brightness** — monotonic with duty increase?
5. **Servo without load first** — listen for PWM whine at 50 Hz.

---

## Performance Tips

| Tip | Detail |
|-----|--------|
| Hardware PWM always | Never bit-bang LED dimming in loop |
| Batch duty updates | Update at 60 Hz human perception limit |
| Integer duty math | Avoid float on M0+ |
| DMA + PWM for audio | Advanced — see future lessons |
| LEDC vs software | ESP32: always prefer LEDC |

---

## Exercises

### Exercise 1: Fixed Duty

Set 25%, 50%, 75% duty; measure average voltage.

### Exercise 2: Smooth Fade

Implement fade over 3 seconds using timer tick pacing.

### Exercise 3: Servo Sweep

Sweep 0°–180°–0° with 50 Hz PWM.

### Exercise 4: Frequency Tradeoff

At fixed timer clock, document max duty resolution vs frequency.

### Exercise 5: Bare-Metal PWM

Configure one timer channel without HAL; verify on scope.

---

## References

- [ESP32-S3 TRM — LEDC](https://www.espressif.com/sites/default/files/documentation/esp32-s3_technical_reference_manual_en.pdf)
- [STM32 RM0383 — TIM PWM mode](https://www.st.com/resource/en/reference_manual/rm0383-stm32f411-advanced-armbased-32bit-mcus-stmicroelectronics.pdf)
- [RP2040 Datasheet — PWM](https://datasheets.raspberrypi.com/rp2040/rp2040-datasheet.pdf)
- [Lesson 12 — ADC](./12-adc.md)
- [Embedded Rust PWM](../learning/11-pwm.md)

---

*Previous: [10-timers.md](./10-timers.md) | Next: [12-adc.md](./12-adc.md)*
