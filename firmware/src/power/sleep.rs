//! Ultra-sleep stub — ext1 wake on REC/PWR pending HIL.

use log::info;

use crate::board::config::{BTN_PWR, BTN_REC, ULTRA_SLEEP_MS};
use zoop_core::sleep::ActivityTimer;
use zoop_core::state::AppState;

pub struct SleepManager {
    pub timer: ActivityTimer,
}

impl SleepManager {
    pub fn new() -> Self {
        Self {
            timer: ActivityTimer::default(),
        }
    }

    pub fn should_sleep(&self, state: AppState, now_ms: u64) -> bool {
        self.timer.should_ultra_sleep(state, now_ms)
    }

    pub fn enter_ultra_sleep(&self) {
        info!(
            "sleep: enter_ultra_sleep after {ULTRA_SLEEP_MS}ms idle \
             (ext1 wake GPIO{BTN_REC}+GPIO{BTN_PWR} — HIL pending)"
        );
    }
}

impl Default for SleepManager {
    fn default() -> Self {
        Self::new()
    }
}
