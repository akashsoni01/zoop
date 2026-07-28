# Communication Protocols — Embedded Swift

This section covers **how embedded devices talk** — from a single GPIO pin to full TCP/IP stacks. Each guide includes theory, timing diagrams, packet formats, electrical specs, Swift HAL and bare-metal sketches, example projects, common mistakes, exercises, and references.

**Primary board:** [ESP32-S3 DevKitC-1](../boards/esp32-s3.md) unless noted otherwise.

---

## How to Use These Guides

1. Complete core lessons first: [08-gpio.md](../08-gpio.md) through [13-dac.md](../13-dac.md).
2. Pick a protocol matching your hardware (check BOM in [projects/](../projects/README.md)).
3. Read **Theory** and **Electrical characteristics** before wiring.
4. Build the **HAL example**; then study the **bare-metal sketch** for register-level details.
5. Cross-check timing with a logic analyzer when signals misbehave.

---

## Protocol Index

### On-Chip / Board-Level

| Guide | Summary | Prerequisites |
|-------|---------|---------------|
| [gpio.md](./gpio.md) | Digital I/O, pull-ups, drive strength | [08-gpio.md](../08-gpio.md) |
| [uart-usart.md](./uart-usart.md) | Async serial, framing, baud rates | [08-gpio.md](../08-gpio.md) |
| [spi.md](./spi.md) | Full-duplex synchronous bus | [08-gpio.md](../08-gpio.md) |
| [i2c.md](./i2c.md) | Two-wire multi-drop bus | [08-gpio.md](../08-gpio.md) |

### Field Buses & Automotive

| Guide | Summary |
|-------|---------|
| [can.md](./can.md) | Controller Area Network |
| [lin.md](./lin.md) | Local Interconnect Network |
| [rs232.md](./rs232.md) | Legacy ±12 V serial |
| [rs485.md](./rs485.md) | Differential multi-drop serial |
| [modbus.md](./modbus.md) | Industrial register protocol |

### USB & High-Speed Wired

| Guide | Summary |
|-------|---------|
| [usb.md](./usb.md) | USB device/host fundamentals |
| [ethernet.md](./ethernet.md) | IEEE 802.3, PHY, TCP/IP stack |

### Wireless

| Guide | Summary |
|-------|---------|
| [wifi.md](./wifi.md) | 802.11 STA/AP |
| [ble.md](./ble.md) | Bluetooth Low Energy GATT |
| [lora.md](./lora.md) | Long-range sub-GHz radio |
| [zigbee.md](./zigbee.md) | 802.15.4 mesh |

### Application-Layer

| Guide | Summary |
|-------|---------|
| [mqtt.md](./mqtt.md) | Pub/sub IoT messaging |
| [http.md](./http.md) | REST, embedded HTTP |
| [websockets.md](./websockets.md) | Full-duplex over HTTP upgrade |

---

## Related Sections

- **Hands-on code:** [examples/](../examples/README.md)
- **Full builds:** [projects/](../projects/README.md)
- **Sensors & displays:** [sensors/](../sensors/README.md), [displays/](../displays/README.md)

---

*Back to [Embedded Swift](../README.md)*
