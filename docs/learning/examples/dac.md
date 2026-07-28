# Example: DAC — Analog Output

**Goal:** Output analog voltage (where supported) or use PWM as pseudo-DAC.

**Prerequisites:** [adc.md](./adc.md), [13-dac.md](../13-dac.md) if present

**Note:** ESP32-S3 has **no native DAC**. Use **LEDC PWM + RC filter** or external I²C DAC (MCP4725).

---

## Wiring (MCP4725 I²C DAC)

```
GPIO8 SDA ──┬── MCP4725
GPIO9 SCL ──┤
3V3, GND ───┘
            └── OUT → scope / Vref
```

---

## Rust Sketch

```rust
use embedded_hal::i2c::I2c;

const MCP4725_ADDR: u8 = 0x60;

fn set_dac<I: I2c>(i2c: &mut I, value: u12) -> Result<(), I::Error> {
    let buf = [(value >> 4) as u8, ((value & 0xF) << 4) as u8];
    i2c.write(MCP4725_ADDR, &buf)
}

// Ownership: i2c bus exclusive — share with Mutex if display also on I²C
```

---

## Compile Notes

For pseudo-DAC on ESP32-S3: 20 kHz PWM + 10 kΩ/100 nF low-pass → ~0–3.3 V smooth DC.

*Next: [dma.md](./dma.md)*
