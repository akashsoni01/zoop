# Lesson 12: ADC — Analog-to-Digital Conversion

An **ADC (Analog-to-Digital Converter)** measures analog voltage and produces a digital number. Read potentiometers, sensors, batteries, and more.

**Prerequisites:** [10-timers.md](./10-timers.md)  
**Next:** [13-dac.md](./13-dac.md)  
**See also:** [08-gpio.md](./08-gpio.md), [11-pwm.md](./11-pwm.md)

---

## Theory

### How ADC Works (Simplified)

```
Analog voltage ──► Sample & Hold ──► Quantizer ──► Digital value (0 … 2^n - 1)
                         │
                    Reference voltage (Vref)
```

The ADC compares input voltage against **Vref** and outputs a proportional integer.

### Key Parameters

| Parameter | Meaning |
|-----------|---------|
| **Resolution** | Bits — ESP32-S3 SAR ADC: 12-bit (0–4095) |
| **Vref** | Reference voltage — often VDD (3.3 V) or internal 1.1 V |
| **Sample rate** | Conversions per second (Hz) |
| **INL/DNL** | Linearity errors — check datasheet |
| **Input impedance** | High-Z — don't drive heavy loads |

Voltage calculation:

```
V_in = (raw_count / max_count) × Vref

Example: raw=2048, 12-bit, Vref=3.3V
V_in = (2048 / 4095) × 3.3 ≈ 1.65 V
```

### Sampling Theorem (Brief)

To accurately capture a signal, sample at **≥ 2× highest frequency** (Nyquist). For slow pot movement, 10–100 Hz suffices. Audio needs kHz rates.

---

## Hardware Overview

### ESP32-S3 SAR ADC

- Two ADC units (ADC1, ADC2)
- 12-bit resolution (configurable attenuation)
- **Attenuation** extends measurable range:

| Attenuation | Approx Range |
|-------------|--------------|
| 0 dB | 0 – 0.8 V |
| 2.5 dB | 0 – 1.1 V |
| 6 dB | 0 – 1.5 V |
| 12 dB | 0 – 3.3 V |

Use **12 dB** for potentiometer tied to 3.3 V.

**Note:** ADC2 conflicts with Wi-Fi on ESP32 — use ADC1 when radio active.

### STM32F411 ADC (Comparison)

12-bit, multiple channels via scan mode, DMA support for continuous sampling.

### RP2040 ADC (Comparison)

12-bit, 4 channels, 500 ksps max, internal 3.3 V reference, optional Vsys sense.

---

## Wiring Diagram

### Potentiometer Voltage Divider

```
        3V3
         │
         │
    ┌────┴────┐
    │  POT    │
    │  10kΩ   │
    └────┬────┘
         │
         ├──────────────► GPIO6 (ADC1_CH5 on ESP32-S3 — verify pin!)
         │                (wiper — variable voltage 0–3.3V)
         │
        GND

Fixed terminals: one to 3V3, one to GND.
Wiper (middle pin) to ADC GPIO.
```

### Direct Sensor (e.g., LM35 — 10 mV/°C)

```
LM35 Vout ──────► ADC pin
LM35 Vcc  ──────► 3V3
LM35 GND  ──────► GND

Keep wires short; add 100 nF cap near sensor for noise filtering.
```

---

## Memory & Register Explanation

### ESP32 ADC Result Registers

| Register | Purpose |
|----------|---------|
| `SENS_SAR_READER1_CTRL` | Start conversion |
| `SENS_SAR_MEAS_START1` | Channel select, start |
| `SENS_SAR_READER1_STATUS` | Done flag |
| `SENS_SAR_READER1_RESULT` | 12-bit result in lower bits |

HAL abstracts polling vs async read.

### Averaging Buffer (Software)

```rust
/// Circular buffer for moving average — fixed size, no heap.
/// Stored in .bss — 16 × u16 = 32 bytes RAM.
struct AvgFilter<const N: usize> {
    buf: [u16; N],
    idx: usize,
}

impl<const N: usize> AvgFilter<N> {
    fn new() -> Self {
        Self { buf: [0; N], idx: 0 }
    }

    fn push(&mut self, sample: u16) -> u16 {
        self.buf[self.idx] = sample;
        self.idx = (self.idx + 1) % N;
        let sum: u32 = self.buf.iter().map(|&x| x as u32).sum();
        (sum / N as u32) as u16
    }
}
```

Ownership: filter owned by main loop; `&mut self` in `push` — exclusive access.

---

## HAL Implementation

ESP32-S3 potentiometer read:

```rust
use esp_hal::adc::{Adc, AdcConfig, Attenuation, ADC};
use esp_hal::gpio::Io;
use esp_hal::peripherals::Peripherals;
use defmt::info;

#[esp_hal::macros::entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let clocks = /* init */;
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    let mut adc_config = AdcConfig::new();
    // Configure GPIO6 as ADC input with 12 dB attenuation (0–3.3V)
    let mut pin = io.pins.gpio6.into_analog();
    adc_config.enable_pin(&mut pin, Attenuation::Attenuation12dB);

    let mut adc = Adc::new(peripherals.ADC, adc_config);

    let mut filter = AvgFilter::<16>::new();

    loop {
        // `read` returns u16 — ownership of result is local
        let raw = adc.read(&mut pin).unwrap();
        let filtered = filter.push(raw);

        // Convert to millivolts (approximate)
        let mv = (filtered as u32 * 3300 / 4095) as u16;
        let percent = (filtered as u32 * 100 / 4095) as u8;

        info!("ADC raw={} mv={} percent={}%", raw, mv, percent);

        // Rate-limit reads — 10 Hz sufficient for pot
        for _ in 0..100_000 { core::hint::spin_loop(); }
    }
}
```

