//! Draft pin map for DIY AI Voice+Vision kit (ESP32-S3-CAM class).
//! Defaults follow Keyestudio KS5028-style wiring; verify silkscreen / schematic.
//! See [`docs/qr/hardware.md`](../../docs/qr/hardware.md).

#![allow(dead_code)] // draft until OLED / I²S / camera drivers land

/// OLED I²C SDA (draft — freeze after HIL).
pub const OLED_SDA: u8 = 8;
/// OLED I²C SCL (draft — freeze after HIL).
pub const OLED_SCL: u8 = 9;

/// Accept / confirm button (REC) — often BOOT/IO0 on CAM boards.
pub const BTN_REC: u8 = 0;
/// Cancel / power / retry (draft — avoid GPIO 1; KS5028 uses it for mic WS).
pub const BTN_PWR: u8 = 21;

// ─── INMP441 I²S RX (KS5028 example) ─────────────────────────────────────
/// Word select / LRCLK.
pub const I2S_MIC_WS: u8 = 1;
/// Bit clock / SCK.
pub const I2S_MIC_BCLK: u8 = 2;
/// Data from mic (SD).
pub const I2S_MIC_DIN: u8 = 42;

// ─── MAX98357A I²S TX (KS5028 example) ───────────────────────────────────
/// Data to amp (DIN).
pub const I2S_DAC_DOUT: u8 = 39;
/// Bit clock.
pub const I2S_DAC_BCLK: u8 = 40;
/// Word select / LRC.
pub const I2S_DAC_WS: u8 = 41;

// Camera: onboard 24-pin FPC (OV series) — board defaults via esp_camera.
