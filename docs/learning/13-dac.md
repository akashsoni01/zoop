# Lesson 13: DAC — Digital-to-Analog Conversion

A **DAC (Digital-to-Digital Converter)** converts a digital number into an analog voltage. Generate waveforms, control analog circuits, or complement ADC readings for closed-loop systems.

**Prerequisites:** [12-adc.md](./12-adc.md)  
**Next:** Return to [00-roadmap.md](./00-roadmap.md) for advanced topics  
**See also:** [11-pwm.md](./11-pwm.md), [10-timers.md](./10-timers.md)

---

## Theory

### DAC vs PWM

| Method | Pros | Cons |
|--------|------|------|
| **True DAC** | Smooth analog output, low ripple | Requires dedicated hardware |
| **PWM + filter** | Works on any GPIO | Needs RC filter; ripple; CPU/timer load |
| **R2R ladder (external)** | DIY DAC | Discrete components; accuracy limited |

Many MCUs lack a true DAC — **PWM with low-pass filter** is the common workaround.

### DAC Parameters

| Parameter | Meaning |
|-----------|---------|
| **Resolution** | Bits — e.g., 8-bit → 256 levels |
| **Vref** | Output scales to Vref (often 3.3 V) |
| **Settling time** | Delay until output stable |
| **Output drive** | Max current — often needs buffer op-amp |

Output voltage:

```
V_out = (digital_value / (2^n - 1)) × Vref
```

### Waveform Generation

Periodic waveforms via **DMA + timer + DAC** (or timer interrupt updating DAC):

```
Sine lookup table [256 x u8] in .rodata (flash)
        │
        ▼
Timer ISR / DMA ──► DAC register ──► analog output
```

---

## Hardware Overview

### ESP32-S3

The ESP32-S3 **does not include a true DAC**. Options:

1. **PWM + RC low-pass filter** — practical for slow signals
2. **I2S + external DAC chip** (PCM5102) — audio quality
3. **SPI DAC** (MCP4921) — 12-bit external

This lesson demonstrates **PWM-DAC** on ESP32-S3 and notes **STM32 internal DAC**.

### STM32F411 (Comparison)

Built-in **12-bit DAC** on PA4 (DAC1) and PA5 (DAC2):

- Buffer enable for lower output impedance
- DMA support for waveform streaming
- ~1 MS/s update rate

### RP2040 (Comparison)

No true DAC — use PWM slices or external chip.

---

## Wiring Diagram

### PWM + RC Low-Pass Filter (ESP32-S3)

```
ESP32-S3
┌──────────────┐
│  GPIO4 (PWM) ├──────┬───────► To op-amp or load
│              │      │
│              │     [R]  1 kΩ
│              │      │
│              │      ├──────► Vout (smooth DC-ish)
│              │      │
│              │     [C]  10 µF
│              │      │
│  GND         ├──────┴────── GND
└──────────────┘

Cutoff frequency: fc ≈ 1 / (2π × R × C)
With R=1kΩ, C=10µF: fc ≈ 16 Hz — good for slow waveforms, not audio.

For audio: fc ≈ 20 kHz → smaller C or active filter.
```

### STM32 Internal DAC Output

```
STM32 PA4 (DAC1_OUT)
    │
    ├──────► Scope probe / op-amp input
    │
   [100nF] optional smoothing cap to GND
    │
   GND
```

### External SPI DAC (MCP4921)

```
ESP32                MCP4921
GPIO11 (MOSI) ──────► SDI
GPIO12 (SCK)  ──────► SCK
GPIO10 (CS)   ──────► CS
3V3           ──────► VDD
GND           ──────► VSS
                     Vout ──► load
```

---

## Memory & Register Explanation

### Sine Lookup Table in Flash

```rust
/// 256-entry sine table — stored in .rodata, zero RAM cost.
/// Values 0–255 centered around 128 for 8-bit PWM/DAC.
static SINE_TABLE: [u8; 256] = generate_sine_table();

const fn generate_sine_table() -> [u8; 256] {
    let mut table = [128u8; 256];
    let mut i = 0;
    while i < 256 {
        // Approximate sin with piecewise — production: use libm at compile time
        // Placeholder: triangle wave for illustration
        table[i] = (128 + (i as i32 * 127 / 128 - 127)) as u8;
        i += 1;
    }
    table
}
```

