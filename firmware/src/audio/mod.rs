//! ES8311 audio stub — FFI to `esp_codec_dev` in HIL phase.

use log::info;
use zoop_core::error::CoreResult;
use zoop_core::io::{Audio, SoundKind};

pub struct Es8311Audio;

impl Es8311Audio {
    pub fn init() -> Self {
        info!("audio: ES8311 stub (I2S + esp_codec_dev — HIL pending)");
        Self
    }
}

impl Audio for Es8311Audio {
    fn set_volume(&mut self, level: u8) {
        info!("audio: volume {level}");
    }

    fn play_beep(&mut self, kind: SoundKind) {
        info!("audio: beep {kind:?}");
    }

    fn start_record(&mut self) -> CoreResult<()> {
        info!("audio: start_record stub");
        Ok(())
    }

    fn stop_record(&mut self) {
        info!("audio: stop_record stub");
    }

    fn read_record_chunk(&mut self, _buf: &mut [u8]) -> CoreResult<usize> {
        Ok(0)
    }

    fn start_playback(&mut self, path: &str) -> CoreResult<()> {
        info!("audio: playback stub {path}");
        Ok(())
    }

    fn stop_playback(&mut self) {
        info!("audio: stop_playback stub");
    }

    fn is_playing(&self) -> bool {
        false
    }
}
