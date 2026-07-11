//! Trait-based I/O boundaries — mockable on host, implemented by firmware BSP.

use crate::error::CoreResult;
use crate::state::ButtonEvent;

/// Monochrome e-Paper display (200×200).
pub trait Display {
    /// Push the current framebuffer to the panel (full or partial refresh).
    fn flush(&mut self) -> CoreResult<()>;
    /// Raw 1-bit framebuffer bytes (`width * height / 8`).
    fn framebuffer_mut(&mut self) -> &mut [u8];
    fn width(&self) -> u16;
    fn height(&self) -> u16;
}

/// Audio record/playback and UI beeps.
pub trait Audio {
    fn set_volume(&mut self, level: u8);
    fn play_beep(&mut self, kind: SoundKind);
    fn start_record(&mut self) -> CoreResult<()>;
    fn stop_record(&mut self);
    fn read_record_chunk(&mut self, buf: &mut [u8]) -> CoreResult<usize>;
    fn start_playback(&mut self, path: &str) -> CoreResult<()>;
    fn stop_playback(&mut self);
    fn is_playing(&self) -> bool;
}

/// Physical button levels (active LOW on hardware).
pub trait Buttons {
    fn rec_pressed(&self) -> bool;
    fn pwr_pressed(&self) -> bool;
}

/// Power rail control — order matters for bring-up.
pub trait PowerRails {
    fn battery_hold_on(&mut self);
    fn epd_power_on(&mut self);
    fn audio_power_on(&mut self);
    fn epd_power_off(&mut self);
    fn audio_power_off(&mut self);
    fn logged_sequence(&self) -> &[&'static str];
}

/// UI sound kinds — ports `sounds.h`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SoundKind {
    Select,
    Next,
    Back,
    Saved,
    Delete,
    Success,
    Error,
}

/// Clock for timers (sleep, debounce, ticker).
pub trait Clock {
    fn now_ms(&self) -> u64;
}

/// Optional RTC / NTP time source.
pub trait TimeSource {
    fn utc_iso(&self) -> Option<String>;
    fn set_utc_iso(&mut self, iso: &str) -> CoreResult<()>;
}

/// Battery ADC reader.
pub trait BatteryAdc {
    fn read_mv_samples(&mut self, count: usize) -> CoreResult<Vec<u32>>;
}

/// Button event source (debounced).
pub trait ButtonEvents {
    fn poll_rec(&mut self, now_ms: u64) -> ButtonEvent;
    fn poll_pwr(&mut self, now_ms: u64) -> ButtonEvent;
    fn idle_rec_hold_started(&mut self, now_ms: u64) -> bool;
}
