//! NTP time sync stub — uses `zoop_core::time` server policy.

use log::info;
use zoop_core::error::CoreResult;
use zoop_core::io::TimeSource;
use zoop_core::time::TimeSyncState;

pub struct NtpClient {
    pub state: TimeSyncState,
}

impl NtpClient {
    pub fn new() -> Self {
        Self {
            state: TimeSyncState::new(),
        }
    }

    /// One sync attempt step (stub until ESP-IDF SNTP wired).
    pub fn sync_step<T: TimeSource>(&mut self, time: &mut T) -> CoreResult<bool> {
        let server = self.state.current_server();
        info!("ntp: sync_step stub via {server}");
        let _ = time;
        Ok(false)
    }

    pub fn on_success<T: TimeSource>(&mut self, time: &mut T, utc_iso: &str) -> CoreResult<()> {
        time.set_utc_iso(utc_iso)?;
        self.state.on_sync_success();
        info!("ntp: time_ready=true");
        Ok(())
    }

    pub fn on_failure(&mut self) {
        self.state.on_sync_failed();
        info!("ntp: failed, next server {:?}", self.state.current_server());
    }
}

impl Default for NtpClient {
    fn default() -> Self {
        Self::new()
    }
}
