//! WiFi, Whisper, and portal — firmware shells over `zoop-core`.

use log::info;

pub mod portal;
pub mod whisper;
pub mod wifi;

pub fn init_network_stubs() {
    info!("network: WiFi/portal/Whisper stubs (Phase 3 — HIL pending)");
}