### embedded-hal ADC Trait

```rust
use embedded_hal::adc::OneShot;

fn read_voltage<A: OneShot>(adc: &mut A, channel: &mut A::Channel) -> Result<u16, A::Error> {
    adc.read(channel)
}
```

STM32:

```rust
let mut adc = Adc::adc1(dp.ADC1, true);
let mut pin = gpioa.pa0.into_analog();
let sample: u16 = adc.read(&mut pin).unwrap();
```

---

## Bare-Metal Implementation

Conceptual ESP32 ADC1 read (polling):

```rust
unsafe fn adc1_read_channel5() -> u16 {
    const SENS_BASE: u32 = 0x6000_8800;
    // 1. Configure attenuation for channel in SENS SAR ATTEN register
    // 2. Select channel in meas start register
    // 3. Trigger conversion
    // 4. Poll status until done
    // 5. Read result register, mask lower 12 bits

    let result_reg = (SENS_BASE + 0x058) as *const u32;
    let raw = core::ptr::read_volatile(result_reg) & 0xFFF;
    raw as u16
}
```

Exact register sequence spans multiple pages in TRM — use HAL for production.

---

## Step-by-Step Explanation

### Step 1: Identify ADC-Capable Pin

Not all GPIOs support ADC. Check pin mux table in TRM / board docs.

### Step 2: Configure GPIO as Analog Input

Disable digital input buffer to prevent leakage.

### Step 3: Set Attenuation / Reference

Match expected voltage range — 12 dB for 0–3.3 V pot.

### Step 4: Calibrate (If Supported)

ESP32 supports offset calibration — HAL may call `adc_calibrate()`.

### Step 5: Read and Convert

Raw → millivolts → engineering units (temperature, etc.).

### Step 6: Filter Noise

Moving average or median filter for stable readings.

### Step 7: Rate-Limit Sampling

Pot doesn't need 1 MHz sampling — saves CPU and reduces noise.

---

## Common Mistakes

| Mistake | Symptom | Fix |
|---------|---------|-----|
| Wrong attenuation | Clipping at max/min | Match input voltage range |
| Floating ADC pin | Random values | Tie to known source or enable pull-down |
| ADC2 + Wi-Fi on ESP32 | Read failures | Use ADC1 |
| No filtering | Noisy readings | Average 8–16 samples |
| High source impedance | Incorrect readings | Buffer with op-amp or lower pot value |
| Exceeding 3.3 V input | **Damages chip** | Voltage divider for higher voltages |

---

## Debugging Tips

1. **Multimeter on ADC pin** — compare with software conversion.
2. **Log raw counts** — distinguish wiring vs conversion bugs.
3. **Short input to GND / 3V3** — expect ~0 / ~4095.
4. **Oscilloscope** — detect noise pickup on long wires.
5. **Graph readings** — rotate pot smoothly; curve should be monotonic.

---

## Performance Tips

| Tip | Detail |
|-----|--------|
| DMA continuous mode (STM32) | CPU-free streaming |
| Oversample + decimate | Improve effective resolution |
| Lower sample rate for slow signals | Less noise, less CPU |
| Differential ADC (if available) | Reject common-mode noise |
| Batch reads in timer ISR | Fixed sample rate for DSP |

---

## Exercises

### Exercise 1: Voltage Meter

Display millivolts via `defmt`; verify with multimeter at 5 positions.

### Exercise 2: PWM Brightness Control

Link pot ADC value to LED PWM duty ([11-pwm.md](./11-pwm.md)).

### Exercise 3: Filter Comparison

Compare no filter vs 4-sample vs 16-sample average — quantify noise.

### Exercise 4: Calibration Table

Build 5-point calibration correcting offset/gain error.

### Exercise 5: Battery Monitor

Measure voltage divider on USB/ battery input; warn below threshold.

---

## References

- [ESP32-S3 TRM — SAR ADC](https://www.espressif.com/sites/default/files/documentation/esp32-s3_technical_reference_manual_en.pdf)
- [embedded-hal OneShot ADC](https://docs.rs/embedded-hal/1.0.0/embedded_hal/adc/trait.OneShot.html)
- [STM32 RM — ADC](https://www.st.com/resource/en/reference_manual/rm0383-stm32f411-advanced-armbased-32bit-mcus-stmicroelectronics.pdf)
- [RP2040 Datasheet — ADC](https://datasheets.raspberrypi.com/rp2040/rp2040-datasheet.pdf)
- [Lesson 13 — DAC](./13-dac.md)

---

*Previous: [11-pwm.md](./11-pwm.md) | Next: [13-dac.md](./13-dac.md)*
