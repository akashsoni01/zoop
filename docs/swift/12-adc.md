# Lesson 12: ADC — Analog-to-Digital Conversion

An **ADC (Analog-to-Digital Converter)** samples an analog voltage and produces a digital number. Read sensors — potentiometers, temperature dividers, light-dependent resistors — and close the loop with PWM output.

**Prerequisites:** [10-timers.md](./10-timers.md)  
**Next:** [13-dac.md](./13-dac.md)  
**See also:** [11-pwm.md](./11-pwm.md), [08-gpio.md](./08-gpio.md)

---

## Theory

### Why ADC?

MCU GPIO reads **digital** levels (HIGH/LOW). Real-world signals are **analog** — continuous voltage. ADC bridges the gap:

```
Analog voltage (0 – Vref) ──► ADC ──► Digital value (0 – 2^N - 1)
```

| Term | Meaning |
|------|---------|
| **Resolution** | Bits (8, 10, 12) — counts = 2^N |
| **Vref** | Reference voltage — full scale maps to this |
| **Sample time** | How long input is sampled — affects accuracy |
| **Sampling rate** | Conversions per second (Hz) |
| **SNR (Signal-to-Noise Ratio)** | Quality metric |

### Voltage Calculation

```
V_in = (raw / max_raw) × Vref

Example: 12-bit ADC, Vref = 3.3 V, raw = 2048
V_in = (2048 / 4095) × 3.3 ≈ 1.65 V
```

Use integer math on MCU:

```swift
func voltageMilliVolts(raw: UInt16, vrefMv: UInt16, maxRaw: UInt16) -> UInt16 {
    return UInt16((UInt32(raw) * UInt32(vrefMv)) / UInt32(maxRaw))
}
```

### Sampling Methods

| Method | Use |
|--------|-----|
| **Single conversion** | One-shot read |
| **Continuous + DMA** | Streaming (advanced) |
| **Oversampling** | Average N samples for noise reduction |

### ESP32-S3 ADC Caveats

- Non-linear at extremes — calibration recommended
- **Attenuation** setting extends input range (0–3.3 V typical)
- Wi-Fi activity can add noise — average samples

---

## Hardware Overview

### ESP32-S3 SAR ADC

- Two ADC units, multiple channels
- 12-bit resolution (0–4095)
- Attenuation: 0 dB, 2.5 dB, 6 dB, 12 dB

### STM32F411 ADC1

- 12-bit, multiple channels on GPIO
- Vref often tied to 3.3 V on Nucleo

### RP2040 ADC

- 4 channels, 12-bit
- Internal 0.7 V reference option — see datasheet

---

## Wiring Diagram

### Potentiometer as Voltage Divider

```
    3V3 ────┬────────────────────────────
            │
           ╱╲
          ╱  ╲  10 kΩ potentiometer
         ╱    ╲
        ╱      ╲
    ───┴────────┴───► GPIO (ADC input, e.g., GPIO1)
        │      │
       GND    wiper

Wiper voltage: 0 V (CCW) to 3.3 V (CW)
```

### Light Sensor (LDR) Divider

```
    3V3 ────[ LDR ]────┬──── GPIO ADC
                       │
                      [10kΩ]
                       │
                      GND
```

Always keep input within 0 – Vref. **Never exceed 3.3 V** on ESP32/STM32 ADC pins.

---

## Memory & Register Explanation

ADC result typically in a data register or FIFO:

| MCU | Result register | Clear/read pattern |
|-----|-----------------|-------------------|
| ESP32 | `SENS` block | Start conversion, poll done |
| STM32 | `ADC_DR` | Read clears EOC flag |
| RP2040 | `FIFO` | Pop sample |

Store calibrated values in flash `.rodata` as lookup tables if needed:

```swift
let calibrationOffset: Int16 = -42
let calibrationScale: UInt16 = 4096
```

---

## HAL-Style Swift Implementation

```swift
public enum AdcError: Error {
    case invalidChannel
    case conversionTimeout
    case notCalibrated
}

public protocol AdcChannel {
    func readRaw() -> Result<UInt16, AdcError>
    func readMillivolts() -> Result<UInt16, AdcError>
}

struct Esp32AdcChannel: AdcChannel {
    let unit: UInt8
    let channel: UInt8
    let attenuation: AdcAttenuation

    func readRaw() -> Result<UInt16, AdcError> {
        adcConfigure(unit: unit, channel: channel, atten: attenuation)
        guard let raw = adcPollConversion(unit: unit, timeoutUs: 10_000) else {
            return .failure(.conversionTimeout)
        }
        return .success(raw)
    }

    func readMillivolts() -> Result<UInt16, AdcError> {
        readRaw().map { raw in
            // 12 dB attenuation ≈ 0–3300 mV on ESP32-S3
            voltageMilliVolts(raw: raw, vrefMv: 3300, maxRaw: 4095)
        }
    }
}

// Pot-controlled PWM brightness
@main
struct AdcPwmApp {
    static func main() {
        var pwm = Esp32LedPwm(gpio: 4, channel: 0)
        _ = pwm.configure(frequencyHz: 5000, dutyPercent: 0)

        let pot = Esp32AdcChannel(unit: 1, channel: 0, attenuation: .db12)

        while true {
            if case .success(let mv) = pot.readMillivolts() {
                // Map 0–3300 mV → 0–100%
                let duty = UInt8(min(100, mv * 100 / 3300))
                _ = pwm.setDutyPercent(duty)
                logMv(mv)
            }
            delayMs(50)
        }
    }
}
```

