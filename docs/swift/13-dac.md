# Lesson 13: DAC — Digital-to-Analog Conversion

A **DAC (Digital-to-Analog Converter)** converts a digital value into a continuous analog voltage. Generate waveforms, control analog circuits, or complement ADC readings for full analog I/O.

**Prerequisites:** [12-adc.md](./12-adc.md)  
**Next:** Extend to buses and wireless (see [00-roadmap.md](./00-roadmap.md) Phase 4)  
**See also:** [11-pwm.md](./11-pwm.md), [glossary.md](./glossary.md)

---

## Theory

### DAC vs PWM Analog Output

| Method | Pros | Cons |
|--------|------|------|
| **Hardware DAC** | Smooth, low ripple | Not on all MCUs |
| **PWM + filter** | Works anywhere | Needs RC filter, ripple |
| **R-2R ladder** | DIY | Discrete components |

Many boards lack a **true DAC pin** — STM32F411 has none; ESP32-S3 has no user DAC; RP2040 has no DAC. This lesson covers hardware DAC where available and **PWM-DAC** as universal fallback.

### DAC Basics

```
Digital value (0 – 2^N - 1) ──► DAC ──► V_out = (value / max) × Vref
```

| Parameter | Typical |
|-----------|---------|
| Resolution | 8–12 bits |
| Settling time | µs – µs |
| Output range | 0 – Vref (often 3.3 V) |

### Waveform Generation

Store one period of a sine in a lookup table (LUT):

```swift
let sineTable: [UInt16] = [
    2048, 2188, 2326, /* ... 64 entries ... */ 2048
]
```

Update DAC or PWM duty each timer tick — **Direct Digital Synthesis (DDS)** lite.

### Swift Fixed-Point

Avoid `sin()` from libm if unavailable — use precomputed LUT in `.rodata`:

```swift
struct SineGenerator {
    private var phaseIndex: UInt8 = 0
    let table: UnsafeBufferPointer<UInt16>

    mutating func nextSample() -> UInt16 {
        let sample = table[Int(phaseIndex)]
        phaseIndex &+= 1
        return sample
    }
}
```

---

## Hardware Overview

### STM32F407 (has DAC — comparison)

- DAC1 on PA4, DAC2 on PA5
- 12-bit, buffer optional
- F411 **does not** include DAC — use PWM-DAC on Nucleo-F411

### ESP32-S3

- No exposed DAC — use **LEDC PWM + RC filter** or external I²C DAC (MCP4725)

### RP2040

- No DAC — PWM slice + filter

### PWM-DAC Filter

```
GPIO (PWM) ──[1kΩ]──┬──► V_analog (~ smoothed)
                    │
                   [10µF]
                    │
                   GND

Cutoff f_c ≈ 1 / (2π × R × C) ≈ 16 Hz for 1kΩ + 10µF
PWM frequency should be >> f_c (e.g., 20 kHz)
```

---

## Wiring Diagram

### External I²C DAC (MCP4725) — Universal

```
    ESP32-S3              MCP4725
  ┌─────────┐           ┌─────────┐
  │ GPIO8   ├─ SDA ────►│ SDA     │
  │ GPIO9   ├─ SCL ────►│ SCL     │
  │ 3V3     ├──────────►│ VDD     │
  │ GND     ├──────────►│ GND     │
  └─────────┘           │ VOUT ───┼──► Scope / amplifier
                        └─────────┘
```

### PWM-DAC to Scope

```
GPIO4 (PWM 20kHz) ──[1kΩ]──[10µF]── GND
                      │
                      └── probe here (analog ~0–3.3 V)
```

---

## Memory & Register Explanation

STM32 DAC register (F407 reference for learning):

| Register | Purpose |
|----------|---------|
| DAC_CR | Enable, trigger |
| DAC_DHR12R1 | 12-bit right-aligned data |
| DAC_DOR1 | Output register (actual voltage) |

LUT lives in flash:

```swift
// 64-point sine, 12-bit centered at 2048, amplitude 2047
let sineLut: (UInt16, UInt16, UInt16, UInt16, /* ... */) = (
    2048, 2148, 2248, /* ... */
)
// Stored in .rodata — no RAM cost for constants
```

---

## HAL-Style Swift Implementation

```swift
public enum DacError: Error {
    case notSupported
    case invalidValue
    case busError
}

public protocol DacChannel {
    mutating func writeRaw(_ value: UInt16, maxValue: UInt16) -> Result<Void, DacError>
    mutating func writeMillivolts(_ mv: UInt16, vrefMv: UInt16) -> Result<Void, DacError>
}

// PWM-DAC fallback — works on ESP32-S3
struct PwmDac: DacChannel {
    var pwm: Esp32LedPwm

    mutating func writeRaw(_ value: UInt16, maxValue: UInt16) -> Result<Void, DacError> {
        let percent = UInt8(min(100, UInt32(value) * 100 / UInt32(maxValue)))
        return pwm.setDutyPercent(percent)
    }

    mutating func writeMillivolts(_ mv: UInt16, vrefMv: UInt16) -> Result<Void, DacError> {
        let raw = UInt16((UInt32(mv) * 4095) / UInt32(vrefMv))
        return writeRaw(raw, maxValue: 4095)
    }
}

// Sine wave player
struct WaveformPlayer<D: DacChannel> {
    var dac: D
    var generator: SineGenerator
    var sampleRateHz: UInt32

    mutating func tick() -> Result<Void, DacError> {
        let sample = generator.nextSample()
        return dac.writeRaw(sample, maxValue: 4095)
    }
}

@main
struct DacSineApp {
    static func main() {
        var pwm = Esp32LedPwm(gpio: 4, channel: 0)
        _ = pwm.configure(frequencyHz: 20000, dutyPercent: 50)
        var dac = PwmDac(pwm: pwm)

        let table = sineLookupTable64()
        var gen = SineGenerator(table: table)
        var player = WaveformPlayer(dac: dac, generator: gen, sampleRateHz: 1000)

        var timer = Stm32PeriodicTimer()
        _ = timer.start(periodMs: 1)  // 1 kHz sample rate

        var lastSample = timer.elapsedMs()
        while true {
            let now = timer.elapsedMs()
            if now &- lastSample >= 1 {
                _ = player.tick()
                lastSample = now
            }
        }
    }
}
```

