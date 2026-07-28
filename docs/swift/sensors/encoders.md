# Rotary Encoders — Quadrature

Guide to **incremental rotary encoders** (quadrature A/B) for Embedded Swift.

**Prerequisites:** [08-gpio.md](../08-gpio.md)

---

## Working Principle

Encoder disk generates **two square waves (A, B) 90° out of phase**. Direction determined by which channel leads. Count edges for position; optional push button on third pin.

Full quadrature: 4× counts per detent on 20 PPR encoder (depends on decoding mode).

---

## Datasheet Notes

| Parameter | Typical EC11 |
|-----------|--------------|
| Pulses/rev | 20 (20 PPR) |
| Detents | 20 |
| Outputs | Open-collector or push-pull |
| Switch | Optional NC push |

---

## Protocol

**GPIO quadrature decode** — poll or interrupt on A/B edges.

State machine (2-bit): previous AB → current AB → increment/decrement.

See [08-gpio.md](../08-gpio.md) for interrupt configuration.

---

## Register Map

Not applicable.

---

## ESP32-S3 Wiring

```
ESP32-S3          Rotary Encoder
────────          ──────────────
GPIO4  ◄───────── CLK (A)
GPIO5  ◄───────── DT  (B)
GPIO6  ◄───────── SW  (button, optional)
3V3    ──────────► +
GND    ──────────► GND
```

Enable internal pull-ups on A, B, SW.

---

## Swift Driver Sketch

```swift
struct QuadratureEncoder<A: DigitalPin, B: DigitalPin> {
    var pinA: A
    var pinB: B
    private(set) var count: Int32 = 0
    private var lastState: UInt8 = 0

    private let table: [Int8] = [
        0, 1, -1, 0, -1, 0, 0, 1, 1, 0, 0, -1, 0, -1, 1, 0
    ]

    mutating func poll() throws {
        let a = try pinA.read() ? 1 : 0
        let b = try pinB.read() ? 1 : 0
        let state = (a << 1) | b
        let idx = Int(lastState << 2) | state
        count += Int32(table[idx])
        lastState = state
    }
}
```

Use GPIO interrupt on A rising/falling for lower latency.

---

## Bare-Metal Notes

- Mechanical bounce: 1–5 ms — ignore transitions faster than debounce window.
- ISR must be short — set flag, decode in main loop if needed.
- Missed counts at high RPM — use interrupt-driven decode.

---

## HAL / Protocol-Oriented Driver Notes

`RotaryEncoder` protocol: `position() -> Int32`, `reset()`. Optional `Button` for SW pin.

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| Count skips | Polling too slow | Use interrupts |
| Wrong direction | A/B swapped | Swap pins or negate |
| Jitter | Bounce | Hardware RC filter or debounce |

---

## Example Project

**Menu UI:** Encoder adjusts ST7789 menu selection; press SW to confirm. Map count to list index.

---

## References

- [08-gpio.md](../08-gpio.md)
