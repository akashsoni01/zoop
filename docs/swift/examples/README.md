# Embedded Swift Examples

Hands-on **code walkthroughs** with fully commented Swift sketches, ownership/ARC notes, and compile instructions. Examples target **ESP32-S3 DevKitC-1** where noted.

---

## Purpose

Core lessons teach concepts. **Examples** show those concepts applied to real peripherals — copy, adapt, and experiment.

Each example includes:

- Prerequisites and wiring summary
- Commented Swift code (HAL-based)
- **Ownership / ARC** notes
- **Compile & flash** commands
- Links to [communication/](../communication/README.md), [sensors/](../sensors/README.md), [displays/](../displays/README.md)

---

## Recommended Order

```
blink-led → button → interrupts → timers → pwm
    → uart-echo → i2c-scanner → sensor-driver
    → wifi-client → mqtt-client
```

---

## Example Index

### GPIO & Timing

| Example | Description |
|---------|-------------|
| [blink-led.md](./blink-led.md) | First output |
| [button.md](./button.md) | Input + debounce |
| [interrupts.md](./interrupts.md) | GPIO ISR |
| [timers.md](./timers.md) | Periodic tasks |
| [pwm.md](./pwm.md) | LED dimming, servo |

### Analog & DMA

| Example | Description |
|---------|-------------|
| [adc.md](./adc.md) | Read potentiometer |
| [dac.md](./dac.md) | Analog output |
| [dma.md](./dma.md) | Fast SPI transfer |

### Serial Buses

| Example | Description |
|---------|-------------|
| [uart-echo.md](./uart-echo.md) | Serial console |
| [spi-display.md](./spi-display.md) | ST7789 TFT |
| [i2c-scanner.md](./i2c-scanner.md) | Bus discovery |

### Sensors & Displays

| Example | Description |
|---------|-------------|
| [sensor-driver.md](./sensor-driver.md) | Driver pattern |
| [oled.md](./oled.md) | SSD1306 128×64 |
| [lcd.md](./lcd.md) | Character LCD |

### Motors & Motion

| Example | Description |
|---------|-------------|
| [motor-driver.md](./motor-driver.md) | H-bridge DC |
| [servo.md](./servo.md) | PWM servo |
| [stepper.md](./stepper.md) | Step/dir |
| [encoder.md](./encoder.md) | Quadrature |

### Power & Time

| Example | Description |
|---------|-------------|
| [rtc.md](./rtc.md) | Real-time clock |
| [deep-sleep.md](./deep-sleep.md) | Ultra-low power |

### Wireless & Cloud

| Example | Description |
|---------|-------------|
| [wifi-client.md](./wifi-client.md) | Join AP |
| [ble-server.md](./ble-server.md) | GATT peripheral |
| [http-server.md](./http-server.md) | Web UI |
| [mqtt-client.md](./mqtt-client.md) | Pub/sub |
| [ota-update.md](./ota-update.md) | Remote firmware |

### USB

| Example | Description |
|---------|-------------|
| [usb-hid.md](./usb-hid.md) | HID device |
| [usb-cdc.md](./usb-cdc.md) | Virtual serial |

### Storage

| Example | Description |
|---------|-------------|
| [file-system.md](./file-system.md) | LittleFS on flash |
| [flash-storage.md](./flash-storage.md) | Raw partitions |
| [eeprom.md](./eeprom.md) | Persistent config |

### Concurrency

| Example | Description |
|---------|-------------|
| [async-concurrency.md](./async-concurrency.md) | Structured async |
| [realtime.md](./realtime.md) | Deterministic loops |

---

## Related Sections

- [communication/](../communication/README.md) — protocol theory
- [projects/](../projects/README.md) — full builds
- [boards/](../boards/README.md) — pin maps and setup

---

*Back to [Embedded Swift](../README.md)*