I²C DAC protocol sketch:

```swift
struct Mcp4725Dac: DacChannel {
    let i2cAddress: UInt8

    mutating func writeRaw(_ value: UInt16, maxValue: UInt16) -> Result<Void, DacError> {
        let scaled = UInt16((UInt32(value) * 4095) / UInt32(maxValue)) & 0x0FFF
        var buf: (UInt8, UInt8) = (
            UInt8((scaled >> 8) & 0x0F) | 0x40,  // fast mode command
            UInt8(scaled & 0xFF)
        )
        return i2cWrite(address: i2cAddress, bytes: &buf)
            .mapError { _ in .busError }
    }

    mutating func writeMillivolts(_ mv: UInt16, vrefMv: UInt16) -> Result<Void, DacError> {
        let raw = UInt16((UInt32(mv) * 4095) / UInt32(vrefMv))
        return writeRaw(raw, maxValue: 4095)
    }
}
```

---

## Bare-Metal / MMIO Swift Notes

STM32F407 DAC1 enable (educational — F411 users skip to PWM-DAC):

```swift
let DAC_BASE: UInt = 0x4000_7400

func dac1Write12(value: UInt16) {
    enableDac1Clock()
    let dac = UnsafeMutablePointer<UInt32>(bitPattern: DAC_BASE)!
    dac.advanced(at: 0x08 / 4).pointee = UInt32(value & 0xFFF)  // DHR12R1
    dac.advanced(at: 0x2C / 4).pointee = 1  // CR EN1
}
```

PWM-DAC bare — set LEDC duty proportional to sample:

```swift
func pwmDacOutput(raw12: UInt16) {
    let percent = UInt8((UInt32(raw12) * 100) / 4095)
    ledcSetDuty(channel: 0, percent: percent)
}
```

---

## Step-by-Step Explanation

### Step 1: Check MCU Capability

Read datasheet — true DAC or PWM-only?

### Step 2: Choose Output Method

Hardware DAC > I²C DAC > PWM-DAC filter.

### Step 3: Build Sine LUT

Generate 64–256 points offline; paste into Swift as constants.

### Step 4: Configure High-Frequency PWM (if PWM-DAC)

20 kHz+ to simplify filtering.

### Step 5: Timer Tick at Sample Rate

1 kHz audio-ish demo = 1 ms tick.

### Step 6: Output Sample Each Tick

Write DAC or PWM duty from LUT index.

### Step 7: Verify on Scope

Expect smooth sine (DAC) or ripple (PWM-DAC) — adjust filter.

---

## Common Mistakes

| Mistake | Symptom | Fix |
|---------|---------|-----|
| PWM frequency too low | Visible ripple on analog | Increase to 20 kHz+ |
| No filter on PWM-DAC | Square-ish analog | Add RC low-pass |
| LUT too small | Distorted sine | Increase to 64+ points |
| Wrong Vref assumption | Incorrect amplitude | Measure Vref |
| I²C DAC address wrong | No output | Scan bus 0x60–0x67 |
| Sample rate jitter | Audio warble | Hardware timer ISR |

---

## Debugging Tips

1. **Scope on DAC output** — verify waveform shape and frequency.
2. **DC test** — output 0%, 50%, 100% duty; measure DC levels.
3. **Freeze LUT index** — output constant voltage step.
4. **Compare FFT** — PWM-DAC shows harmonics; true DAC cleaner.
5. **Headphones via amp** — only with proper analog stage — not direct GPIO.

---

## Performance Tips

| Tip | Detail |
|-----|--------|
| LUT in flash | `.rodata` — zero RAM |
| Integer samples | No libm sin() |
| DMA to DAC | STM32F407 advanced path |
| Double-buffer LUT | Glitch-free period wrap |
| Match sample rate to timer | Integer tick division |

---

## Exercises

### Exercise 1: DC Levels

Output 25%, 50%, 75% on PWM-DAC; measure with multimeter.

### Exercise 2: Sine on Scope

Generate 100 Hz sine; measure period on scope.

### Exercise 3: Triangle Wave

Replace LUT with triangle ramp function.

### Exercise 4: ADC ↔ DAC Loop

Connect DAC output to ADC input (verify levels safe); log round-trip.

### Exercise 5: I²C DAC

If MCP4725 available, output same sine via I²C; compare quality to PWM-DAC.

---

## References

- [STM32F407 RM0090 — DAC chapter](https://www.st.com/resource/en/reference_manual/dm00031020-stm32f407-advanced-arm-based-32-bit-mcus-stmicroelectronics.pdf)
- [MCP4725 Datasheet](https://ww1.microchip.com/downloads/en/DeviceDoc/22039d.pdf)
- [ESP32-S3 TRM — LEDC (PWM-DAC)](https://www.espressif.com/sites/default/files/documentation/esp32-s3_technical_reference_manual_en.pdf)
- [Lesson 12 — ADC](./12-adc.md)
- [Embedded Rust DAC](../learning/13-dac.md)

---

*Previous: [12-adc.md](./12-adc.md) | Next: [00-roadmap.md](./00-roadmap.md) Phase 4*