Index advances in timer ISR — `idx` wraps at 256 for periodic waveform.

### STM32 DAC Registers

| Register | Purpose |
|----------|---------|
| `DAC_CR` | Enable, buffer, trigger |
| `DAC_DHR12R1` | 12-bit right-aligned data for channel 1 |
| `DAC_DOR1` | Output data register (actual output) |

---

## HAL Implementation

### ESP32-S3 — Sine Wave via PWM (LEDC)

```rust
use esp_hal::ledc::{channel, timer, LowSpeed, LEDC, LSChannel, LSTimer};
use esp_hal::gpio::Io;
use esp_hal::peripherals::Peripherals;

/// Phase index — modified in timer ISR, read in main or ISR.
/// Atomic for safe ISR/main sharing on single-core access pattern.
static WAVE_INDEX: core::sync::atomic::AtomicUsize =
    core::sync::atomic::AtomicUsize::new(0);

static SINE_TABLE: [u8; 256] = generate_sine_table();

const fn generate_sine_table() -> [u8; 256] {
    let mut t = [128u8; 256];
    let mut i = 0usize;
    while i < 256 {
        // Simple quarter-sine approximation for demo
        t[i] = (128 + (i as i32 - 128) * 127 / 128) as u8;
        i += 1;
    }
    t
}

#[esp_hal::macros::entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let clocks = /* init */;
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    let ledc = LEDC::new(peripherals.LEDC, &clocks);
    let mut timer = ledc.get_timer::<LowSpeed>(LSTimer::Timer0);
    let mut channel = ledc.get_channel::<LowSpeed>(
        LSChannel::Channel0,
        io.pins.gpio4,
    );

    // High PWM frequency + RC filter = smoother analog
    timer.configure(timer::config::TimerConfig::default().frequency(20.kHz())).unwrap();
    channel.configure(channel::config::ChannelConfig::default()).unwrap();

    // Timer ISR updates PWM duty from table — see [10-timers.md](./10-timers.md)
    setup_waveform_timer(/* ... */);

    loop {
        core::hint::spin_loop();
    }
}

#[esp_hal::macros::handler]
fn waveform_timer_isr() {
    let idx = WAVE_INDEX.fetch_add(1, core::sync::atomic::Ordering::Relaxed) & 0xFF;
    let duty = SINE_TABLE[idx] as u32;
    // Set LEDC duty — API call; keep ISR short
    // channel.set_duty(duty) — requires static or global channel ref pattern
}
```

**Note:** Sharing `channel` with ISR requires careful static initialization (`StaticCell`) — see [09-interrupts.md](./09-interrupts.md).

### STM32 Internal DAC

```rust
use stm32f4xx_hal::dac::{Dac, DacChannel, WaveGenerator, SignalGenerator};

let dac = Dac::new(dp.DAC, &clocks);
let mut dac1 = dac.channel1.pa4;

dac1.set_value(2048);  // Mid-scale 12-bit → ~1.65 V
dac1.enable();
```

### embedded-hal DAC Trait

```rust
use embedded_hal::dac::SetDac;

fn output_voltage<D: SetDac>(dac: &mut D, value: D::Word) -> Result<(), D::Error> {
    dac.set_dac(value)
}
```

---

## Bare-Metal Implementation

STM32 DAC1 enable (conceptual):

```rust
const DAC_BASE: u32 = 0x4000_7400;
const RCC_APB1ENR: u32 = 0x4002_1000 + 0x40;

unsafe fn dac1_init() {
    // Enable DAC clock in RCC
    let rcc = RCC_APB1ENR as *mut u32;
    write_volatile(rcc, read_volatile(rcc) | (1 << 29));

    // Enable DAC channel 1
    let cr = (DAC_BASE) as *mut u32;
    write_volatile(cr, read_volatile(cr) | 0x1);

    // Write 12-bit value to DHR12R1
    let dhr = (DAC_BASE + 0x08) as *mut u32;
    write_volatile(dhr, 2048);
}

unsafe fn write_volatile(addr: *mut u32, val: u32) {
    core::ptr::write_volatile(addr, val);
}
unsafe fn read_volatile(addr: *const u32) -> u32 {
    core::ptr::read_volatile(addr)
}
```

