//! Host-testable mock BSP implementations.

use std::collections::HashMap;

use crate::error::CoreResult;
use crate::io::{Audio, BatteryAdc, Buttons, Clock, Display, PowerRails, SoundKind, TimeSource};

pub const EPD_WIDTH: u16 = 200;
pub const EPD_HEIGHT: u16 = 200;
pub const FRAMEBUFFER_BYTES: usize = (EPD_WIDTH as usize * EPD_HEIGHT as usize) / 8;

/// In-memory 200×200 mono framebuffer display.
#[derive(Debug, Clone)]
pub struct MockDisplay {
    pub buffer: Vec<u8>,
    pub flush_count: u32,
}

impl MockDisplay {
    pub fn new() -> Self {
        Self {
            buffer: vec![0xFF; FRAMEBUFFER_BYTES],
            flush_count: 0,
        }
    }

    pub fn hash(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        self.buffer.hash(&mut h);
        h.finish()
    }

    pub fn is_solid(&self, white: bool) -> bool {
        let byte = if white { 0xFF } else { 0x00 };
        self.buffer.iter().all(|&b| b == byte)
    }
}

impl Default for MockDisplay {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for MockDisplay {
    fn flush(&mut self) -> CoreResult<()> {
        self.flush_count += 1;
        Ok(())
    }

    fn framebuffer_mut(&mut self) -> &mut [u8] {
        &mut self.buffer
    }

    fn width(&self) -> u16 {
        EPD_WIDTH
    }

    fn height(&self) -> u16 {
        EPD_HEIGHT
    }
}

/// Programmable button levels for tests.
#[derive(Debug, Clone, Default)]
pub struct MockButtons {
    pub rec: bool,
    pub pwr: bool,
}

impl Buttons for MockButtons {
    fn rec_pressed(&self) -> bool {
        self.rec
    }

    fn pwr_pressed(&self) -> bool {
        self.pwr
    }
}

/// Logs power rail transitions.
#[derive(Debug, Clone, Default)]
pub struct MockPowerRails {
    pub log: Vec<&'static str>,
}

impl MockPowerRails {
    pub fn new() -> Self {
        Self::default()
    }

    fn push(&mut self, step: &'static str) {
        self.log.push(step);
    }
}

impl PowerRails for MockPowerRails {
    fn battery_hold_on(&mut self) {
        self.push("battery_hold_on");
    }

    fn epd_power_on(&mut self) {
        self.push("epd_power_on");
    }

    fn audio_power_on(&mut self) {
        self.push("audio_power_on");
    }

    fn epd_power_off(&mut self) {
        self.push("epd_power_off");
    }

    fn audio_power_off(&mut self) {
        self.push("audio_power_off");
    }

    fn logged_sequence(&self) -> &[&'static str] {
        &self.log
    }
}

/// Records beeps and simulates record/playback.
#[derive(Debug, Clone, Default)]
pub struct MockAudio {
    pub beep_log: Vec<SoundKind>,
    pub recording: bool,
    pub playing: bool,
    pub record_chunks: Vec<Vec<u8>>,
    pub volume: u8,
}

impl MockAudio {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_pcm(&mut self, data: &[u8]) {
        self.record_chunks.push(data.to_vec());
    }
}

impl Audio for MockAudio {
    fn set_volume(&mut self, level: u8) {
        self.volume = level;
    }

    fn play_beep(&mut self, kind: SoundKind) {
        self.beep_log.push(kind);
    }

    fn start_record(&mut self) -> CoreResult<()> {
        self.recording = true;
        self.record_chunks.clear();
        Ok(())
    }

    fn stop_record(&mut self) {
        self.recording = false;
    }

    fn read_record_chunk(&mut self, buf: &mut [u8]) -> CoreResult<usize> {
        if !self.recording {
            return Ok(0);
        }
        let fill = buf.len().min(512);
        for b in &mut buf[..fill] {
            *b = 0x7F;
        }
        self.push_pcm(&buf[..fill]);
        Ok(fill)
    }

    fn start_playback(&mut self, _path: &str) -> CoreResult<()> {
        self.playing = true;
        Ok(())
    }

    fn stop_playback(&mut self) {
        self.playing = false;
    }

    fn is_playing(&self) -> bool {
        self.playing
    }
}

/// Fake clock with manual advance.
#[derive(Debug, Clone, Default)]
pub struct MockClock {
    pub now_ms: u64,
}

impl Clock for MockClock {
    fn now_ms(&self) -> u64 {
        self.now_ms
    }
}

/// Fixed UTC time for tests.
#[derive(Debug, Clone, Default)]
pub struct MockTime {
    pub utc: Option<String>,
}

impl TimeSource for MockTime {
    fn utc_iso(&self) -> Option<String> {
        self.utc.clone()
    }

    fn set_utc_iso(&mut self, iso: &str) -> CoreResult<()> {
        self.utc = Some(iso.to_string());
        Ok(())
    }
}

/// Simulated battery ADC samples.
#[derive(Debug, Clone)]
pub struct MockBatteryAdc {
    pub samples_mv: Vec<u32>,
}

impl MockBatteryAdc {
    pub fn with_voltage(v_battery: f32) -> Self {
        let adc_mv = (v_battery / 2.0 * 1000.0) as u32;
        Self {
            samples_mv: vec![adc_mv; 16],
        }
    }
}

impl BatteryAdc for MockBatteryAdc {
    fn read_mv_samples(&mut self, count: usize) -> CoreResult<Vec<u32>> {
        let n = count.min(self.samples_mv.len());
        Ok(self.samples_mv[..n].to_vec())
    }
}

/// Pin-level button schedule: `(time_ms, rec, pwr)`.
#[derive(Debug, Clone)]
pub struct ButtonSchedule {
    pub events: Vec<(u64, bool, bool)>,
}

impl ButtonSchedule {
    pub fn apply_at(&self, buttons: &mut MockButtons, now_ms: u64) {
        let mut rec = false;
        let mut pwr = false;
        for &(t, r, p) in &self.events {
            if t <= now_ms {
                rec = r;
                pwr = p;
            }
        }
        buttons.rec = rec;
        buttons.pwr = pwr;
    }
}

/// In-memory note file registry for portal tests.
#[derive(Debug, Default)]
pub struct MockFileRegistry {
    pub files: HashMap<String, Vec<u8>>,
}

impl MockFileRegistry {
    pub fn insert_str(&mut self, path: &str, content: &str) {
        self.files.insert(path.to_string(), content.as_bytes().to_vec());
    }
}
