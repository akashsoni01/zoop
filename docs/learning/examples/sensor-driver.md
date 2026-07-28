# Example: Sensor Driver Pattern

**Goal:** Write a reusable `embedded-hal` driver crate for an I²C sensor.

**Prerequisites:** [i2c-scanner.md](./i2c-scanner.md), [07-hal.md](../07-hal.md)

**Reference sensor:** [sensors/bme280.md](../sensors/bme280.md)

---

## Rust Sketch (driver crate)

```rust
// bme280.rs — in your driver crate
use embedded_hal::i2c::I2c;

pub struct Bme280<I2C> {
    i2c: I2C,
    addr: u8,
}

pub enum Error<E> {
    Bus(E),
    InvalidChipId,
}

impl<I2C, E> Bme280<I2C>
where
    I2C: I2c<Error = E>,
{
    pub fn new(i2c: I2C, addr: u8) -> Self {
        Self { i2c, addr }
    }

    pub fn init(&mut self) -> Result<(), Error<E>> {
        let id = self.read_reg(0xD0)?;
        if id != 0x60 { return Err(Error::InvalidChipId); }
        // load calibration...
        Ok(())
    }

    fn read_reg(&mut self, reg: u8) -> Result<u8, Error<E>> {
        let mut buf = [0u8];
        self.i2c.write_read(self.addr, &[reg], &mut buf).map_err(Error::Bus)?;
        Ok(buf[0])
    }
}

// Ownership: Bme280 owns I2C bus — compose with embedded-hal-bus for sharing
```

### Ownership

Generic `I2C` — driver **owns** bus by value; parent passes `I2cDevice` from mutex when sharing with OLED.

---

## Compile Notes

Publish as workspace crate; firmware depends on path. Test with `linux-embedded-hal` on Raspberry Pi (optional).

*Next: [oled.md](./oled.md)*
