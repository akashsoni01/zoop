# Project: Data Logger

Log sensor CSV to SD card with [RTC](../examples/rtc.md) timestamps.

---

## BOM

ESP32-S3, microSD module (SPI), BME280, coin cell for RTC backup (if external RTC).

---

## Wiring

```
SD SPI: GPIO11 MOSI, GPIO12 SCK, GPIO13 MISO, GPIO10 CS
Sensor I²C: GPIO8/9
```

---

## Firmware Plan

1. SPI SD init — FAT via `embedded-sdmmc` or IDF VFS
2. [sensor-driver](../examples/sensor-driver.md) periodic sample
3. [file-system.md](../examples/file-system.md) append `log.csv`
4. [deep-sleep.md](../examples/deep-sleep.md) between samples

---

## Testing

- [ ] Files readable on PC
- [ ] Timestamp monotonic across sleep

---

## Extensions

- [LoRa](../communication/lora.md) metadata upload when gateway in range
