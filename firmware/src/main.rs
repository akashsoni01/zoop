mod app;
mod audio;
mod board;
mod display;
mod input;
mod network;
mod power;
mod storage;

use app::FirmwareEngine;
use audio::Es8311Audio;
use board::config::FIRMWARE_VERSION;
use board::power::BoardPower;
use board::rtc::RtcChip;
use board::secrets::{TRANSCRIPTION_HOST, TRANSCRIPTION_PROVIDER};
use display::EpaperDisplay;
use esp_idf_svc::sys::link_patches;
use log::info;
use network::{init_network_stubs, WhisperClient};
use storage::SdStorage;
use zoop_core::transcribe::TranscriptionConfig;

fn main() {
    link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("=== Zoop {FIRMWARE_VERSION} ===");
    info!("Transcription: {TRANSCRIPTION_PROVIDER} @ {TRANSCRIPTION_HOST}");

    let mut power = BoardPower::new();
    power.init();

    let display = EpaperDisplay::new();
    display.init();

    let storage = SdStorage::mount().expect("sd stub");
    let audio = Es8311Audio::init();
    let rtc = RtcChip::init();
    rtc.sync_system_from_chip();

    let whisper_config =
        TranscriptionConfig::from_secrets(TRANSCRIPTION_PROVIDER, "", "", Some(TRANSCRIPTION_HOST))
            .ok();
    let whisper = WhisperClient::new(whisper_config);
    whisper.log_config();

    init_network_stubs();

    let mut engine = FirmwareEngine::new(storage, display, audio, rtc, whisper);
    if let Err(e) = engine.boot() {
        info!("app: boot failed ({e}) — SD mount required for full app");
    }

    info!("BSP + app engine ready — host logic: `cargo test --workspace --exclude zoop-firmware`");

    loop {
        if let Err(e) = engine.tick() {
            info!("app: tick error {e}");
        }
        // Stub loop — real timing via esp-idf timer in HIL phase
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}
