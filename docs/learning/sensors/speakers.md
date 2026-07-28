# Speakers & Buzzers (I²S DAC, PWM)

Audio **output** on embedded systems — **I²S amplifiers** (MAX98357A), **PWM buzzers**, and **passive piezo** tones.

**Prerequisites:** [11-pwm.md](../11-pwm.md), [10-timers.md](../10-timers.md)

---

## Working Principle

| Device | Mechanism | Interface |
|--------|-----------|-----------|
| Active buzzer | Built-in oscillator | GPIO HIGH/LOW |
| Passive piezo | Resonate at drive frequency | PWM square wave |
| I²S amp | Digital PCM → analog | I²S + amplifier |

**PWM tone generation:** toggle frequency `f` on GPIO for musical note; duty cycle ~50% for piezo.

---

## Datasheet Key Points (MAX98357A)

| Parameter | Value |
|-----------|-------|
| Input | I²S, 16/24/32-bit |
| Output power | 3.2 W @ 4 Ω |
| Gain | 3/6/9/12/15 dB (GAIN pin strapping) |
| Supply | 2.5–5.5 V |

---

## Protocol

### I²S to MAX98357A

```
MAX98357A        ESP32-S3
─────────        ────────
VIN  ◄── 5V (for volume)
GND  ──────────► GND
DIN  ◄────────── GPIO14 (I2S DOUT)
BCLK ◄────────── GPIO12
LRC  ◄────────── GPIO13
GAIN ── strap ──► 9 dB typical
```

### Passive Buzzer (PWM)

```
Piezo + ──► GPIO18 (PWM via LEDC)
Piezo − ──► GND
```

Use series resistor 100 Ω if driving directly from GPIO.

---

## Register Map

Not applicable for buzzer or MAX98357A (hardware-configured).

---

## Rust Driver Sketch

### PWM Tone (esp-hal LEDC)

```rust
pub struct Buzzer {
    frequency_hz: u32,
}

impl Buzzer {
    pub fn play_tone(&mut self, freq: u32, duration_ms: u32) {
        self.set_pwm_freq(freq);
        self.delay_ms(duration_ms);
        self.stop();
    }

    pub fn play_melody(&mut self, notes: &[(u32, u32)]) {
        for &(freq, ms) in notes {
            if freq > 0 {
                self.play_tone(freq, ms);
            } else {
                self.delay_ms(ms);
            }
        }
    }
}
```

Note frequencies: `f = 440 × 2^(n/12)` for semitone `n` from A4.

### I²S Playback

Stream PCM buffer via DMA to MAX98357A — same peripheral as [microphones.md](./microphones.md) but TX direction.

**Crates:** `embedded-audio` ecosystem (experimental), chip HAL I2S examples.

---

## Bare-Metal Notes

- **Active buzzer:** never PWM — DC on/off only.
- **Piezo:** frequency range ~200 Hz–10 kHz; poor bass.
- I²S amp needs **continuous clock** when playing — stop cleanly to avoid pop.
- 5 V on MAX98357A for louder output — level-shift I²S lines if amp is 5 V (often 3.3 V compatible).

---

## HAL / embedded-hal Notes

```rust
use embedded_hal::pwm::SetDutyCycle;

pub trait ToneGenerator {
    type Error;
    fn tone(&mut self, hz: u32) -> Result<(), Self::Error>;
    fn silence(&mut self) -> Result<(), Self::Error>;
}
```

Implement with `SetDutyCycle` + frequency change via timer prescaler.

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| Weak sound | Wrong buzzer type | Passive needs PWM frequency |
| Constant whine | GPIO left driving | silence() after tone |
| I²S noise | Buffer underrun | Increase DMA buffer |
| Distortion | Gain too high | Lower GAIN strap or digital volume |

---

## Example Project: Alarm Chime

1. BME280 high temp → play 3-note alert on piezo.
2. Visual sync on NeoPixel flash.
3. Snooze via touch pad ([touch-sensors.md](./touch-sensors.md)).

---

## Exercises

1. Play "Happy Birthday" note table on passive buzzer.
2. Stream 1 kHz sine lookup table over I²S (8 kHz sample rate).
3. Measure current draw active vs passive buzzer at same perceived loudness.

---

## References

- Maxim MAX98357A datasheet
- [11-pwm.md](../11-pwm.md), [microphones.md](./microphones.md)
- [10-timers.md](../10-timers.md)
