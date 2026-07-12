//! PCF85063 RTC stub — I2C read/write pending HIL.

use log::info;

use crate::board::config::{I2C_RTC_ADDR, I2C_SCL_PIN, I2C_SDA_PIN};

pub struct RtcChip;

impl RtcChip {
    pub fn init() -> Self {
        info!(
            "rtc: PCF85063 @ 0x{I2C_RTC_ADDR:02X} on I2C {I2C_SDA_PIN}/{I2C_SCL_PIN} (HIL pending)"
        );
        Self
    }

    pub fn read_utc_iso(&self) -> Option<String> {
        None
    }

    pub fn write_utc_iso(&self, iso: &str) {
        info!("rtc: write_utc_iso stub ({iso})");
    }

    pub fn sync_system_from_chip(&self) {
        info!("rtc: sync_system_from_chip stub");
    }

    pub fn sync_chip_from_system(&self) {
        info!("rtc: sync_chip_from_system stub");
    }
}

impl Default for RtcChip {
    fn default() -> Self {
        Self::init()
    }
}
