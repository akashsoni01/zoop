//! Board pins, timing, and paths — mirrors `pala_note/config.h` and `board_cfg.h`.

/// Firmware version string (UART boot log).
pub const FIRMWARE_VERSION: &str = "v1.0";

/// Display dimensions (200×200 e-Paper).
pub const EPD_WIDTH: u16 = 200;
pub const EPD_HEIGHT: u16 = 200;

// ─── EPD SPI (SPI2) ───────────────────────────────────────────────────────
pub const EPD_DC_PIN: i32 = 10;
pub const EPD_CS_PIN: i32 = 11;
pub const EPD_SCK_PIN: i32 = 12;
pub const EPD_MOSI_PIN: i32 = 13;
pub const EPD_RST_PIN: i32 = 9;
pub const EPD_BUSY_PIN: i32 = 8;
pub const EPD_PWR_PIN: i32 = 6;

// ─── Power ────────────────────────────────────────────────────────────────
pub const AUDIO_PWR_PIN: i32 = 42;
pub const VBAT_PWR_PIN: i32 = 17;
pub const PWR_HOLD_PIN: i32 = 17;
pub const BAT_ADC_PIN: i32 = 4;

// ─── Buttons (active LOW, pull-up) ────────────────────────────────────────
pub const BTN_REC: i32 = 0;
pub const BTN_PWR: i32 = 18;

// ─── I2C ──────────────────────────────────────────────────────────────────
pub const I2C_SDA_PIN: i32 = 47;
pub const I2C_SCL_PIN: i32 = 48;
pub const I2C_RTC_ADDR: u8 = 0x51;
pub const I2C_SHTC3_ADDR: u8 = 0x70;
pub const I2C_ES8311_ADDR: u8 = 0x18;

// ─── SD-MMC (1-bit SDIO) ──────────────────────────────────────────────────
pub const SD_CLK: i32 = 39;
pub const SD_CMD: i32 = 41;
pub const SD_D0: i32 = 40;

// ─── I2S (ES8311) — from board_cfg.h S3_ePaper_1_54 ─────────────────────
pub const I2S_MCLK: i32 = 14;
pub const I2S_BCLK: i32 = 15;
pub const I2S_WS: i32 = 38;
pub const I2S_DIN: i32 = 16;
pub const I2S_DOUT: i32 = 45;
pub const I2S_PA: i32 = 46;
pub const I2S_PA_GAIN: i32 = 6;

// ─── Audio ────────────────────────────────────────────────────────────────
pub const SAMPLE_RATE: u32 = 16_000;
pub const REC_BUF: usize = 8 * 1024;

// ─── Storage paths (SD mount `/sdcard`) ───────────────────────────────────
pub const SD_MOUNT: &str = "/sdcard";
pub const NOTES_DIR: &str = "/notes";
pub const INDEX_FILE: &str = "/notes/index.csv";
pub const TAG_FILE: &str = "/notes/tags.txt";

// ─── UI timing (ms) ───────────────────────────────────────────────────────
pub const REC_HOLD_MS: u32 = 350;
pub const BTN_LONG_MS: u32 = 600;
pub const DOUBLE_MS: u32 = 200;
pub const ULTRA_SLEEP_MS: u64 = 120_000;
pub const TICKER_INTERVAL_MS: u32 = 950;

// ─── Battery ──────────────────────────────────────────────────────────────
pub const BAT_CHECK_INTERVAL_MS: u32 = 30_000;
pub const BAT_LOW_THRESHOLD: u8 = 15;
pub const BAT_RECOVER_THRESHOLD: u8 = 20;

// ─── Tags ─────────────────────────────────────────────────────────────────
pub const MAX_TAGS: usize = 20;

// ─── Time ─────────────────────────────────────────────────────────────────
pub const LOCAL_TIME_OFFSET_MIN: i32 = 120;

// ─── Menu / settings counts ───────────────────────────────────────────────
pub const MENU_COUNT: usize = 4;
pub const SETTINGS_COUNT: usize = 3;

pub const MENU_ITEMS: [&str; MENU_COUNT] = ["Notes", "Tags", "Sync", "Settings"];
pub const SETTINGS_ITEMS: [&str; SETTINGS_COUNT] = ["Sounds", "Transfer", "Device"];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timing_matches_config_h() {
        assert_eq!(SAMPLE_RATE, 16_000);
        assert_eq!(REC_HOLD_MS, 350);
        assert_eq!(ULTRA_SLEEP_MS, 120_000);
    }
}