---

## Step-by-Step Explanation

### Step 1: Choose Output Method

- True DAC available? Use it.
- Otherwise PWM + filter or external SPI/I2C DAC.

### Step 2: Design Filter (PWM-DAC)

Calculate RC cutoff based on max signal frequency.

### Step 3: Create Waveform Table

Store in flash; one period of sine/triangle/sawtooth.

### Step 4: Configure Timer for Sample Rate

```
f_sample = 256 × f_wave   (256 points per cycle)
For 10 Hz sine: f_sample = 2560 Hz
```

### Step 5: Update Output in Timer ISR or DMA

Advance index, write next sample.

### Step 6: Verify with Oscilloscope

Measure THD, amplitude, frequency on output.

### Step 7: Buffer Output (If Driving Load)

Use op-amp follower for high-impedance loads.

---

## Common Mistakes

| Mistake | Symptom | Fix |
|---------|---------|-----|
| PWM frequency too low | Visible ripple on output | Increase f_pwm; adjust filter |
| Filter cutoff too high | Sawtooth instead of sine | Lower fc or raise sample rate |
| No output buffer | Distorted waveform under load | Op-amp buffer |
| ISR too slow | Jitter on waveform | Use DMA |
| Assuming ESP32 has DAC | Feature not available | Use PWM or external chip |
| Rail clipping | Flat peaks | Reduce amplitude in table |

---

## Debugging Tips

1. **Scope on PWM pin before filter** — verify duty modulation.
2. **Scope after filter** — verify smoothing.
3. **DC test:** Set constant duty 50% — multimeter should read ~1.65 V.
4. **Freeze index** — output constant voltage to isolate filter issues.
5. **Compare FFT** — scope math on sine purity.

---

## Performance Tips

| Tip | Detail |
|-----|--------|
| DMA + DAC (STM32) | Zero CPU during playback |
| Maximize PWM frequency (within LEDC limits) | Smaller filter components |
| Store tables in `.rodata` | No RAM for waveform data |
| Use power-of-two table size | Fast index wrap with `& 0xFF` |
| Direct Digital Synthesis (DDS) | Phase accumulator for arbitrary frequency |

---

## Exercises

### Exercise 1: Triangle Wave

Generate triangle wave at 5 Hz; measure peak-to-peak voltage.

### Exercise 2: Frequency Control

Potentiometer ([12-adc.md](./12-adc.md)) adjusts waveform frequency.

### Exercise 3: PWM vs True DAC

If you have STM32, compare PWM+filter vs internal DAC on scope.

### Exercise 4: SPI DAC

Interface MCP4921 via SPI; output 0–3.3 V ramp.

### Exercise 5: Function Generator

Button selects sine / triangle / square; LCD or `defmt` shows selection.

---

## References

- [STM32F411 RM — DAC](https://www.st.com/resource/en/reference_manual/rm0383-stm32f411-advanced-armbased-32bit-mcus-stmicroelectronics.pdf)
- [ESP32-S3 TRM — LEDC (PWM-DAC)](https://www.espressif.com/sites/default/files/documentation/esp32-s3_technical_reference_manual_en.pdf)
- [MCP4921 Datasheet](https://ww1.microchip.com/downloads/en/DeviceDoc/22248A.pdf)
- [embedded-hal SetDac](https://docs.rs/embedded-hal/1.0.0/embedded_hal/dac/trait.SetDac.html)
- [Lesson 11 — PWM](./11-pwm.md)

---

*Previous: [12-adc.md](./12-adc.md) | Next: [00-roadmap.md](./00-roadmap.md) (Advanced Phase)*
