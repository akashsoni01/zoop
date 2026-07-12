//! NTP sync policy — ports `syncTimeFromNTP` server list and `timeReady` gating.

/// NTP servers tried in order (reference: `pala_note` network sync).
pub const NTP_SERVERS: &[&str] = &["pool.ntp.org", "time.google.com", "time.cloudflare.com"];

/// Tracks whether device time is trusted for note timestamps.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeSyncState {
    pub time_ready: bool,
    server_index: usize,
    attempts: u32,
}

impl TimeSyncState {
    pub fn new() -> Self {
        Self {
            time_ready: false,
            server_index: 0,
            attempts: 0,
        }
    }

    pub fn with_time_ready(ready: bool) -> Self {
        Self {
            time_ready: ready,
            ..Self::new()
        }
    }

    /// Current NTP server hostname for this attempt.
    pub fn current_server(&self) -> &'static str {
        NTP_SERVERS[self.server_index % NTP_SERVERS.len()]
    }

    /// Advance to next server after a failed sync attempt.
    pub fn on_sync_failed(&mut self) {
        self.attempts += 1;
        self.server_index = (self.server_index + 1) % NTP_SERVERS.len();
    }

    /// Mark time as ready after successful NTP + RTC write.
    pub fn on_sync_success(&mut self) {
        self.time_ready = true;
        self.attempts = 0;
    }

    pub fn attempt_count(&self) -> u32 {
        self.attempts
    }

    /// Gate note `created_utc` until NTP has run at least once.
    pub fn can_stamp_notes(&self) -> bool {
        self.time_ready
    }
}

impl Default for TimeSyncState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycles_ntp_servers_on_failure() {
        let mut state = TimeSyncState::new();
        assert_eq!(state.current_server(), "pool.ntp.org");
        state.on_sync_failed();
        assert_eq!(state.current_server(), "time.google.com");
        state.on_sync_failed();
        assert_eq!(state.current_server(), "time.cloudflare.com");
        state.on_sync_failed();
        assert_eq!(state.current_server(), "pool.ntp.org");
    }

    #[test]
    fn success_sets_time_ready() {
        let mut state = TimeSyncState::new();
        assert!(!state.can_stamp_notes());
        state.on_sync_success();
        assert!(state.time_ready);
        assert!(state.can_stamp_notes());
        assert_eq!(state.attempt_count(), 0);
    }

    #[test]
    fn default_server_list_matches_reference() {
        assert_eq!(NTP_SERVERS.len(), 3);
        assert!(NTP_SERVERS.contains(&"pool.ntp.org"));
    }
}