Oversampling helper:

```swift
func readAveraged<C: AdcChannel>(_ channel: C, samples: UInt8) -> Result<UInt16, AdcError> {
    var sum: UInt32 = 0
    for _ in 0..<samples {
        switch channel.readRaw() {
        case .success(let raw): sum += UInt32(raw)
        case .failure(let e): return .failure(e)
        }
    }
    return .success(UInt16(sum / UInt32(samples)))
}
```

---

## Bare-Metal / MMIO Swift Notes

STM32 ADC1 single conversion (simplified):

```swift
let ADC1_BASE: UInt = 0x4001_2000

func adc1Init() {
    enableAdc1Clock()
    enableGpioAnalogMode(port: .a, pin: 0)  // PA0 = ADC1_IN0

    let adc = UnsafeMutablePointer<UInt32>(bitPattern: ADC1_BASE)!
    // Enable ADC, calibration sequence per RM0383
    adc.advanced(at: 0x08 / 4).pointee = 1  // CR2 ADON
}

func adc1ReadChannel0() -> UInt16? {
    let adc = UnsafeMutablePointer<UInt32>(bitPattern: ADC1_BASE)!
    // Start conversion, wait EOC, read DR
    adc.advanced(at: 0x08 / 4).pointee |= 1 << 30  // SWSTART
    var timeout = 100_000
    while (adc.advanced(at: 0x00 / 4).pointee & (1 << 1)) == 0 && timeout > 0 {
        timeout -= 1
    }
    guard timeout > 0 else { return nil }
    return UInt16(adc.advanced(at: 0x4C / 4).pointee & 0xFFF)
}
```

---

## Step-by-Step Explanation

### Step 1: Configure GPIO as Analog

Disable digital input/output on ADC pin — set analog mode in MODER or equivalent.

### Step 2: Set ADC Resolution and Reference

12-bit, Vref = VDDA (usually 3.3 V).

### Step 3: Select Channel

Match physical pin to ADC channel in datasheet table.

### Step 4: Calibrate (if required)

STM32 and ESP32 benefit from factory or runtime calibration.

### Step 5: Trigger Conversion

Software trigger for learning; timer trigger for streaming.

### Step 6: Read and Convert

Apply voltage formula; log over UART.

### Step 7: Close Loop

Map reading to PWM duty or threshold action.

---

## Common Mistakes

| Mistake | Symptom | Fix |
|---------|---------|-----|
| Digital mode on ADC pin | Wrong readings | Set analog mode |
| Input > 3.3 V | Damaged ADC | Voltage divider |
| No sample time for high source impedance | Noisy readings | Increase sample cycles |
| Floating input | Random values | Tie divider, enable pull-down |
| Ignoring ESP32 non-linearity | Inaccurate at extremes | Use calibration API |
| Single sample | Noise | Average 8–16 samples |

---

## Debugging Tips

1. **Multimeter on ADC pin** — compare with computed voltage.
2. **Raw value only first** — verify 0–4095 range before scaling.
3. **Slow UART log** — plot raw vs pot position.
4. **Short wires** — long jumper wires pick up noise.
5. **Repeat with known voltage** — divide from 3.3 V and GND with resistors.

---

## Performance Tips

| Tip | Detail |
|-----|--------|
| Oversample in software | Cheap noise reduction |
| Integer millivolts | Avoid float on M0+ |
| Timer-triggered ADC | Regular sample rate |
| DMA for streaming | Audio / data logging |
| Disable Wi-Fi during sensitive reads | ESP32 noise reduction |

---

## Exercises

### Exercise 1: Raw Read

Log raw ADC value while turning pot; verify monotonic increase.

### Exercise 2: Millivolt Display

Convert to mV; compare with multimeter at three positions.

### Exercise 3: Threshold LED

Turn LED on when pot > 50% (use GPIO or PWM threshold).

### Exercise 4: Averaging

Compare single sample vs 16-sample average noise on scope.

### Exercise 5: ADC → PWM

Map pot position to LED brightness continuously.

---

## References

- [ESP32-S3 TRM — SAR ADC](https://www.espressif.com/sites/default/files/documentation/esp32-s3_technical_reference_manual_en.pdf)
- [STM32 RM0383 — ADC](https://www.st.com/resource/en/reference_manual/rm0383-stm32f411-advanced-armbased-32bit-mcus-stmicroelectronics.pdf)
- [RP2040 Datasheet — ADC](https://datasheets.raspberrypi.com/rp2040/rp2040-datasheet.pdf)
- [Lesson 13 — DAC](./13-dac.md)
- [Embedded Rust ADC](../learning/12-adc.md)

---

*Previous: [11-pwm.md](./11-pwm.md) | Next: [13-dac.md](./13-dac.md)*
