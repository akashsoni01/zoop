# LoRa — Long Range Radio

**LoRa** (Long Range) is a ** chirp spread spectrum** modulation in **sub-GHz** ISM bands (868 MHz EU, 915 MHz US). **LoRaWAN** adds network layer; raw LoRa allows **point-to-point** custom protocols.

**Prerequisites:** [spi.md](./spi.md), [09-interrupts.md](../09-interrupts.md)

---

## Theory

Key parameters (trade range vs rate):

| Parameter | Effect |
|-----------|--------|
| **SF** (Spreading Factor 7–12) | Higher SF = longer range, slower |
| **BW** (Bandwidth 125–500 kHz) | Narrower = sensitivity |
| **CR** (Coding Rate 4/5–4/8) | Forward error correction |
| **TX power** | +2 to +20 dBm typical |

Semtech **SX1262** / **SX1276** modules (SPI) common with ESP32-S3 gateways.

---

## Timing Diagram (ASCII)

LoRa chirp (symbol — frequency sweeps):

```
Freq
 ↑    ╱────── chirp up (representing symbol)
 │   ╱
 │  ╱
 │ ╱
 └──────────────────→ time

Preamble: sequence of up-chirps for sync

TX packet timeline:

|-- preamble --|-- header --|-- payload symbols --|-- CRC --|
|<────────── Time on Air (ToA) ──────────────────────────>|
```

ToA depends on SF, payload length — use Semtech calculator.

---

## Packet Format

**LoRa PHY packet (SX1262):**

| Field | Description |
|-------|-------------|
| Preamble | Programmable length |
| Sync word | Network discrimination (0x1424 private / 0x3444 public) |
| Header | Explicit (CRC, length) or implicit |
| Payload | 1–255 bytes |
| CRC | Optional 16-bit |

**LoRaWAN** adds MAC header, DevAddr, FPort, MIC — see LoRa Alliance spec.

---

## Electrical Characteristics

| Parameter | Typical |
|-----------|---------|
| Supply | 3.3 V, 120 mA peak TX @ +22 dBm |
| SPI | Mode 0, ≤ 16 MHz |
| Antenna | 868/915 MHz matched — **never TX without load** |
| Range | 2–15+ km line-of-sight (SF12, outdoor) |

Add bulk capacitor (100 µF + 100 nF) near module for TX spikes.

---

## Rust HAL Sketch (sx126x driver)

```rust
use embedded_hal::spi::SpiDevice;
use sx126x::{Sx1262, Config};

pub fn init_lora<S: SpiDevice>(
    spi: S,
    reset: &mut impl OutputPin,
    busy: &impl InputPin,
) -> Sx1262<S> {
    let mut radio = Sx1262::new(spi, reset, busy);
    radio.init().unwrap();
    radio.set_frequency(868_000_000).unwrap();
    radio.set_spreading_factor(7).unwrap();
    radio.set_bandwidth(Bw::_125KHz).unwrap();
    radio
}

pub fn send_packet<S: SpiDevice>(radio: &mut Sx1262<S>, data: &[u8]) {
    radio.set_tx_buffer(data).unwrap();
    radio.transmit().unwrap();
    while !radio.tx_done().unwrap() {} // prefer DIO1 IRQ in production
}
```

**Ownership:** `Sx1262` owns SPI device; IRQ pin triggers receive in background task.

---

## Bare-Metal Sketch (Concept)

SX1262 opcode over SPI:

```rust
// WRITE_BUFFER @ 0x00, then SET_TX, busy pin polling
const OPCODE_SET_TX: u8 = 0x83;
// Full driver: sx126x-rs, lora-rs crates
```

---

## Example Projects

| Link | Use |
|------|-----|
| [projects/weather-station.md](../projects/weather-station.md) | Remote sensor uplink |
| [projects/iot-gateway.md](../projects/iot-gateway.md) | LoRaWAN gateway |
| [projects/data-logger.md](../projects/data-logger.md) | Field logging |

---

## Common Mistakes

1. **Wrong regional frequency** — legal issues (868 vs 915 MHz).
2. **Duty cycle exceeded** (EU 1%) — gateway drops or fines.
3. **Implicit header mode mismatch** between TX/RX.
4. **No DIO/IRQ wiring** — polling BUSY wastes CPU.
5. **SF mismatch** — receiver cannot decode.

---

## Exercises

1. Point-to-point ping at SF7 and SF12; compare RSSI/SNR.
2. Calculate ToA for 20-byte payload (Semtech tool vs measured air time).
3. Integrate with [examples/sensor-driver.md](../examples/sensor-driver.md) telemetry.
4. Add CAD (Channel Activity Detect) before TX.

---

## References

- Semtech SX1262 datasheet
- [LoRa Alliance](https://lora-alliance.org/)
- [lora-rs](https://github.com/lora-rs/lora-rs) ecosystem
- Lesson: [spi.md](./spi.md)

---

*Prev: [ble.md](./ble.md) | Next: [zigbee.md](./zigbee.md)*
