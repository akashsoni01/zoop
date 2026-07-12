# sdkconfig.defaults

- **Path:** `firmware/sdkconfig.defaults`
- **Purpose:** ESP-IDF defaults for Zoop on Waveshare ESP32-S3-ePaper-1.54 (target, PSRAM, WiFi buffers, FAT LFN, sleep GPIO workaround).
- **Key types / functions:** N/A (Kconfig)
  - `CONFIG_IDF_TARGET=esp32s3`, 240 MHz, main stack 8192
  - OPI PSRAM 80 MHz, FreeRTOS 1000 Hz, FATFS LFN heap, WiFi RX buffers
- **Dependencies:** Referenced by `firmware/Cargo.toml` package metadata
- **Tests:** Applied during embuild
- **Status:** Active
- **Related:** [Cargo.firmware.md](Cargo.firmware.md), [cargo-config.md](cargo-config.md)
