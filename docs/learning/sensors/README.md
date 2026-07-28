# Embedded Rust Sensor Learning Guides

Hands-on guides for reading physical-world signals with **Embedded Rust** on microcontrollers. Each guide explains the physics, the datasheet, the bus protocol, and how to write portable drivers using **embedded-hal**.

**Primary target board:** [ESP32-S3 DevKitC-1](../README.md#hardware-requirements) — wiring examples use ESP32-S3 pin names unless noted.

---

## Prerequisites

Complete these core lessons before diving into sensors:

| Lesson | Why You Need It |
|--------|-----------------|
| [07-hal.md](../07-hal.md) | `embedded-hal` traits (`I2c`, `DelayNs`, `InputPin`) |
| [08-gpio.md](../08-gpio.md) | Single-wire protocols, pull-ups, bit-banging |
| [10-timers.md](../10-timers.md) | Microsecond timing (DHT, HC-SR04, WS2812) |
| [12-adc.md](../12-adc.md) | Analog gas/IR sensors, microphone sampling |
| [16-spi.md](../16-spi.md) | SPI sensors (IMU, RFID, some displays) |
| [17-i2c.md](../17-i2c.md) | I²C sensors (environmental, light, RTC) |
| [09-interrupts.md](../09-interrupts.md) | Encoder edges, PPG FIFO, GPS NMEA parsing |

Keep [glossary.md](../glossary.md) open for acronyms (I²C, PPG, TOF, etc.).

---

## Learning Objectives

After working through this section you will be able to:

1. **Read a sensor datasheet** — identify supply voltage, interface, register map, conversion formulas, and timing constraints.
2. **Choose the right bus** — GPIO bit-bang vs I²C vs SPI vs 1-Wire vs UART vs ADC.
3. **Write portable Rust drivers** — implement or use crates that depend on `embedded-hal` traits, not a specific MCU.
4. **Debug hardware** — use a multimeter, logic analyzer, and `defmt` logs to isolate wiring vs timing vs driver bugs.
5. **Integrate sensors into projects** — sample rates, filtering, calibration, and power management on battery devices.

---

## Folder Structure

```
sensors/
├── README.md                 ← You are here
├── temperature-humidity.md   ← DHT11/22, AHT20, SHT31, DS18B20
├── bmp280.md / bme280.md     ← Pressure (+ humidity on BME)
├── imu-overview.md           ← Accelerometer/gyro/magnetometer concepts
├── mpu6050.md / mpu9250.md   ← InvenSense 6/9-axis
├── icm20948.md / lsm6ds3.md  ← TDK/ST alternatives
├── gps.md                    ← NMEA over UART
├── gas-sensors.md            ← MQ series, analog + digital
├── ir-sensors.md             ← Reflective, break-beam, remote
├── ultrasonic-hc-sr04.md    ← Time-of-flight via sound
├── lidar.md / tof-vl53l0x.md ← Laser / IR time-of-flight
├── current-ina219.md         ← I²C current/power monitor
├── voltage-ads1115.md        ← 16-bit external ADC
├── light-bh1750.md           ← Ambient lux
├── max30102.md               ← Pulse oximeter / PPG
├── hall-sensors.md           ← Magnetic field switches
├── touch-sensors.md          ← Capacitive touch ICs & pads
├── keypads.md / encoders.md  ← Matrix input & rotary
├── rfid-mfrc522.md           ← 13.56 MHz RFID (SPI)
├── nfc-pn532.md              ← NFC reader/writer
├── fingerprint.md            ← Optical/capacitive modules
├── microphones.md            ← I²S / PDM / analog
├── speakers.md               ← I²S DAC / PWM buzzer
├── rtc.md                    ← Real-time clock (DS3231, PCF8563)
├── eeprom.md                 ← I²C serial EEPROM
├── flash-memory.md           ← External SPI flash (W25Q)
└── camera-modules.md         ← OV2640, ESP32 camera pipeline
```

---

## Sensor Index by Category

### Environmental

| Guide | Sensor(s) | Interface | Difficulty |
|-------|-----------|-----------|------------|
| [temperature-humidity.md](./temperature-humidity.md) | DHT11, DHT22, AHT20, SHT31, DS18B20 | GPIO / I²C / 1-Wire | ★★☆ |
| [bmp280.md](./bmp280.md) | BMP280 | I²C / SPI | ★★☆ |
| [bme280.md](./bme280.md) | BME280 | I²C / SPI | ★★☆ |
| [light-bh1750.md](./light-bh1750.md) | BH1750 | I²C | ★☆☆ |
| [gas-sensors.md](./gas-sensors.md) | MQ-2, MQ-135, etc. | ADC / GPIO | ★★☆ |

### Motion & Orientation

| Guide | Sensor(s) | Interface | Difficulty |
|-------|-----------|-----------|------------|
| [imu-overview.md](./imu-overview.md) | Concepts | — | ★★☆ |
| [mpu6050.md](./mpu6050.md) | MPU-6050 | I²C | ★★☆ |
| [mpu9250.md](./mpu9250.md) | MPU-9250 | I²C / SPI | ★★★ |
| [icm20948.md](./icm20948.md) | ICM-20948 | I²C / SPI | ★★★ |
| [lsm6ds3.md](./lsm6ds3.md) | LSM6DS3 | I²C / SPI | ★★☆ |

### Distance & Ranging

| Guide | Sensor(s) | Interface | Difficulty |
|-------|-----------|-----------|------------|
| [ultrasonic-hc-sr04.md](./ultrasonic-hc-sr04.md) | HC-SR04 | GPIO + timer | ★★☆ |
| [tof-vl53l0x.md](./tof-vl53l0x.md) | VL53L0X | I²C | ★★☆ |
| [lidar.md](./lidar.md) | TF-Luna, RPLidar | UART / UART+PWM | ★★★ |
| [ir-sensors.md](./ir-sensors.md) | TCRT5000, IR LED+receiver | ADC / GPIO | ★☆☆ |

### Power & Analog Front-End

| Guide | Sensor(s) | Interface | Difficulty |
|-------|-----------|-----------|------------|
| [current-ina219.md](./current-ina219.md) | INA219 | I²C | ★★☆ |
| [voltage-ads1115.md](./voltage-ads1115.md) | ADS1115 | I²C | ★★☆ |

### Biometrics & Health

| Guide | Sensor(s) | Interface | Difficulty |
|-------|-----------|-----------|------------|
| [max30102.md](./max30102.md) | MAX30102 | I²C | ★★★ |

### Human Interface & Identification

| Guide | Sensor(s) | Interface | Difficulty |
|-------|-----------|-----------|------------|
| [hall-sensors.md](./hall-sensors.md) | A3144, DRV5032 | GPIO | ★☆☆ |
| [touch-sensors.md](./touch-sensors.md) | TTP223, MPR121 | GPIO / I²C | ★★☆ |
| [keypads.md](./keypads.md) | 4×4 matrix | GPIO | ★★☆ |
| [encoders.md](./encoders.md) | Rotary quadrature | GPIO + IRQ | ★★☆ |
| [rfid-mfrc522.md](./rfid-mfrc522.md) | MFRC522 | SPI | ★★★ |
| [nfc-pn532.md](./nfc-pn532.md) | PN532 | I²C / UART / SPI | ★★★ |
| [fingerprint.md](./fingerprint.md) | R307, AS608 | UART | ★★★ |

### Audio & Time

| Guide | Sensor(s) | Interface | Difficulty |
|-------|-----------|-----------|------------|
| [microphones.md](./microphones.md) | INMP441, analog electret | I²S / ADC | ★★★ |
| [speakers.md](./speakers.md) | MAX98357A, buzzer | I²S / PWM | ★★☆ |
| [rtc.md](./rtc.md) | DS3231, PCF8563 | I²C | ★★☆ |

### Storage & Imaging

| Guide | Sensor(s) | Interface | Difficulty |
|-------|-----------|-----------|------------|
| [eeprom.md](./eeprom.md) | AT24C256 | I²C | ★★☆ |
| [flash-memory.md](./flash-memory.md) | W25Q128 | SPI | ★★★ |
| [camera-modules.md](./camera-modules.md) | OV2640 | Parallel / ESP cam | ★★★★ |
| [gps.md](./gps.md) | NEO-6M, u-blox | UART | ★★☆ |

---

## Recommended Study Order

```
Environmental basics          Motion / ranging           Advanced
─────────────────────         ────────────────           ────────
temperature-humidity    →     imu-overview          →    gps
bmp280 / bme280               mpu6050                    rfid / nfc
light-bh1750                  ultrasonic-hc-sr04         fingerprint
                              tof-vl53l0x                camera-modules
current-ina219                encoders                   microphones
rtc                           keypads
```

Start with **one I²C sensor** (BMP280 or BH1750) and **one GPIO-timing sensor** (DHT22 or HC-SR04) to cover the two most common driver patterns.

---

## Exercises (Cross-Sensor)

### Exercise 1 — Sensor Dashboard (Beginner)

Wire BMP280 + BH1750 on the same I²C bus. Print temperature, pressure, and lux every second over `defmt`. **Goal:** shared bus, distinct I²C addresses.

### Exercise 2 — Motion Logger (Intermediate)

Use MPU-6050 at 100 Hz; detect free-fall (>2 g on all axes) and log peak acceleration. **Goal:** register configuration, signed 16-bit scaling, simple state machine.

### Exercise 3 — Power Monitor (Intermediate)

INA219 on the ESP32-S3 3.3 V rail (or a bench supply feeding the board). Report voltage, current, and mW every 500 ms. **Goal:** calibration register, shunt resistor math.

### Exercise 4 — Indoor Navigation Kit (Advanced)

Combine BME280 (environment), LSM6DS3 (IMU), VL53L0X (distance), and SSD1306 display (see [displays/](../displays/README.md)). Show a scrolling sensor panel. **Goal:** multi-driver integration, non-blocking sampling.

### Exercise 5 — Portable Driver Crate (Advanced)

Extract your BMP280 driver into a standalone crate with only `embedded-hal` + `embedded-hal-async` dependencies. Publish locally and consume from a second binary. **Goal:** trait-based portability.

---

## Common Tools & Crates

| Crate | Purpose |
|-------|---------|
| [embedded-hal](https://docs.rs/embedded-hal) | Trait definitions (`I2c`, `SpiDevice`, `DelayNs`) |
| [embedded-hal-async](https://docs.rs/embedded-hal-async) | Async trait equivalents |
| [defmt](https://docs.rs/defmt) | Structured logging |
| [heapless](https://docs.rs/heapless) | Fixed-size buffers for NMEA, samples |
| [micromath](https://docs.rs/micromath) | `no_std` math (altitude, tilt) |

Board-specific HAL: [`esp-hal`](https://docs.rs/esp-hal) (ESP32-S3), `stm32xx-hal`, `rp2040-hal`.

---

## Wiring Conventions (ESP32-S3)

All guides use these defaults unless a sensor requires otherwise:

| Signal | DevKitC-1 Pin | Notes |
|--------|---------------|-------|
| 3V3 | 3V3 | Most breakout boards |
| GND | GND | Common ground required |
| I²C SDA | GPIO8 | [17-i2c.md](../17-i2c.md) |
| I²C SCL | GPIO9 | 400 kHz Fast Mode |
| SPI MOSI | GPIO11 | [16-spi.md](../16-spi.md) |
| SPI MISO | GPIO13 | |
| SPI SCK | GPIO12 | |
| SPI CS | GPIO10 | Per-device, active low |

Logic level: ESP32-S3 GPIO is **3.3 V**. Do not connect 5 V sensor outputs directly — use level shifters or 3.3 V-compatible modules.

---

## Related Sections

- [Displays](../displays/README.md) — visualize sensor data on OLED/TFT
- [08-gpio.md](../08-gpio.md) — bit-banging and digital input
- [12-adc.md](../12-adc.md) — analog sensor front-end

---

*Pick a sensor from the index above, or start with [temperature-humidity.md](./temperature-humidity.md).*
