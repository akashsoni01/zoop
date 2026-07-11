//! Whisper upload — uses `zoop_core::network::whisper` + ESP-TLS (HIL pending).

use log::info;
use zoop_core::transcribe::TranscriptionConfig;

pub struct WhisperClient {
    pub config: Option<TranscriptionConfig>,
}

impl WhisperClient {
    pub fn new(config: Option<TranscriptionConfig>) -> Self {
        Self { config }
    }

    pub fn log_config(&self) {
        if let Some(ref cfg) = self.config {
            info!("whisper: host {} (TLS upload — HIL pending)", cfg.host);
        }
    }
}
