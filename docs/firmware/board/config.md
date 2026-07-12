# board/config.rs

- **Path:** `firmware/src/board/config.rs`
- **Purpose:** Board pins, timing, paths, and menu labels — mirrors `pala_note/config.h` and `board_cfg.h`.
- **Key types / functions:**
  - `FIRMWARE_VERSION`
  - EPD pins (`EPD_DC_PIN` … `EPD_PWR_PIN`), audio/battery power, `BTN_REC`/`BTN_PWR`
  - I2C addresses (RTC, SHTC3, ES8311), SDIO pins, I2S pins
  - Paths: `SD_MOUNT`, `NOTES_DIR`, `INDEX_FILE`, `TAG_FILE`
  - Timing: `REC_HOLD_MS`, `BTN_LONG_MS`, `ULTRA_SLEEP_MS`, battery thresholds
  - `MENU_ITEMS`, `SETTINGS_ITEMS`, `LOCAL_TIME_OFFSET_MIN`
- **Dependencies:** None
- **Tests:** Inline `#[cfg(test)]` module in source (pin sanity); host CI excludes firmware
- **Status:** Constants ready; GPIO use is HIL
- **Related:** [../../core/paths.md](../../core/paths.md), [power.md](power.md)
