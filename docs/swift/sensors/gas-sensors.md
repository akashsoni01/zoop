# Gas Sensors — MQ Series

Guide to **MQ-2**, **MQ-135**, and similar **semiconductor gas sensors** using analog output and optional digital threshold pins.

**Prerequisites:** [08-gpio.md](../08-gpio.md)

---

## Working Principle

**SnO₂ (tin dioxide)** heated surface adsorbs gas molecules, changing resistance. A heater element (5 V) maintains operating temperature (~300 °C). Output is **analog voltage** proportional to gas concentration after warmup.

MQ-2: LPG, propane, methane, smoke. MQ-135: NH₃, NOx, alcohol, CO₂ (indoor air quality proxy).

---

## Datasheet Notes

| Parameter | MQ-2 typical |
|-----------|--------------|
| Heater voltage | 5 V |
| Load resistor RL | 10 kΩ on module |
| Preheat time | 24–48 h first use; 2–5 min per session |
| Detectable range | 200–10000 ppm (LPG) |

Module boards include comparator (LM393) for **digital D0** output with potentiometer threshold.

---

## Protocol

**Analog:** Voltage 0–3.3 V on AO pin → ESP32-S3 **ADC** channel.

**Digital:** D0 is active-low when gas exceeds threshold. Read via GPIO per [08-gpio.md](../08-gpio.md).

No I²C register map on basic modules.

---

## Register Map

Not applicable for analog MQ modules. I²C gas sensors (SGP30, BME680 IAQ) use chip-specific registers — out of scope for basic MQ boards.

---

## ESP32-S3 Wiring

```
ESP32-S3          MQ-2 Module
────────          ───────────
5V (VIN)  ──────► VCC (heater — needs 5 V)
3V3       ──────► (if module has 3.3 V logic only)
GPIO1 (ADC) ◄─── AO
GPIO2       ◄─── D0 (optional digital)
GND       ──────► GND
```

Use voltage divider if AO swings above 3.3 V.

---

## Swift Driver Sketch

```swift
protocol ADCChannel {
    mutating func readRaw() throws -> UInt16
    var maxValue: UInt16 { get }
    var referenceVoltage: Float { get }
}

protocol DigitalPin {
    mutating func read() throws -> Bool
}

struct MQ2Sensor<A: ADCChannel> {
    var adc: A
    var rlKOhm: Float = 10.0
    var r0: Float = 10.0  // baseline resistance in clean air (calibrate)

    mutating func readVoltage() throws -> Float {
        let raw = try adc.readRaw()
        return Float(raw) / Float(adc.maxValue) * adc.referenceVoltage
    }

    mutating func readRatio() throws -> Float {
        let v = try readVoltage()
        let rs = ((3.3 - v) / v) * rlKOhm
        return rs / r0
    }

    mutating func calibrateR0(samples: Int = 100) throws {
        var sum: Float = 0
        for _ in 0..<samples {
            let v = try readVoltage()
            let rs = ((3.3 - v) / v) * rlKOhm
            sum += rs
        }
        r0 = sum / Float(samples)
    }
}
```

---

## Bare-Metal Notes

- Heater draws ~150 mA — do not power from 3.3 V pin alone; use **5 V VIN**.
- Sensor drift requires periodic recalibration in known clean air.
- Apply moving average filter — raw signal is noisy.

---

## HAL / Protocol-Oriented Driver Notes

Abstract ADC via `ADCChannel` protocol. Keep ppm conversion curves (log-log lookup) in separate `MQ2Curve` struct from hardware layer.

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| Always max reading | Short warmup / saturated | Wait 5 min; ventilate area |
| Always zero | Heater off | Check 5 V supply |
| ADC pegged | Voltage > 3.3 V | Add divider |

---

## Example Project

**Air quality alarm:** MQ-135 with buzzer ([speakers.md](./speakers.md)). Calibrate R0 outdoors. Trigger alert when ratio exceeds 2× baseline for 30 s.

---

## References

- [MQ-2 Datasheet (Hanwei)](https://www.sparkfun.com/datasheets/Sensors/Biometric/MQ-2.pdf)
- [08-gpio.md](../08-gpio.md) — ADC and digital input
