//! Ultra-sleep and activity timer logic — ports `sleep.cpp` without ESP deep sleep.

use crate::state::AppState;

pub const DEFAULT_ULTRA_SLEEP_MS: u64 = 120_000;
pub const BAT_WARN_OVERLAY_MS: u64 = 2_500;

/// Tracks idle time and decides when ultra-sleep should trigger.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivityTimer {
    pub ultra_sleep_ms: u64,
    last_activity_ms: u64,
    pub low_battery_latched: bool,
    pub battery_warn_until_ms: Option<u64>,
}

impl ActivityTimer {
    pub fn new(ultra_sleep_ms: u64) -> Self {
        Self {
            ultra_sleep_ms,
            last_activity_ms: 0,
            low_battery_latched: false,
            battery_warn_until_ms: None,
        }
    }

    pub fn reset_activity(&mut self, now_ms: u64) {
        self.last_activity_ms = now_ms;
    }

    pub fn idle_ms(&self, now_ms: u64) -> u64 {
        now_ms.saturating_sub(self.last_activity_ms)
    }

    pub fn should_ultra_sleep(&self, state: AppState, now_ms: u64) -> bool {
        if matches!(
            state,
            AppState::Recording | AppState::Transfer | AppState::Error
        ) {
            return false;
        }
        self.idle_ms(now_ms) >= self.ultra_sleep_ms
    }

    pub fn update_battery_warning(
        &mut self,
        percent: Option<u8>,
        low_threshold: u8,
        recover_threshold: u8,
        now_ms: u64,
    ) -> bool {
        let Some(pct) = percent else {
            return false;
        };
        if pct <= low_threshold {
            self.low_battery_latched = true;
            self.battery_warn_until_ms = Some(now_ms + BAT_WARN_OVERLAY_MS);
            return true;
        }
        if pct >= recover_threshold {
            self.low_battery_latched = false;
        }
        false
    }

    pub fn battery_warning_active(&self, now_ms: u64) -> bool {
        self.battery_warn_until_ms
            .map(|until| now_ms < until)
            .unwrap_or(false)
    }

    pub fn clear_battery_warning_overlay(&mut self) {
        self.battery_warn_until_ms = None;
    }
}

impl Default for ActivityTimer {
    fn default() -> Self {
        Self::new(DEFAULT_ULTRA_SLEEP_MS)
    }
}

/// Wake cause after deep sleep (host-simulated).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WakeCause {
    PowerOn,
    RecHeld,
    PwrHeld,
    Timer,
}

/// Map GPIO wake mask to cause (reference: EXT1 on REC+PWR).
pub fn wake_cause_from_pins(rec_held: bool, pwr_held: bool) -> WakeCause {
    if rec_held {
        WakeCause::RecHeld
    } else if pwr_held {
        WakeCause::PwrHeld
    } else {
        WakeCause::PowerOn
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ultra_sleep_after_idle() {
        let mut t = ActivityTimer::new(120_000);
        t.reset_activity(0);
        assert!(!t.should_ultra_sleep(AppState::Idle, 60_000));
        assert!(t.should_ultra_sleep(AppState::Idle, 120_001));
    }

    #[test]
    fn no_sleep_during_recording_or_transfer() {
        let mut t = ActivityTimer::new(120_000);
        t.reset_activity(0);
        assert!(!t.should_ultra_sleep(AppState::Recording, 200_000));
        assert!(!t.should_ultra_sleep(AppState::Transfer, 200_000));
    }

    #[test]
    fn battery_warning_latches_and_recovers() {
        let mut t = ActivityTimer::new(120_000);
        assert!(t.update_battery_warning(Some(10), 15, 20, 1000));
        assert!(t.low_battery_latched);
        assert!(t.battery_warning_active(2000));
        t.update_battery_warning(Some(25), 15, 20, 3000);
        assert!(!t.low_battery_latched);
    }

    #[test]
    fn wake_cause_mapping() {
        assert_eq!(wake_cause_from_pins(true, false), WakeCause::RecHeld);
        assert_eq!(wake_cause_from_pins(false, true), WakeCause::PwrHeld);
    }
}
