//! E-Paper display driver stub — delegates draw to `zoop-core`.

use log::info;
use zoop_core::display::draw::BYTES;
use zoop_core::error::CoreResult;
use zoop_core::io::Display;

use crate::board::config::{EPD_HEIGHT, EPD_WIDTH};

pub struct EpaperDisplay {
    buffer: Vec<u8>,
}

impl EpaperDisplay {
    pub fn new() -> Self {
        Self {
            buffer: vec![0xFF; BYTES],
        }
    }

    pub fn init(&self) {
        info!(
            "epaper: init {EPD_WIDTH}x{EPD_HEIGHT} stub (SPI partial refresh — HIL pending)"
        );
    }
}

impl Default for EpaperDisplay {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for EpaperDisplay {
    fn flush(&mut self) -> CoreResult<()> {
        info!("epaper: flush {} bytes (stub)", self.buffer.len());
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
