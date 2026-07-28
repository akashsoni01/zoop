# Speakers & Buzzers — I²S DAC, PWM

Guide to **MAX98357A I²S amplifier**, **passive buzzers**, and **piezo** output for Embedded Swift on ESP32-S3.

**Prerequisites:** [08-gpio.md](../08-gpio.md)

---

## Working Principle

**I²S DAC/amp** (MAX98357A): digital PCM stream converted to analog drive for speaker. **Passive buzzer**: square wave at resonant frequency produces tone. **Piezo**: similar, driven by PWM or GPIO toggle.

---

## Datasheet Notes

| Parameter | MAX98357A |
|-----------|-----------|
| Input | I²S, 16/24/32 bit |
| Output power | 3.2 W @ 4 Ω |
| Sample rates | 8–96 kHz |
| Gain | 3/6/9/12/15 dB (GAIN pin) |

Passive buzzer: specify resonant frequency (e.g. 4 kHz).

---

## Protocol

**I²S TX:** ESP32-S3 outputs BCLK, LRCK, DOUT to MAX98357A DIN.

**PWM buzzer:** LEDC PWM on GPIO at target frequency per [08-gpio.md](../08-gpio.md).

No register map on MAX98357A (pin-strapped gain).

---

## Register Map

MAX98357A: none. ESP32-S3 LEDC/PWM timer registers configured via HAL.

---

## ESP32-S3 Wiring (MAX98357A)

```
ESP32-S3          MAX98357A
────────          ─────────
GPIO4  ─────────► DIN
GPIO5  ─────────► BCLK
GPIO6  ─────────► LRC
3V3    ─────────► VIN
GND    ─────────► GND
Speaker ◄────────► OUT+/OUT-
```

Passive buzzer: GPIO7 → buzzer + → GND.

---

## Swift Driver Sketch

```swift
protocol I2STX {
    mutating func configure(sampleRate: Int) throws
    mutating func writeSamples(_ buffer: [Int16]) throws
}

struct ToneGenerator<P: PWMChannel> {
    var pwm: P

    mutating func play(frequencyHz: Int, durationMs: Int) throws {
        try pwm.setFrequency(frequencyHz)
        try pwm.setDutyPercent(50)
        delayMs(durationMs)
        try pwm.setDutyPercent(0)
    }
}

struct WAVPlayer<I: I2STX> {
    var i2s: I

    mutating func playPCM(_ samples: [Int16], sampleRate: Int) throws {
        try i2s.configure(sampleRate: sampleRate)
        try i2s.writeSamples(samples)
    }
}
```

---

## Bare-Metal Notes

- Speaker draws significant current — separate decoupling cap 100 µF near amp.
- I²S DMA for gapless playback.
- Passive buzzer: no DC across element — use AC square wave only.

---

## HAL / Protocol-Oriented Driver Notes

`AudioOutput` protocol with `play(tone:)` and `play(pcm:)`. Buzzer backend uses `PWMChannel`.

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| Silent I²S | LRCK/BCLK swap | Verify wiring |
| Distortion | Undervoltage | Separate 5 V for amp if needed |
| Weak buzzer | Wrong frequency | Match resonant freq |

---

## Example Project

**Alarm clock:** RTC ([rtc.md](./rtc.md)) triggers buzzer melody via PWM at set time.

---

## References

- [MAX98357A Datasheet (Maxim)](https://datasheets.maximintegrated.com/en/ds/MAX98357A-MAX98357B.pdf)
- [08-gpio.md](../08-gpio.md)
