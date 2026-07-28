# LoRa — Long Range Radio

**LoRa** (Long Range) is a **chirp spread spectrum** modulation in sub-GHz ISM bands (433/868/915 MHz). **LoRaWAN** adds network server, gateways, and OTAA join — this guide focuses on **LoRa PHY/MAC** at the module level.

**Prerequisites:** [spi.md](./spi.md), [09-interrupts.md](../09-interrupts.md)

---

## Theory

**Semtech transceivers** (SX1276, SX1262) connect via **SPI**. Key parameters:

| Parameter | Effect |
|-----------|--------|
| Spreading Factor (SF7–SF12) | Range vs data rate |
| Bandwidth (125/250/500 kHz) | Sensitivity vs speed |
| Coding Rate (4/5–4/8) | FEC overhead |
| TX power | +2 to +20 dBm (region limits) |

LoRaWAN adds class A/B/C device behavior and 1% duty cycle rules in EU868.

---

## Timing Diagram (ASCII)

LoRa chirp (conceptual — up-chirp encodes symbol):

```
Frequency
    ▲     ╱╲    ╱╲    ╱╲
    │    ╱  ╲  ╱  ╲  ╱  ╲
    │   ╱    ╲╱    ╲╱    ╲
    └──────────────────────► time
         SF7 symbol period
```

TX packet: preamble chirps → sync word → header → payload CRC.

---

## Packet Format

**LoRa modem payload (plain LoRa, not LoRaWAN MAC):**

| Field | Description |
|-------|-------------|
| Preamble | Programmable length |
| Sync word | Network/private (0x12 public) |
| Header | Explicit: len, CRC on/off |
| Payload | 1–255 bytes |
| CRC | Optional 16-bit |

**LoRaWAN MAC frame** adds MHDR, DevAddr, FCtrl, FCnt, FPort, FRMPayload, MIC.

---

## Electrical Characteristics

| Parameter | Typical |
|-----------|---------|
| Supply | 3.3 V, 120 mA peak TX |
| RF output | +14 dBm common (868 MHz) |
| Antenna | 50 Ω, band-matched |
| SPI | Mode 0, ≤ 10 MHz |
| DIO pins | TX/RX done interrupts |

Keep antenna away from metal; match region frequency (868 EU, 915 US).

---

## Swift HAL Sketch

```swift
import Embedded
import ESP32Hardware

struct SX1262Radio {
    var spi: SPI2
    var cs: GPIO.Output<GPIO10>
    var reset: GPIO.Output<GPIO9>
    var busy: GPIO.Input<GPIO8>

    mutating func initLoRa(freqHz: UInt32, sf: UInt8 = 7) throws {
        try resetPulse()
        try setStandby()
        try setFrequency(freqHz)
        try setModulationParams(sf: sf, bw: .k125, cr: .fourFive)
    }

    mutating func transmit(_ payload: [UInt8]) throws {
        try writeBuffer(payload)
        try setTx()
        try waitOnBusy(timeoutMs: 5000)
    }
}
```

**Ownership:** Radio driver struct owns SPI + control pins; exclusive access during TX/RX.

---

## Example Projects

| Link | Description |
|------|-------------|
| [projects/iot-gateway.md](../projects/iot-gateway.md) | LoRa → MQTT bridge |

---

## Common Mistakes

1. **Wrong frequency for region** — illegal TX.
2. **BUSY pin not polled** — SPI commands ignored.
3. **Antenna disconnected** — high VSWR, damaged PA.
4. **SF12 + long payload + duty cycle** — EU868 violation.
5. **Sharing SPI without CS discipline** — corrupted registers.

---

## Exercises

1. Send "hello" on 868 MHz; decode with second module.
2. Sweep SF7–SF12 and measure time-on-air.
3. Implement RX interrupt via DIO1.
4. Bridge packets to MQTT in [projects/iot-gateway.md](../projects/iot-gateway.md).

---

## References

- Semtech SX1262 datasheet
- LoRaWAN specification (LoRa Alliance)
- [projects/iot-gateway.md](../projects/iot-gateway.md)

---

*Prev: [ble.md](./ble.md) | Next: [zigbee.md](./zigbee.md)*

*Back to [Embedded Swift](../README.md)*
