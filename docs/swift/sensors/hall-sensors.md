# Hall Effect Sensors

Guide to **A3144** latching/digital and **DRV5032** omnipolar Hall switches for magnetic proximity detection.

**Prerequisites:** [08-gpio.md](../08-gpio.md)

---

## Working Principle

**Hall effect** sensors detect magnetic field strength. Digital types output HIGH/LOW when field crosses threshold — used for wheel speed, door position, and brushless commutation.

A3144: unipolar, active-low open-collector. DRV5032: push-pull or open-drain, configurable polarity.

---

## Datasheet Notes

| Parameter | A3144 |
|-----------|-------|
| Supply | 4.5–24 V (3.3 V modules exist) |
| Output | Open-collector, active low |
| Operate point | ~30–250 G |
| Response | µs range |

DRV5032: 1.65–5.5 V, ideal for ESP32-S3 3.3 V.

---

## Protocol

**Digital GPIO** — read pin state. Optional **GPIO interrupt** on edge for counting rotations.

No register map on discrete Hall switches. I²C Hall sensors (DRV5032 digital out only on some boards).

See [08-gpio.md](../08-gpio.md).

---

## Register Map

Not applicable for basic A3144 modules. DRV5032 has OTP configuration via magnetic sign sequence on power-up (see TI datasheet) — not runtime I²C.

---

## ESP32-S3 Wiring

```
ESP32-S3          Hall Module (A3144)
────────          ───────────────────
GPIO4  ◄───────── OUT (open-collector)
3V3    ──────────► (internal pull-up on ESP or module)
GND    ──────────► GND
5V/3V3 ──────────► VCC (module dependent)
```

Open-collector: enable **internal pull-up** on ESP32 GPIO input.

---

## Swift Driver Sketch

```swift
protocol DigitalPin {
    mutating func read() throws -> Bool
}

protocol GPIOInterrupt {
    mutating func enableFallingEdge(handler: @escaping () -> Void) throws
}

struct HallSensor<P: DigitalPin> {
    var pin: P
    var activeLow: Bool = true

    mutating func isMagnetPresent() throws -> Bool {
        let level = try pin.read()
        return activeLow ? !level : level
    }
}

struct HallPulseCounter<P: DigitalPin> {
    var pin: P
    private(set) var count: UInt32 = 0

    mutating func onEdge() { count &+= 1 }
}
```

---

## Bare-Metal Notes

- Debounce mechanical vibration near threshold — software debounce 1–5 ms.
- Magnet orientation matters for unipolar sensors.
- Keep Hall sensor away from high-current traces causing stray fields.

---

## HAL / Protocol-Oriented Driver Notes

Trivial `MagneticSwitch` protocol. For RPM: combine with timer interrupt counting edges per revolution.

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| Always triggered | Stray field / no pull-up | Add pull-up; relocate |
| Never triggers | Weak magnet / wrong pole | Flip magnet; move closer |
| Bounce | Vibration | Debounce in software |

---

## Example Project

**Bike speedometer:** Hall on wheel spoke + magnet. Count pulses per second → km/h on SSD1306.

---

## References

- [A3144 Datasheet (Allegro)](https://www.allegromicro.com/en/products/sense/magnetic-speed/a3144)
- [DRV5032 Datasheet (TI)](https://www.ti.com/lit/ds/symlink/drv5032.pdf)
- [08-gpio.md](../08-gpio.md)
