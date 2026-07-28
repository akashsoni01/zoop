# I²C — Inter-Integrated Circuit

**I²C** (I-squared-C, **I2C**) is a **two-wire**, **multi-master**, **multi-slave** bus: **SDA** (data) and **SCL** (clock). Each device has a 7-bit (or 10-bit) **address**.

**Prerequisites:** [08-gpio.md](../08-gpio.md), [gpio.md](./gpio.md)

---

## Theory

Both lines are **open-drain** with **pull-up resistors** (typically 2.2 kΩ–10 kΩ to 3.3 V). The master generates SCL; either party can pull SDA LOW.

Transaction sequence:

1. **START** — SDA falls while SCL high
2. **Address + R/W bit** — 7 bits address, 1 bit direction
3. **ACK** — receiver pulls SDA LOW
4. **Data bytes** — MSB first, ACK per byte
5. **STOP** — SDA rises while SCL high

**Clock stretching:** a slow slave holds SCL LOW until ready — ESP32-S3 hardware handles this.

Common speeds: **100 kHz** (Standard), **400 kHz** (Fast), **1 MHz** (Fast+).

---

## Timing Diagram (ASCII)

Write transaction — address 0x48, write byte 0x3C:

```
SCL  ──┐ ┌─┐ ┌─┐ ┌─┐ ┌─┐ ┌─┐ ┌─┐ ┌─┐ ┌─┐ ┌─┐ ┌─┐ ┌─┐ ┌─┐ ┌─┐ ┌─┐
       └─┘ └─┘ └─┘ └─┘ └─┘ └─┘ └─┘ └─┘ └─┘ └─┘ └─┘ └─┘ └─┘ └─┘

SDA  ──┐ S │A6│A5│A4│A3│A2│A1│A0│W│ACK│D7│..│D0│ACK│ P
       └───────────────────────────────────────────────

S = START, P = STOP, W = Write (0), shaded = slave pulls LOW (ACK)
Address byte = 0x48 << 1 | 0 = 0x90 on wire
```

Repeated START (Sr) allows combined write-then-read (common in sensors):

```
SDA  ── S │ addr W │ data │ Sr │ addr R │ data │ P
```

---

## Packet Format

| Field | Bits | Description |
|-------|------|-------------|
| START | — | Bus claim |
| Slave address | 7 | 0x03–0x77 valid range |
| R/W | 1 | 0 = write, 1 = read |
| Data | 8 × N | Payload |
| ACK/NACK | 1 per byte | LOW = ACK |
| STOP | — | Release bus |

**Register map pattern** (most sensors):

```
START | Addr+W | RegIndex | Sr | Addr+R | Data... | STOP
```

---

## Electrical Characteristics

| Parameter | Typical |
|-----------|---------|
| VDD | 3.3 V (ESP32-S3) |
| Pull-ups | 4.7 kΩ @ 100 kHz, 2.2 kΩ @ 400 kHz |
| Max bus capacitance | 400 pF (Standard mode) |
| 5 V devices | Use level shifter (TXS0102, PCA9306) |

Long cables increase capacitance — reduce pull-up value or lower clock speed.

---

## Swift HAL Sketch

```swift
import Embedded
import ESP32Hardware

let bme280Address: UInt8 = 0x76

struct I2CSensor {
    var bus: I2C0

    mutating func readChipID() throws -> UInt8 {
        var id: UInt8 = 0
        try bus.writeRead(
            address: bme280Address,
            write: [0xD0],
            read: &id
        )
        return id
    }
}

func scanBus(bus: inout I2C0) -> [UInt8] {
    var found: [UInt8] = []
    for addr in 0x08...0x77 {
        if bus.probe(address: addr) {
            found.append(addr)
        }
    }
    return found
}
```

**Ownership:** `I2CSensor` holds `I2C0` by value or `&mut` borrow — only one task mutates the bus at a time. Use a `Mutex<I2C0>` if async tasks share the bus.

**Compile (ESP32-S3):**

```bash
swift build -c release
espflash flash --monitor .build/release/I2CScanner.bin
```

---

## Bare-Metal Sketch (Concept)

```swift
// START condition via HAL preferred; bit-bang for learning:
// 1. SDA high, SCL high, SDA low (START)
// 2. Shift 8 bits, sample SDA on SCL rising
// 3. Release SDA for ACK slot, clock once
// 4. STOP: SCL high, SDA low→high
```

See [06-register-programming.md](../06-register-programming.md) for register access patterns.

---

## Example Projects

| Link | Sensor/Device |
|------|---------------|
| [examples/i2c-scanner.md](../examples/i2c-scanner.md) | Bus discovery |
| [examples/sensor-driver.md](../examples/sensor-driver.md) | Generic driver pattern |
| [projects/environmental-monitor.md](../projects/environmental-monitor.md) | BME280 + SHT40 |
| [projects/weather-station.md](../projects/weather-station.md) | Multi-sensor node |

---

## Common Mistakes

1. **Missing pull-ups** — SDA/SCL never reach HIGH.
2. **Wrong address** — ADDR pin changes LSB (0x76 vs 0x77).
3. **Confusing 8-bit vs 7-bit address** in datasheets.
4. **No repeated START** — read fails after write pointer set.
5. **Bus lock after crash** — power-cycle or clock 9 pulses on SCL.

---

## Exercises

1. Build [examples/i2c-scanner.md](../examples/i2c-scanner.md) and log all addresses.
2. Read BME280 chip ID register `0xD0` — expect `0x60`.
3. Implement `writeRead` wrapper with timeout for stuck slaves.
4. Measure rise time with 4.7 kΩ vs 2.2 kΩ pull-ups.

---

## References

- NXP I²C specification (UM10204)
- [examples/i2c-scanner.md](../examples/i2c-scanner.md)
- [sensors/bme280.md](../sensors/bme280.md)

---

*Prev: [spi.md](./spi.md) | Next: [can.md](./can.md)*

*Back to [Embedded Swift](../README.md)*
