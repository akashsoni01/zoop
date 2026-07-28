# HC-SR04 — Ultrasonic Distance Sensor

The **HC-SR04** measures distance using **ultrasonic time-of-flight** — send a 40 kHz pulse burst, measure echo return time.

**Prerequisites:** [08-gpio.md](../08-gpio.md)

---

## Working Principle

Trigger pin receives ≥10 µs HIGH pulse. Module emits 8× 40 kHz ultrasonic bursts. Echo pin goes HIGH for duration proportional to round-trip time. Speed of sound ≈ 343 m/s at 20 °C.

Distance (cm) ≈ `duration_us / 58.0`

Range: ~2 cm – 400 cm (depends on target surface and angle).

---

## Datasheet Notes

| Parameter | Value |
|-----------|-------|
| VCC | 5 V (logic tolerant on echo to 3.3 V on many modules) |
| Trigger pulse | 10 µs minimum |
| Echo output | 5 V TTL (use divider for 3.3 V ESP32) |
| Measuring angle | ~15° |
| Cycle period | ≥60 ms recommended |

---

## Protocol

**GPIO bit-bang** — no registers. Two pins:

| Pin | Direction | Function |
|-----|-----------|----------|
| TRIG | Output | Start pulse |
| ECHO | Input | Pulse width = round trip |

See [08-gpio.md](../08-gpio.md) for input capture timing.

---

## Register Map

Not applicable — pure timing protocol.

---

## ESP32-S3 Wiring

```
ESP32-S3          HC-SR04
────────          ───────
GPIO4  ─────────► TRIG
GPIO5  ◄───────── ECHO (via 1k/2k divider to 3.3 V)
5V     ─────────► VCC
GND    ─────────► GND
```

Echo divider: 1 kΩ from ECHO to GPIO5, 2 kΩ from GPIO5 to GND.

---

## Swift Driver Sketch

```swift
struct HCSR04<T: DigitalPin, E: DigitalPin, D: MicroDelay> {
    var trig: T
    var echo: E
    var delay: D

    mutating func measureCm() throws -> Float {
        try trig.setOutput()
        try echo.setInputPullDown()
        try trig.setLow()
        delay.delayMicroseconds(2)
        try trig.setHigh()
        delay.delayMicroseconds(10)
        try trig.setLow()

        let timeout: UInt32 = 30_000
        var us: UInt32 = 0
        try waitEchoHigh(timeout: timeout, elapsed: &us)
        let start = us
        try waitEchoLow(timeout: timeout, elapsed: &us)
        let width = us - start
        return Float(width) / 58.0
    }

    private mutating func waitEchoHigh(timeout: UInt32, elapsed: inout UInt32) throws { }
    private mutating func waitEchoLow(timeout: UInt32, elapsed: inout UInt32) throws { }
}
```

Prefer hardware **input capture** or `esp_timer` for µs accuracy on ESP32-S3.

---

## Bare-Metal Notes

- Disable interrupts during echo measurement if ISR latency > few µs.
- Timeout mandatory — no echo returns infinite wait.
- Temperature affects speed of sound: `v = 331.3 + 0.606 × T(°C)` m/s.

---

## HAL / Protocol-Oriented Driver Notes

Driver takes `DigitalPin + MicroDelay`. Optional `TemperatureProvider` for speed-of-sound compensation. Wrap median-of-3 filter at application layer.

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| Always max range | No echo / wrong pin | Check divider; soft target |
| Erratic readings | Noise, vibration | Median filter; ≥60 ms between reads |
| 0 cm | Echo stuck high | Power-cycle; add timeout |
| Works at 5 V logic only | Echo overdrives GPIO | Use voltage divider |

---

## Example Project

**Parking sensor:** HC-SR04 + buzzer ([speakers.md](./speakers.md)). Beep rate increases as distance drops below 50 cm.

---

## References

- [HC-SR04 User Guide](https://www.micropython.org/resources/microbit/hc-sr04-us-100.pdf)
- [08-gpio.md](../08-gpio.md)
