# HC-SR04 — Ultrasonic Distance Sensor

The **HC-SR04** measures distance by emitting **40 kHz ultrasonic pulses** and timing the **echo** return.

**Prerequisites:** [08-gpio.md](../08-gpio.md), [10-timers.md](../10-timers.md)

---

## Working Principle

1. Host triggers **10 µs** HIGH pulse on TRIG.
2. Module sends 8× 40 kHz bursts from transmitter.
3. Receiver detects reflection; **ECHO** pin goes HIGH for duration proportional to round-trip time.

```text
distance [cm] = (echo_high_us / 58.0)
distance [cm] = (echo_high_us × speed_of_sound) / 2
```

Speed of sound ≈ 343 m/s at 20 °C (adjust for temperature via BME280 if needed).

---

## Datasheet Key Points

| Parameter | Value |
|-----------|-------|
| Range | 2 cm – 400 cm |
| Accuracy | ±3 mm (ideal) |
| Trigger pulse | 10 µs minimum |
| Echo | 5 V TTL on many modules — **use divider for ESP32** |
| Measuring angle | ~15° cone |

---

## Protocol

**GPIO bit-bang** — no I²C/SPI.

| Pin | Direction | Description |
|-----|-----------|-------------|
| VCC | Power | 5 V (better range than 3.3 V) |
| TRIG | Host → sensor | Input, trigger pulse |
| ECHO | Sensor → host | Output, timed pulse |
| GND | Ground | Common |

---

## Register Map

Not applicable.

---

## ESP32-S3 Wiring

```
HC-SR04           ESP32-S3
───────           ────────
VCC  ◄── 5V
GND  ───────────► GND
TRIG ───────────► GPIO4  (output)
ECHO ──[10k]──┬──► GPIO5  (input)
              │
             [20k]
              │
             GND
```

Divider scales 5 V echo to ~3.3 V. Some 3.3 V-tolerant clones exist — verify with multimeter idle HIGH level.

---

## Rust Driver Sketch

```rust
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::{InputPin, OutputPin};

pub struct HcSr04<TRIG, ECHO, DELAY> {
    trig: TRIG,
    echo: ECHO,
    delay: DELAY,
}

impl<TRIG, ECHO, DELAY, E> HcSr04<TRIG, ECHO, DELAY>
where
    TRIG: OutputPin<Error = E>,
    ECHO: InputPin<Error = E>,
    DELAY: DelayNs,
{
    pub fn read_cm(&mut self) -> Result<u32, E> {
        self.trig.set_low()?;
        self.delay.delay_us(2);
        self.trig.set_high()?;
        self.delay.delay_us(10);
        self.trig.set_low()?;

        // Wait for ECHO rising edge with timeout
        let width_us = self.measure_echo_high_us()?;
        Ok(width_us / 58)
    }

    fn measure_echo_high_us(&mut self) -> Result<u32, E> {
        // Poll or use hardware timer capture for accuracy
        todo!("timeout 30 ms max (~5 m range)")
    }
}
```

**Crates:** `hc-sr04` (verify ESP compatibility).

Use **hardware timer capture** on ESP32-S3 for µs accuracy instead of busy loops.

---

## Bare-Metal Notes

- **Minimum interval** between readings: 60 ms (module recovery).
- **Timeout** on echo wait — no object = stuck HIGH without timeout handling.
- Disable interrupts during short poll **or** use capture peripheral.
- Temperature affects speed of sound: `v ≈ 331.3 + 0.606 × T°C` m/s.

---

## HAL / embedded-hal Notes

```rust
pub trait RangeSensor {
    type Error;
    fn read_mm(&mut self) -> Result<u32, Self::Error>;
}
```

Implement for HC-SR04 wrapper; swap with [tof-vl53l0x.md](./tof-vl53l0x.md) in application via trait.

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| Always max range | No echo, soft surface | Aim at hard flat surface |
| Erratic readings | ECHO noise | Divider; shorter wires |
| 0 cm | ECHO stuck low | Check 5 V power |
| ESP32 crash | 5 V on GPIO | **Must** use divider |
| Crosstalk | Multiple sensors | Stagger triggers in time |

---

## Example Project: Parking Assist

1. HC-SR04 front-facing.
2. Map distance to NeoPixel color gradient ([neopixel-ws2812.md](../displays/neopixel-ws2812.md)).
3. Beep faster below 30 cm ([speakers.md](./speakers.md)).

---

## Exercises

1. Implement timer-capture echo width on ESP32-S3 (document register config).
2. Temperature-correct distance using BME280 reading.
3. Compare HC-SR04 vs VL53L0X at 5 cm, 50 cm, 200 cm.

---

## References

- HC-SR04 user guide (manufacturer PDF)
- [tof-vl53l0x.md](./tof-vl53l0x.md) — optical alternative
- [10-timers.md](../10-timers.md)
- [08-gpio.md](../08-gpio.md)
