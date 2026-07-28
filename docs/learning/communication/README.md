# Communication Protocols — Embedded Rust

This section covers **how embedded devices talk** — from a single GPIO pin to full TCP/IP stacks. Each guide includes theory, timing diagrams, packet formats, electrical specs, Rust HAL and bare-metal sketches, example projects, common mistakes, exercises, and references.

**Primary board:** [ESP32-S3 DevKitC-1](../README.md#hardware-requirements) unless noted otherwise.

---

## How to Use These Guides

1. Complete core lessons first: [08-gpio.md](../08-gpio.md) through [13-dac.md](../13-dac.md).
2. Pick a protocol matching your hardware (check BOM in [projects/](../projects/README.md)).
3. Read **Theory** and **Electrical characteristics** before wiring.
4. Build the **HAL example**; then study the **bare-metal sketch** to see register-level details.
5. Cross-check timing with a logic analyzer when signals misbehave.

---

## Protocol Index

### On-Chip / Board-Level (GPIO & Serial Buses)

| Guide | Summary | Prerequisites |
|-------|---------|---------------|
| [gpio.md](./gpio.md) | Digital I/O, pull-ups, drive strength | [08-gpio.md](../08-gpio.md) |
| [uart-usart.md](./uart-usart.md) | Async serial, framing, baud rates | [08-gpio.md](../08-gpio.md) |
| [spi.md](./spi.md) | Full-duplex synchronous bus | [08-gpio.md](../08-gpio.md) |
| [i2c.md](./i2c.md) | Two-wire multi-drop bus | [08-gpio.md](../08-gpio.md) |

### Field Buses & Automotive

| Guide | Summary | Prerequisites |
|-------|---------|---------------|
| [can.md](./can.md) | Controller Area Network | [09-interrupts.md](../09-interrupts.md) |
| [lin.md](./lin.md) | Local Interconnect Network | [uart-usart.md](./uart-usart.md) |
| [rs232.md](./rs232.md) | Legacy ±12 V serial | [uart-usart.md](./uart-usart.md) |
| [rs485.md](./rs485.md) | Differential multi-drop serial | [uart-usart.md](./uart-usart.md) |
| [modbus.md](./modbus.md) | Industrial register protocol | [rs485.md](./rs485.md) |

### USB & High-Speed Wired

| Guide | Summary | Prerequisites |
|-------|---------|---------------|
| [usb.md](./usb.md) | USB device/host fundamentals | [09-interrupts.md](../09-interrupts.md) |
| [ethernet.md](./ethernet.md) | IEEE 802.3, PHY, TCP/IP stack | [07-hal.md](../07-hal.md) |

### Wireless

| Guide | Summary | Prerequisites |
|-------|---------|---------------|
| [wifi.md](./wifi.md) | 802.11 STA/AP on ESP32-S3 | [04-cargo.md](../04-cargo.md) |
| [ble.md](./ble.md) | Bluetooth Low Energy GATT | [09-interrupts.md](../09-interrupts.md) |
| [lora.md](./lora.md) | Long-range sub-GHz radio | [spi.md](./spi.md) |
| [zigbee.md](./zigbee.md) | 802.15.4 mesh (802.15.4 MAC) | [spi.md](./spi.md) |

### Application-Layer Protocols

| Guide | Summary | Prerequisites |
|-------|---------|---------------|
| [mqtt.md](./mqtt.md) | Pub/sub IoT messaging | [wifi.md](./wifi.md) |
| [http.md](./http.md) | REST, embedded HTTP server/client | [wifi.md](./wifi.md) |
| [websockets.md](./websockets.md) | Full-duplex over HTTP upgrade | [http.md](./http.md) |

---

## Learning Path by Use Case

```
Sensors on one board     → GPIO, I²C, SPI
Debug / boot console     → UART
Display / flash / radio  → SPI or I²C
Industrial / automotive  → CAN, RS-485, Modbus
Consumer IoT             → Wi-Fi + MQTT/HTTP
Wearables / beacons      → BLE
Long-range telemetry     → LoRa
```

---

## Related Sections

- **Hands-on code:** [examples/](../examples/README.md) — blink, UART echo, I²C scanner, Wi-Fi client, etc.
- **Full builds:** [projects/](../projects/README.md) — weather station, IoT gateway, CAN analyzer, etc.
- **Sensors & displays:** See [examples/sensor-driver.md](../examples/sensor-driver.md), [examples/oled.md](../examples/oled.md), [examples/lcd.md](../examples/lcd.md).
- **Glossary:** [glossary.md](../glossary.md)

---

*Back to [Learning Path](../README.md)*
