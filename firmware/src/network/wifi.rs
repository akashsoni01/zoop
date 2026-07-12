//! WiFi STA policy — delegates to `zoop_core::network::wifi`.

use log::info;
use zoop_core::network::wifi::{advance_wifi_connect, WifiConnectPhase, SYNC_MAX_ATTEMPTS};

pub struct WifiManager {
    pub phase: WifiConnectPhase,
}

impl WifiManager {
    pub fn new() -> Self {
        Self {
            phase: WifiConnectPhase::Idle,
        }
    }

    pub fn connect_step(&mut self, connected: bool, elapsed_ms: u64) {
        self.phase =
            advance_wifi_connect(self.phase, connected, elapsed_ms, 500, SYNC_MAX_ATTEMPTS);
        info!("wifi: phase {:?}", self.phase);
    }
}

impl Default for WifiManager {
    fn default() -> Self {
        Self::new()
    }
}
