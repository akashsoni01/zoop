# NeoPixel WS2812 — Addressable RGB LEDs

Guide to **WS2812** / **NeoPixel** addressable RGB LED chains using precise one-wire timing on ESP32-S3.

**Prerequisites:** [08-gpio.md](../08-gpio.md)

---

## Working Principle

Each LED contains **controller + RGB LED**. Data shifts through chain: first LED latches first 24 bits (GRB order), forwards rest. Single GPIO bit-bangs or uses **RMT/I²S** peripheral for timing.

Bit encoding: **T0H** ~0.4 µs + **T0L** ~0.85 µs = 0; **T1H** ~0.8 µs + **T1L** ~0.45 µs = 1 (WS2812B typical).

---

## Datasheet Notes

| Parameter | WS2812B |
|-----------|---------|
| Data rate | 800 kHz |
| Colors | 24-bit GRB |
| Supply | 5 V (3.3 V data often works on short chains) |
| Current | ~60 mA max per LED white full |

---

## Protocol

**One-wire async serial** — no clock line, no registers. Frame: `[G8][R8][B8] × N` + **≥50 µs LOW reset**.

See [08-gpio.md](../08-gpio.md) for GPIO timing strategies.

---

## Register Map

Not applicable — LEDs are data-driven, not register-addressed.

---

## ESP32-S3 Wiring

```
ESP32-S3          NeoPixel Strip
────────          ──────────────
GPIO18 ─────────► DIN (level shift if 5 V strip)
5V     ─────────► VCC (adequate PSU — not USB only for many LEDs)
GND    ─────────► GND (common with ESP32)
```

Use 330 Ω series resistor on DIN; 1000 µF cap across strip power.

---

## Swift Driver Sketch

```swift
struct RGB8: Sendable {
    var g: UInt8, r: UInt8, b: UInt8
}

protocol NeoPixelTiming {
    mutating func writeBit(_ bit: Bool) throws
    mutating func resetPulse() throws
}

struct WS2812Chain<T: NeoPixelTiming> {
    var timing: T
    var count: Int
    var pixels: [RGB8]

    mutating func show() throws {
        for px in pixels {
            for bit in (0..<8).reversed() { try timing.writeBit((px.g >> bit) & 1 == 1) }
            for bit in (0..<8).reversed() { try timing.writeBit((px.r >> bit) & 1 == 1) }
            for bit in (0..<8).reversed() { try timing.writeBit((px.b >> bit) & 1 == 1) }
        }
        try timing.resetPulse()
    }

    mutating func set(index: Int, color: RGB8) {
        guard pixels.indices.contains(index) else { return }
        pixels[index] = color
    }
}

// ESP32-S3: prefer RMT peripheral backend implementing NeoPixelTiming
```

---

## Bare-Metal Notes

- Disable interrupts corrupts bit timing on bit-bang — use RMT or I²S on ESP32-S3.
- First LED data line level: 3.3 V usually sufficient for WS2812B at short distance; level shifter for reliability.
- Estimate power: 24 LEDs × 60 mA = 1.4 A at full white.

---

## HAL / Protocol-Oriented Driver Notes

`AddressableLEDStrip` protocol: `set(index:color:)`, `show()`. Backends: `BitBangWS2812`, `RMTWS2812`.

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| Wrong colors | RGB vs GRB order | Send GRB |
| Flicker white | Insufficient reset | ≥50 µs LOW after frame |
| First LED wrong | Level shift | Add 74HCT125 shifter |
| Dim at end | Voltage drop | Inject power mid-strip |

---

## Example Project

**Lux bar graph:** Map BH1750 reading ([sensors/light-bh1750.md](../sensors/light-bh1750.md)) to 24-LED gradient with gamma correction.

---

## References

- [WS2812B Datasheet](https://cdn-shop.adafruit.com/datasheets/WS2812B.pdf)
- [08-gpio.md](../08-gpio.md)
