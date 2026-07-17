//! Draft pin map for ESP32-S3 camera + OLED QR kit.
//! Values mirror [`docs/qr/hardware.md`]; freeze after Phase 0 HIL.

#![allow(dead_code)] // draft constants until OLED / I²S / camera drivers land

/// OLED I²C SDA (draft).
pub const OLED_SDA: u8 = 8;
/// OLED I²C SCL (draft).
pub const OLED_SCL: u8 = 9;

/// Accept / confirm button (REC).
pub const BTN_REC: u8 = 0;
/// Cancel / power / retry button (PWR).
pub const BTN_PWR: u8 = 1;

/// I²S bit clock for INMP441 (placeholder).
pub const I2S_MIC_BCLK: u8 = 41;
/// I²S word select for INMP441 (placeholder).
pub const I2S_MIC_WS: u8 = 42;
/// I²S data in from INMP441 (placeholder).
pub const I2S_MIC_DIN: u8 = 40;

// MAX98357A I²S TX pins: TBD (share / time-slice with mic).
// Camera DVP / SCCB: TBD per board SKU (OV2640 / Sense defaults).
