# board/rtc.rs

- **Path:** `firmware/src/board/rtc.rs`
- **Purpose:** PCF85063 RTC stub — I2C read/write pending HIL.
- **Key types / functions:**
  - `RtcChip::init`, `read_utc_iso` (returns `None`), `write_utc_iso`, `sync_system_from_chip`, `sync_chip_from_system`
- **Dependencies:** `board::config` I2C pins/address, `log`
- **Tests:** N/A on host
- **Status:** HIL stub
- **Related:** [../../core/time.md](../../core/time.md), [../network/ntp.md](../network/ntp.md)
