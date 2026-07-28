# Example: ADC — Analog Input

**Goal:** Read potentiometer voltage with 12-bit ADC.

**Prerequisites:** [07-hal.md](../07-hal.md), [12-adc.md](../12-adc.md) if present

---

## Wiring

```
3V3 ──── potentiometer ──── GND
              │
           GPIO6 (ADC1_CH5 — verify pin map for ESP32-S3)
```

---

## Rust Sketch

```rust
use esp_hal::analog::ADC;
use embedded_hal::adc::OneShot;

fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let mut adc = ADC::new(peripherals.ADC1);
    let mut pin = peripherals.GPIO6;

    loop {
        let raw: u16 = adc.read_oneshot(&mut pin).unwrap();
        // ESP32-S3 ADC: 12-bit, attenuation affects range
        defmt::info!("ADC: {}", raw);
    }
}
```

### Ownership

`adc` borrows `pin` mutably for each conversion — cannot read two channels simultaneously without mux.

---

## Compile Notes

Enable ADC calibration in IDF/esp-hal for absolute voltage. See [sensors/temperature-humidity.md](../sensors/temperature-humidity.md) for I²C alternative.

*Next: [dac.md](./dac.md)*
