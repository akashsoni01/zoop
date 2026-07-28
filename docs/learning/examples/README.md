# Embedded Rust Examples

Hands-on **code walkthroughs** with fully commented Rust sketches, ownership notes, and compile instructions. Examples are **board-agnostic** where possible, with **ESP32-S3 DevKitC-1** notes throughout.

---

## Purpose

Numbered lessons ([01-rust-basics.md](../01-rust-basics.md)–[22-embassy.md](../22-embassy.md)) teach concepts. **Examples** show those concepts applied to real peripherals — copy, adapt, and experiment.

Each example includes:

- Prerequisites and wiring summary
- Commented Rust code (illustrative / HAL-based)
- **Ownership** explanation — who owns pins, peripherals, buffers
- **Compile & flash** commands for ESP32-S3
- Links to [communication/](../communication/README.md) protocol guides and [sensors/](../sensors/README.md) / [displays/](../displays/README.md)

---

## How to Follow Examples

1. **Complete prerequisites** listed in each guide (usually through [08-gpio.md](../08-gpio.md) and [07-hal.md](../07-hal.md)).
2. **Create a project** from [esp-template](https://github.com/esp-rs/esp-template) or copy the snippet into your firmware crate.
3. **Wire hardware** per the ASCII diagram in each doc (breadboard first).
4. **Build and flash** — fix errors top-down (toolchain → features → pins).
5. **Extend** — exercises at the end of each example suggest next steps.

Recommended order for beginners:

```
blink-led → button → interrupts → timers → pwm
    → uart-echo → i2c-scanner → sensor-driver
    → wifi-client → mqtt-client
```

---

## Prerequisites

| Requirement | Setup |
|-------------|-------|
| Rust stable + espup | [README § Rust Installation](../README.md#rust-installation) |
| ESP32-S3 DevKitC-1 | USB-C, built-in LED often GPIO48 |
| `espflash` | `cargo install espflash` |
| Optional components | Listed per example (LEDs, sensors, displays) |

Target triple: `xtensa-esp32s3-none-elf`

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

### Async Frameworks

| Example | Description |
|---------|-------------|
| [async-embassy.md](./async-embassy.md) | Embassy executor |
| [rtic.md](./rtic.md) | RTIC scheduler |

---

## Related Sections

- [communication/](../communication/README.md) — protocol theory
- [projects/](../projects/README.md) — full builds combining multiple examples
- [sensors/](../sensors/README.md) — BME280, IMU, GPS, etc.
- [displays/](../displays/README.md) — OLED, e-paper, TFT

---

*Back to [Learning Path](../README.md)*
