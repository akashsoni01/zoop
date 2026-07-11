mod app;
mod audio;
mod board;
mod display;
mod input;
mod network;
mod power;
mod storage;

use board::config::FIRMWARE_VERSION;
use board::power::BoardPower;
use board::secrets::TRANSCRIPTION_HOST;
use display::EpaperDisplay;
use esp_idf_svc::sys::link_patches;
use log::info;
use network::init_network_stubs;

fn main() {
    link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("=== Zoop {FIRMWARE_VERSION} ===");
    info!("Transcription host: {TRANSCRIPTION_HOST}");

    let mut power = BoardPower::new();
    power.init();

    let display = EpaperDisplay::new();
    display.init();

    let _ = storage::SdStorage::mount();
    let _ = audio::Es8311Audio::init();
    init_network_stubs();

    info!("BSP stubs ready — host logic: `cd ../core && cargo test`");
    info!("M0 UART log OK (host-verified); HIL milestones M0–M4 pending device");
}
