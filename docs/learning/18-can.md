# Lesson 18 — CAN Bus (Controller Area Network)

**Prerequisites:** [09-interrupts.md](./09-interrupts.md), [10-timers.md](./10-timers.md)

**Board focus:** ESP32 variants with TWAI (Two-Wire Automotive Interface — Espressif's CAN controller). STM32 is the industry reference.

---

## Theory

**CAN** (Controller Area Network) is a **multi-master serial bus** designed for noisy automotive and industrial environments. It uses **differential signaling** on two wires: **CAN_H** and **CAN_L**.

### Key properties

- **Broadcast bus:** All nodes see all frames; acceptance filtering is local.
- **Arbitration:** Non-destructive bit-wise priority by **ID** — lower ID wins.
- **No host required:** Any node can transmit when bus is idle.
- **Robust:** CRC (Cyclic Redundancy Check), ACK slot, error confinement.

### Frame format (Classical CAN 2.0A)

```
 SOF | ID (11b) | RTR | IDE | r0 | DLC | DATA (0–8) | CRC | ACK | EOF
```

| Field | Meaning |
|-------|---------|
| **ID** | Message identifier — also priority |
| **DLC** | Data Length Code — 0–8 bytes (CAN FD extends this) |
| **RTR** | Remote Transmission Request — request data without payload |
| **ACK** | Receiving nodes acknowledge correct CRC |

**CAN FD** (Flexible Data-rate): larger payload (64 bytes), dual bit rates — common in modern vehicles; check transceiver and controller support.

### Bit timing

CAN requires all nodes share **bitrate** within tight tolerance. Bit time splits into **Time Quanta (TQ)** segments:

```
Bit time = Sync + Prop + Phase1 + Phase2
Sample point typically at ~75–87.5% of bit time
```

Common bitrates: **125 kbit/s**, **250 kbit/s**, **500 kbit/s**, **1 Mbit/s**.

---

## Hardware Overview

### ESP32 / ESP32-S3 — TWAI

Espressif implements **TWAI** (compatible with ISO 11898-1 Classical CAN):

- Integrated controller — **requires external transceiver** (e.g., SN65HVD230, TJA1050).
- Filter hardware for ID/mask acceptance.
- Error counters, bus-off recovery, listen-only mode.

ESP32-S3 does **not** include a transceiver on DevKitC-1 — you must add a CAN HAT or module.

### STM32 bxCAN / FDCAN

STM32 Nucleo boards often need a transceiver shield; **FDCAN** supports CAN FD on newer chips.

### Physical layer

```
        CAN_H ───┬───────────────┬───
                 │   120 Ω       │
Node A ══════════╪═══════════════╪══════════ Node B
                 │   (bus term)  │
        CAN_L ───┴───────────────┴───
```

**120 Ω termination** at **both ends** of the bus — required for signal integrity.

---

## ASCII Wiring

```
ESP32-S3                SN65HVD230 Transceiver      CAN Bus
┌──────────┐            ┌──────────────┐
│ GPIO4    ├── TX ─────►│ TXD          │
│ GPIO5    │◄─ RX ──────┤ RXD          │
│ 3V3      ├────────────►│ VCC          │
│ GND      ├────────────►│ GND          │
└──────────┘            │ CANH ────────┼──► to bus CAN_H
                        │ CANL ────────┼──► to bus CAN_L
                        └──────────────┘
                              │
                           120 Ω between CANH and CANL (at each end)
```

TWAI TX/RX are **logic-level** to the transceiver, not differential pins directly.

---

## Memory & Register Notes

### TWAI registers (conceptual)

| Register | Purpose |
|----------|---------|
| `TWAI_TIMING` | BRP, TSEG1, TSEG2, SJW — bit timing |
| `TWAI_FILTER` | Acceptance code/mask |
| `TWAI_TX` | TX buffer — ID, DLC, data |
| `TWAI_RX` | RX FIFO |
| `TWAI_STATUS` | TX/RX pending, bus-off, error passive |

### Bit timing example (500 kbit/s @ 80 MHz APB — illustrative)

```
Prescaler + time segments must yield 500 kbit/s and ~80% sample point
Verify with esp-hal / esp-idf twai_timing_config_t calculator
```

Always validate on a scope or CAN analyzer.

---

## HAL Example — TWAI send/receive (esp-idf-hal / esp-hal style)

```rust
// Conceptual esp-idf-hal TWAI — API names vary by crate version
// Requires external transceiver wired as above

use esp_idf_hal::gpio::PinDriver;
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_hal::twai::{TwaiDriver, TwaiConfig, TwaiMode, filter::SingleFilter};

fn main() -> anyhow::Result<()> {
    // Initialize TWAI at 500 kbit/s
    let config = TwaiConfig::new()
        .mode(TwaiMode::Normal)
        .bitrate(/* TwaiBitrate::Bitrate500K */);

    // tx_pin, rx_pin from GPIO matrix
    // let mut twai = TwaiDriver::new(peripheral, tx, rx, &config)?;

    // Acceptance filter: accept all (bring-up only)
    // twai.set_filter(SingleFilter::accept_all())?;

    // Transmit standard frame ID 0x123, 2 data bytes
    let frame = esp_idf_hal::twai::TwaiFrame::new(
        0x123,
        false, // standard ID
        &[0xDE, 0xAD],
    )?;
    // twai.transmit(&frame, ms(100))?;

    // Receive loop
    loop {
        // if let Ok(rx) = twai.receive(ms(1000)) {
        //     log::info!("ID=0x{:03X} data={:?}", rx.identifier(), rx.data());
        // }
    }
}
```

For bare `esp-hal` (no_std), check current TWAI support in [esp-hal releases](https://github.com/esp-rs/esp-hal) — TWAI landed in recent versions.

---

## Filters

Hardware filters reduce CPU load by rejecting unrelated IDs:

| Filter type | Behavior |
|-------------|----------|
| **Mask** | `(incoming_id & mask) == (filter_id & mask)` |
| **Dual / range** | Accept ID range — chip-specific |
| **Accept all** | Debug only — high CPU on busy bus |

Example: ECU only cares about `0x100–0x10F`:

```
filter_id = 0x100
mask      = 0x7F0   // ignore lower 4 bits
```

---

## Step-by-Step

1. Wire transceiver: TWAI TX→TXD, RX←RXD, termination, common ground.
2. Configure **bitrate** matching all bus nodes (or second USB-CAN adapter on PC).
3. Start in **listen-only** if joining live vehicle bus (avoid ACK disruption).
4. Send test frame; verify on PCAN-View / SavvyCAN / python-can.
5. Add filters for production IDs.
6. Implement error handling: bus-off → recovery sequence.

---

## Common Mistakes

| Mistake | Symptom | Fix |
|---------|---------|-----|
| No transceiver | TX toggles, no bus traffic | Add SN65HVD230 |
| Single termination | Reflections, errors | 120 Ω at both ends |
| Bitrate mismatch | Error frames, bus-off | Match all nodes |
| Missing common GND | Intermittent NACK | Tie grounds |
| ACK on empty bus | Error passive | Second node or loopback adapter |
| Wrong pin mux | Silent TX | Check GPIO matrix |
| CAN FD frame on classic bus | Errors | Match frame type |

---

## Debugging Tips

- **CAN analyzer** (USB-CAN, PEAK) — gold standard.
- Monitor **TEC/REC** error counters in TWAI status registers.
- **Listen-only mode** snoops without transmitting ACK.
- Oscilloscope on CAN_H/CAN_L — differential voltage should swing ~2 V.
- Log **bus-off** events via `defmt` — [25-debugging.md](./25-debugging.md).

---

## Performance Tips

- Use **hardware filters** — don't receive every frame in software.
- Batch processing: RX interrupt queues frames; parse in lower-priority task.
- **CAN FD** if payload > 8 bytes regularly — requires FDCAN/TWAI FD support.
- For OBD-II diagnostics, implement ISO-TP segmentation in software layer above CAN.

---

## Exercises

1. **Loopback two boards** at 250 kbit/s; measure round-trip with timer.
2. **Filter exercise:** Accept only ID `0x200`; confirm other IDs ignored.
3. **DBC parser:** Load a simplified DBC file on host; decode signals in Rust test — [26-testing.md](./26-testing.md).
4. **Bus-off recovery:** Force errors (disconnect termination); log recovery time.
5. **Compare STM32:** Port echo to `stm32f4xx-hal` bxCAN on Nucleo + transceiver.

---

## References

- [ISO 11898-1 CAN specification overview](https://www.iso.org/standard/63648.html)
- [ESP32 TWAI documentation (ESP-IDF)](https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-reference/peripherals/twai.html)
- [SocketCAN on Linux](https://www.kernel.org/doc/html/latest/networking/can.html) — protocol learning
- [boards/stm32.md](./boards/stm32.md) — bxCAN/FDCAN
- [09-interrupts.md](./09-interrupts.md)

---

*Previous: [17-i2c.md](./17-i2c.md) · Next: [19-usb.md](./19-usb.md)*
