# Microphones — I²S, PDM, Analog

Guide to **INMP441 I²S**, **PDM MEMS**, and **analog electret** microphones for Embedded Swift on ESP32-S3.

**Prerequisites:** [08-gpio.md](../08-gpio.md)

---

## Working Principle

**MEMS microphone** converts sound pressure to digital (PDM bitstream or I²S PCM) or analog voltage. ESP32-S3 **I²S peripheral** captures PCM samples for processing (RMS level, simple FFT, voice detection).

INMP441: 24-bit I²S, left-justified, typical 16 kHz–48 kHz sample rate.

---

## Datasheet Notes

| Parameter | INMP441 |
|-----------|---------|
| Interface | I²S ( Philips format) |
| SNR | 61 dBA |
| Sample rate | 16–48 kHz |
| Supply | 1.8–3.3 V |
| L/R pin | Select left/right channel |

---

## Protocol

**I²S:** BCLK (bit clock), LRCK/WS (word select), DIN (data from mic). Standard Philips I²S — 32-bit frame, 24-bit data.

**Analog:** AC-coupled signal into ESP32 **ADC** — lower quality, simpler wiring.

No register map on INMP441 (hardwired mode).

---

## Register Map

INMP441: none (hardware strapping). ESP32-S3 I²S configured via SoC registers:

| Config | Typical |
|--------|---------|
| Mode | Master RX |
| Sample width | 32 bit container, 24 valid |
| Sample rate | 16000 Hz |
| Channels | Mono (L/R tied low) |

---

## ESP32-S3 Wiring (INMP441)

```
ESP32-S3          INMP441
────────          ───────
GPIO4  ◄───────── SD (DOUT)
GPIO5  ─────────► SCK (BCLK)
GPIO6  ─────────► WS  (LRCK)
3V3    ─────────► VDD
GND    ─────────► GND
GND    ─────────► L/R (left channel)
```

---

## Swift Driver Sketch

```swift
protocol I2SRX {
    mutating func configure(sampleRate: Int, bits: Int) throws
    mutating func readSamples(into buffer: inout [Int32], count: Int) throws
}

struct MicrophoneLevel<I: I2SRX> {
    var i2s: I

    mutating func readRMS(window: Int = 256) throws -> Float {
        var buf = [Int32](repeating: 0, count: window)
        try i2s.readSamples(into: &buf, count: window)
        var sum: Float = 0
        for s in buf {
            let f = Float(s) / Float(1 << 23)
            sum += f * f
        }
        return sqrt(sum / Float(window))
    }
}
```

---

## Bare-Metal Notes

- DMA ring buffer recommended for continuous capture.
- Keep mic away from switching power noise.
- DC blocking capacitor on analog mic path.

---

## HAL / Protocol-Oriented Driver Notes

`AudioInput` protocol: `sampleRate`, `read(buffer:)`. Backends: `I2SMic`, `ADCMic`, `PDMMic`.

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| Silence | L/R wrong | Tie L/R to GND or 3V3 |
| Clipping | Gain too high | Reduce digital gain |
| Noise hum | Ground loop | Star ground; shield cable |

---

## Example Project

**Clap detector:** INMP441 at 16 kHz; trigger LED when RMS exceeds threshold for 50 ms.

---

## References

- [INMP441 Datasheet (InvenSense)](https://invensense.tdk.com/wp-content/uploads/2016/02/INMP441-DS-Rev1.0.pdf)
- [08-gpio.md](../08-gpio.md)
