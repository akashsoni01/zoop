# Infrared (IR) Sensors

Guide to **reflective IR** (TCRT5000), **break-beam pairs**, and **IR receiver modules** (38 kHz remote) for Embedded Swift.

**Prerequisites:** [08-gpio.md](../08-gpio.md)

---

## Working Principle

### Reflective (TCRT5000)

IR LED emits light; phototransistor measures reflected intensity. Distance to reflective surface affects analog output — used for line following and proximity.

### Break-beam

IR LED and receiver face each other. Object blocking beam drops receiver signal — used for counting and safety interlocks.

### 38 kHz Remote Receiver

Module (VS1838B) demodulates modulated IR — outputs digital pulses for NEC/RC5 protocols.

---

## Datasheet Notes

| Parameter | TCRT5000 |
|-----------|----------|
| Peak wavelength | 950 nm |
| Detector | Phototransistor |
| Output | Analog + digital (on module) |
| Range | ~1–15 mm (reflective) |

VS1838B: 38 kHz carrier, 3.3–5 V, active-low output when signal present.

---

## Protocol

**Analog:** ADC read on AO pin.

**Digital:** GPIO read on DO (comparator output).

**Remote:** Pulse-width protocol on DATA pin — NEC: 9 ms low + 4.5 ms high leader, then 32 bits.

No register map.

---

## Register Map

Not applicable. IR remote protocols are timing-based, not register-oriented.

---

## ESP32-S3 Wiring

### TCRT5000 Module

```
ESP32-S3          TCRT5000
────────          ────────
GPIO1 (ADC) ◄─── AO
GPIO2         ◄─── DO
3V3           ───► VCC
GND           ───► GND
```

### VS1838B Receiver

```
ESP32-S3          VS1838B
────────          ───────
GPIO3         ◄─── OUT
3V3           ───► VCC
GND           ───► GND
```

Use [08-gpio.md](../08-gpio.md) for interrupt-on-falling-edge on remote input.

---

## Swift Driver Sketch

```swift
struct TCRT5000<A: ADCChannel, D: DigitalPin> {
    var adc: A
    var digital: D

    mutating func readAnalog() throws -> Float {
        let raw = try adc.readRaw()
        return Float(raw) / Float(adc.maxValue)
    }

    mutating func isBlocked(threshold: Float = 0.5) throws -> Bool {
        try readAnalog() < threshold
    }
}

struct NECDecoder<P: DigitalPin> {
    var pin: P
    var lastEdgeUs: UInt32 = 0

    mutating func poll() throws -> UInt32? {
        // Measure pulse widths; decode 32-bit NEC frame
        // Leader: ~9000 µs low, ~4500 µs high
        return nil
    }
}
```

---

## Bare-Metal Notes

- Ambient sunlight saturates reflective sensors — shield or use pulsed IR with synchronous detection.
- Remote decoding needs µs timing — use GPIO interrupt + hardware timer.
- TCRT5000 LED current: limit with 100 Ω series resistor if bare sensor.

---

## HAL / Protocol-Oriented Driver Notes

Split analog proximity (`ReflectiveIR`) from protocol decoder (`NECDecoder`, `RC5Decoder`). Decoders are pure timing logic testable with recorded pulse arrays.

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| Always low analog | Too far / dark surface | Adjust distance; raise LED current |
| Remote no decode | Wrong carrier freq | Confirm 38 kHz module |
| False triggers | Sunlight | Hood sensor; use break-beam |

---

## Example Project

**Line follower:** Two TCRT5000 sensors on GPIO1/GPIO2 ADC. PWM motor control based on differential reading.

---

## References

- [TCRT5000 Datasheet (Vishay)](https://www.vishay.com/docs/83366/tcrt5000.pdf)
- [08-gpio.md](../08-gpio.md)
