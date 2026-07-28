# Microphones (I²S, PDM, Analog)

Capturing **audio** on embedded systems — **I²S digital MEMS** (INMP441), **PDM** microphones, and **analog electret** capsules with ADC.

**Prerequisites:** [12-adc.md](../12-adc.md), I²S lesson (planned), [09-interrupts.md](../09-interrupts.md)

---

## Working Principle

| Type | Mechanism | Interface |
|------|-----------|-----------|
| Electret analog | FET + diaphragm → voltage | ADC + bias circuit |
| MEMS I²S | MEMS + ADC on chip | I²S digital |
| PDM | 1-bit density modulation | PDM clock + data |

**I²S (Inter-IC Sound):** serial bus with **BCLK** (bit clock), **LRCK/WS** (word select), **DIN/DOUT** (data). Standard for CD-quality streams.

**PDM:** high-rate 1-bit stream; decimate in software or hardware filter.

---

## Datasheet Key Points (INMP441)

| Parameter | Value |
|-----------|-------|
| Interface | I²S, 24-bit |
| SNR | 61 dBA |
| Sample rates | 8–48 kHz typical |
| Supply | 1.8–3.3 V |

---

## Protocol

### I²S Wiring (INMP441 → ESP32-S3)

```
INMP441          ESP32-S3
───────          ────────
VDD  ──────────► 3V3
GND  ──────────► GND
SCK  ◄────────── GPIO12 (I2S BCLK)
WS   ◄────────── GPIO13 (I2S LRCK)
SD   ──────────► GPIO14 (I2S DIN)
L/R  ── GND ───► (left channel)
```

See [16-spi.md](../16-spi.md) for general serial timing concepts; I²S is separate peripheral.

### Analog Electret

```
Mic + ──► coupling cap ──► ESP32 ADC GPIO1
Mic − ──► GND
Bias resistor per electret module schematic
```

---

## Register Map

Digital MEMS mics are **hardwired mode** (no I²C config on INMP441). Some mics (ICS-43434) have optional mode pins only.

---

## Rust Driver Sketch (I²S Capture)

```rust
// esp-hal I2S RX example pattern
pub struct AudioCapture {
    buffer: [i32; 1024],
    sample_rate: u32,
}

impl AudioCapture {
    pub fn read_samples(&mut self) -> &[i32] {
        // DMA fills buffer; return slice for processing
        &self.buffer
    }

    pub fn rms_level(&self) -> f32 {
        let sum: i64 = self.buffer.iter().map(|&s| (s as i64).pow(2)).sum();
        ((sum as f32) / self.buffer.len() as f32).sqrt()
    }
}
```

**Crates:** `esp-hal` I2S, `dasp` (DSP traits), `micromath`.

---

## Bare-Metal Notes

- **DMA required** for high sample rates — CPU bit-banging insufficient.
- Buffer **double-buffering** (ping-pong) avoids gaps.
- Electret on SAR ADC: **high noise** — use I²S MEMS for quality.
- PDM: ESP32-S3 has PDM RX mode — decimation filter in hardware.

---

## HAL / embedded-hal Notes

No universal embedded-hal I²S trait yet — use chip HAL (`esp-hal::i2s`, `stm32xx-hal::i2s`).

Abstract at application level:

```rust
pub trait AudioInput {
    fn fill_buffer(&mut self, dest: &mut [i16]) -> Result<(), AudioError>;
}
```

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| Silence | L/R pin wrong | Tie L/R for channel |
| Clipping | Gain too high | Reduce digital gain |
| Whine | Wi-Fi + analog | Use digital I²S mic |
| Wrong rate | MCLK/BCLK ratio | Match driver to 48 kHz / 16 kHz |

---

## Example Project: Clap Detector

1. INMP441 @ 16 kHz mono.
2. Compute RMS over 32 ms windows.
3. Threshold crossing → toggle NeoPixel ([neopixel-ws2812.md](../displays/neopixel-ws2812.md)).

---

## Exercises

1. Visualize waveform on SSD1306 scroll plot.
2. Implement simple band-pass for voice band (300–3400 Hz) — conceptual FIR.
3. Compare RMS noise floor: analog vs I²S mic.

---

## References

- TDK InvenSense INMP441 datasheet
- ESP32-S3 I2S documentation (Espressif TRM)
- [speakers.md](./speakers.md) — output path
- [12-adc.md](../12-adc.md)
