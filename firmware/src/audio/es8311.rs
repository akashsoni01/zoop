//! ES8311 audio codec stub — FFI to `esp_codec_dev` in HIL phase.

use log::info;
use zoop_core::error::CoreResult;
use zoop_core::io::{Audio, SoundKind};

use crate::board::config::{
    I2C_ES8311_ADDR, I2S_BCLK, I2S_DIN, I2S_DOUT, I2S_MCLK, I2S_PA, I2S_WS,
};

pub struct Es8311Audio;

impl Es8311Audio {
    pub fn init() -> Self {
        info!(
            "audio: ES8311 @ 0x{I2C_ES8311_ADDR:02X} I2S mclk={I2S_MCLK} bclk={I2S_BCLK} \
             ws={I2S_WS} din={I2S_DIN} dout={I2S_DOUT} pa={I2S_PA} (HIL pending)"
        );
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
