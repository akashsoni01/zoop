# Gas Sensors (MQ Series)

**MQ-series** gas sensors (MQ-2, MQ-135, etc.) use a **heated metal oxide (MOX)** element whose resistance changes with target gas concentration. Output is **analog** (and often digital via onboard comparator).

**Prerequisites:** [12-adc.md](../12-adc.md), [08-gpio.md](../08-gpio.md)

---

## Working Principle

1. **Heater** brings sensing element to operating temperature (150–300 °C internally).
2. **MOX conductivity** changes when reducible gases (LPG, smoke, CO) contact the surface.
3. **Load resistor** forms voltage divider → ADC voltage ∝ gas concentration (non-linear, uncalibrated).

| Model | Detects (indicative) |
|-------|---------------------|
| MQ-2 | LPG, propane, smoke |
| MQ-135 | Air quality, NH₃, benzene, CO₂ proxy |
| MQ-7 | Carbon monoxide |
| MQ-9 | CO, combustible gas |

Datasheets provide **Rs/R0 vs ppm** curves — R0 measured in **clean air** after burn-in.

---

## Datasheet Key Points

| Parameter | Typical |
|-----------|---------|
| Heater voltage | 5 V required (MQ modules) |
| Sensor voltage | 5 V divider → **needs level shift** for 3.3 V ADC |
| Preheat time | 24–48 h first use; 3 min minimum per reading |
| Load resistor RL | Often 10 kΩ on module |

---

## Protocol

**Analog:** Voltage on AOUT pin → ESP32-S3 ADC ([12-adc.md](../12-adc.md)).

**Digital:** DOUT pin high/low vs onboard pot threshold — `InputPin` only.

No I²C/SPI registers.

---

## Register Map

Not applicable (analog sensor). Treat as voltage source with ~kΩ source impedance.

---

## ESP32-S3 Wiring

```
MQ-2 Module (5V)          ESP32-S3
──────────────            ────────
VCC  ◄─── 5V (USB/VIN)    (5V rail required for heater)
GND  ──── GND ──────────── GND
AOUT ──── voltage divider ─► GPIO1 (ADC1)
DOUT ──── optional ────────► GPIO2 (digital)
```

**Level shifting AOUT:** Use resistor divider 10 kΩ / 20 kΩ to scale 0–5 V → 0–3.3 V, or use op-amp buffer.

```
AOUT ──[10k]──┬──► GPIO1 (ADC)
              │
             [20k]
              │
             GND
```

---

## Rust Driver Sketch

```rust
use embedded_hal::adc::OneShot;

pub struct Mq2<ADC, PIN> {
    adc: ADC,
    pin: PIN,
    r0: f32, // baseline Rs in clean air
}

impl<ADC, PIN, E> Mq2<ADC, PIN>
where
    ADC: OneShot<PIN, Error = E>,
{
    pub fn read_ratio(&mut self) -> Result<f32, E> {
        let raw = self.adc.read(&mut self.pin)?;
        let v = raw as f32 / 4095.0 * 3.3;
        // Rs = ((Vcc - V) / V) * RL  (module schematic dependent)
        let rs = (5.0 - v) / v * 10_000.0;
        Ok(rs / self.r0)
    }

    pub fn calibrate_clean_air(&mut self, samples: usize) -> Result<(), E> {
        // average ratio at boot in known clean air
        todo!()
    }
}
```

Convert ratio to ppm using datasheet log-log plot (piecewise linear approximation).

---

## Bare-Metal Notes

- **ADC1 only** on ESP32 when Wi-Fi active (ADC2 conflict).
- Average **64+ samples** — heater noise is significant.
- **Temperature/humidity** affect readings — cross-compensate with BME280 if needed.
- Never expose to **silicone vapors** — poisons MOX permanently.

---

## HAL / embedded-hal Notes

```rust
use embedded_hal::adc::OneShot;
use embedded_hal::delay::DelayNs;
```

Preheat gate in application:

```rust
delay.delay_ms(180_000); // 3 min before trusting readings
```

For digital DOUT: `embedded_hal::digital::InputPin`.

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| Always max ADC | Saturated gas or wrong divider | Ventilate; adjust divider |
| Slow response | Normal for MOX | Wait minutes after gas event |
| Drift over days | Aging | Recalibrate R0 weekly |
| 5 V on ESP32 pin | Wiring error | **Never** connect 5 V AOUT directly |

---

## Example Project: Air Quality Alert

1. MQ-135 + BME280 on same firmware.
2. If Rs/R0 > threshold for 30 s, trigger buzzer ([speakers.md](./speakers.md)).
3. Log peaks to flash with timestamp from RTC ([rtc.md](./rtc.md)).

---

## Exercises

1. Capture Rs/R0 over 24 h burn-in; plot stabilization.
2. Implement datasheet curve lookup table for LPG ppm estimate.
3. Compare analog vs digital DOUT trigger levels.

---

## References

- Hanwei MQ-2 / MQ-135 datasheets
- [12-adc.md](../12-adc.md)
- [bme280.md](./bme280.md) — environmental compensation
