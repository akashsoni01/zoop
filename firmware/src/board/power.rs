//! Board power rails — ESP-IDF GPIO stubs (HIL pending device).

use log::info;
use zoop_core::io::PowerRails;

use crate::board::config::{AUDIO_PWR_PIN, EPD_PWR_PIN, VBAT_PWR_PIN};

/// Firmware power rail driver (logs until GPIO wired in HIL).
pub struct BoardPower {
    log: Vec<&'static str>,
}

impl BoardPower {
    pub fn new() -> Self {
        Self { log: Vec::new() }
    }

    pub fn init(&mut self) {
        info!("board_power: init (Phase 1 stub — flash for HIL)");
        zoop_core::power_on_sequence(self);
    }
}

impl Default for BoardPower {
    fn default() -> Self {
        Self::new()
    }
}

impl PowerRails for BoardPower {
    fn battery_hold_on(&mut self) {
        self.log.push("battery_hold_on");
        info!("VBAT hold ON (GPIO{VBAT_PWR_PIN})");
    }

    fn epd_power_on(&mut self) {
        self.log.push("epd_power_on");
        info!("EPD rail ON (GPIO{EPD_PWR_PIN})");
    }

    fn audio_power_on(&mut self) {
        self.log.push("audio_power_on");
        info!("Audio rail ON (GPIO{AUDIO_PWR_PIN})");
    }

    fn epd_power_off(&mut self) {
        self.log.push("epd_power_off");
        info!("EPD rail OFF");
    }

    fn audio_power_off(&mut self) {
        self.log.push("audio_power_off");
        info!("Audio rail OFF");
    }

    fn logged_sequence(&self) -> &[&'static str] {
        &self.log
    }
}
