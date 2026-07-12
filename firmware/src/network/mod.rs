//! WiFi, Whisper, NTP, and portal — firmware shells over `zoop-core`.

use log::info;

pub mod ntp;
pub mod portal;
pub mod whisper;
pub mod wifi;

pub use ntp::NtpClient;
pub use portal::TransferPortal;
pub use whisper::WhisperClient;
pub use wifi::WifiManager;

pub fn init_network_stubs() {
    let _wifi = WifiManager::new();
    let _portal = TransferPortal::default();
    let _ntp = NtpClient::new();
    info!("network: WiFi/portal/NTP/Whisper stubs (Phase 3 — HIL pending)");
}
