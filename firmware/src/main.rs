mod board;

use board::config::FIRMWARE_VERSION;
use board::secrets::TRANSCRIPTION_HOST;
use esp_idf_svc::sys::link_patches;
use log::info;

fn main() {
    link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("=== Zoop {FIRMWARE_VERSION} ===");
    info!(
        "Transcription host: {TRANSCRIPTION_HOST} (Phase 0 scaffold — BSP bring-up starts in Phase 1)"
    );
}
