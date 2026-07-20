//! OceanLabz DIY AI Voice Kit — GPIO draft.
//!
//! Source of truth: [`physical-components/hardware_spec.md`](../../physical-components/hardware_spec.md)
//! Verify against the wiring card in your kit box (HIL), then update both files together.

#![allow(dead_code)] // draft until OLED / I²S / camera drivers land

// ─── OLED I²C (draft — avoid mic SD on GPIO 41) ──────────────────────────
/// OLED I²C SDA.
pub const OLED_SDA: u8 = 8;
/// OLED I²C SCL.
pub const OLED_SCL: u8 = 9;

/// Accept / confirm (often BOOT / IO0).
pub const BTN_REC: u8 = 0;
/// Cancel / retry (draft free pin — confirm after HIL).
pub const BTN_PWR: u8 = 14;

// ─── INMP441 I²S RX (OceanLabz ESP32-S3 Camera board reference) ──────────
/// Word select / LRCLK.
pub const I2S_MIC_WS: u8 = 39;
/// Bit clock / SCK.
pub const I2S_MIC_BCLK: u8 = 40;
/// Data from mic (SD).
pub const I2S_MIC_DIN: u8 = 41;

// ─── MAX98357A I²S TX (OceanLabz ESP32-S3 Camera board reference) ────────
/// Word select / LRC.
pub const I2S_DAC_WS: u8 = 21;
/// Data to amp (DIN).
pub const I2S_DAC_DOUT: u8 = 47;
/// Bit clock.
pub const I2S_DAC_BCLK: u8 = 48;

// Camera: onboard 24-pin FPC (OV series) — board defaults via esp_camera.
// Amp Vin: often 5V per OceanLabz guides; GAIN commonly tied to GND.
