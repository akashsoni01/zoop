//! Button debounce, long-press, and double-tap — ports `buttons.cpp` without blocking delays.

use crate::io::ButtonEvents;
use crate::state::ButtonEvent;

pub const DEFAULT_DEBOUNCE_MS: u64 = 6;
pub const DEFAULT_REC_HOLD_MS: u64 = 350;
pub const DEFAULT_LONG_MS: u64 = 600;
pub const DEFAULT_DOUBLE_MS: u64 = 200;

/// Time-based button engine for host simulation and firmware polling.
#[derive(Debug, Clone)]
pub struct ButtonEngine {
    pub debounce_ms: u64,
    pub rec_hold_ms: u64,
    pub long_ms: u64,
    pub double_ms: u64,
    pwr_ready: bool,
    rec_down_since: Option<u64>,
    rec_release_at: Option<u64>,
    rec_waiting_double: bool,
    pwr_down_since: Option<u64>,
    idle_rec_down_since: Option<u64>,
}

impl ButtonEngine {
    pub fn new() -> Self {
        Self {
            debounce_ms: DEFAULT_DEBOUNCE_MS,
            rec_hold_ms: DEFAULT_REC_HOLD_MS,
            long_ms: DEFAULT_LONG_MS,
            double_ms: DEFAULT_DOUBLE_MS,
            pwr_ready: true,
            rec_down_since: None,
            rec_release_at: None,
            rec_waiting_double: false,
            pwr_down_since: None,
            idle_rec_down_since: None,
        }
    }

    pub fn with_timing(rec_hold_ms: u64, long_ms: u64, double_ms: u64) -> Self {
        let mut e = Self::new();
        e.rec_hold_ms = rec_hold_ms;
        e.long_ms = long_ms;
        e.double_ms = double_ms;
        e
    }

    fn debounced_down(down_since: &mut Option<u64>, pressed: bool, now_ms: u64, debounce_ms: u64) -> bool {
        if pressed {
            if down_since.is_none() {
                *down_since = Some(now_ms);
            }
            down_since
                .map(|t| now_ms.saturating_sub(t) >= debounce_ms)
                .unwrap_or(false)
        } else {
            *down_since = None;
            false
        }
    }

    pub fn poll_rec_raw(&mut self, pressed: bool, now_ms: u64) -> ButtonEvent {
        if !pressed {
            if let Some(down) = self.rec_down_since.take() {
                let held = now_ms.saturating_sub(down);
                if held >= self.long_ms {
                    return ButtonEvent::Long;
                }
                self.rec_release_at = Some(now_ms);
                self.rec_waiting_double = true;
            }
            return ButtonEvent::None;
        }

        if self.rec_down_since.is_none() {
            self.rec_down_since = Some(now_ms);
            return ButtonEvent::None;
        }

        let down = self.rec_down_since.unwrap();
        if now_ms.saturating_sub(down) < self.debounce_ms {
            return ButtonEvent::None;
        }

        if now_ms.saturating_sub(down) >= self.long_ms {
            return ButtonEvent::Long;
        }

        ButtonEvent::None
    }

    pub fn poll_rec_release(&mut self, now_ms: u64) -> ButtonEvent {
        if !self.rec_waiting_double {
            return ButtonEvent::None;
        }
        let release = match self.rec_release_at {
            Some(t) => t,
            None => return ButtonEvent::None,
        };

        if now_ms.saturating_sub(release) < self.double_ms {
            return ButtonEvent::None;
        }

        self.rec_waiting_double = false;
        self.rec_release_at = None;
        ButtonEvent::Single
    }

    pub fn poll_rec_double(&mut self, pressed: bool, now_ms: u64) -> ButtonEvent {
        if !self.rec_waiting_double {
            return ButtonEvent::None;
        }
        let release = match self.rec_release_at {
            Some(t) => t,
            None => return ButtonEvent::None,
        };
        if now_ms.saturating_sub(release) > self.double_ms {
            return ButtonEvent::None;
        }
        if pressed && Self::debounced_down(&mut self.rec_down_since, true, now_ms, self.debounce_ms) {
            self.rec_waiting_double = false;
            self.rec_release_at = None;
            self.rec_down_since = None;
            return ButtonEvent::Double;
        }
        ButtonEvent::None
    }

    pub fn poll_pwr_raw(&mut self, pressed: bool, now_ms: u64) -> ButtonEvent {
        if !pressed {
            self.pwr_down_since = None;
            self.pwr_ready = true;
            return ButtonEvent::None;
        }
        if !self.pwr_ready {
            return ButtonEvent::None;
        }
        if Self::debounced_down(&mut self.pwr_down_since, pressed, now_ms, self.debounce_ms) {
            self.pwr_ready = false;
            return ButtonEvent::Single;
        }
        ButtonEvent::None
    }

    pub fn poll_idle_rec_hold(&mut self, pressed: bool, now_ms: u64) -> bool {
        if !pressed {
            self.idle_rec_down_since = None;
            return false;
        }
        if self.idle_rec_down_since.is_none() {
            self.idle_rec_down_since = Some(now_ms);
            return false;
        }
        let down = self.idle_rec_down_since.unwrap();
        now_ms.saturating_sub(down) >= self.rec_hold_ms
    }
}

impl Default for ButtonEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Adapter that reads pin levels from a `Buttons` trait implementation.
pub struct ButtonPoller<B: crate::io::Buttons> {
    pub engine: ButtonEngine,
    pins: B,
}

impl<B: crate::io::Buttons> ButtonPoller<B> {
    pub fn new(pins: B) -> Self {
        Self {
            engine: ButtonEngine::new(),
            pins,
        }
    }

    pub fn pins(&self) -> &B {
        &self.pins
    }

    pub fn pins_mut(&mut self) -> &mut B {
        &mut self.pins
    }

    pub fn rec_pressed(&self) -> bool {
        self.pins.rec_pressed()
    }
}

impl<B: crate::io::Buttons> ButtonEvents for ButtonPoller<B> {
    fn poll_rec(&mut self, now_ms: u64) -> ButtonEvent {
        let pressed = self.pins.rec_pressed();
        if let ev @ (ButtonEvent::Long | ButtonEvent::Double) =
            self.engine.poll_rec_double(pressed, now_ms)
        {
            return ev;
        }
        if let ev @ ButtonEvent::Double = self.engine.poll_rec_double(pressed, now_ms) {
            return ev;
        }
        let raw = self.engine.poll_rec_raw(pressed, now_ms);
        if raw != ButtonEvent::None {
            return raw;
        }
        if !pressed {
            if let ev @ ButtonEvent::Long = self.engine.poll_rec_raw(false, now_ms) {
                return ev;
            }
            return self.engine.poll_rec_release(now_ms);
        }
        ButtonEvent::None
    }

    fn poll_pwr(&mut self, now_ms: u64) -> ButtonEvent {
        self.engine.poll_pwr_raw(self.pins.pwr_pressed(), now_ms)
    }

    fn idle_rec_hold_started(&mut self, now_ms: u64) -> bool {
        self.engine
            .poll_idle_rec_hold(self.pins.rec_pressed(), now_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pwr_single_requires_release_before_repeat() {
        let mut eng = ButtonEngine::new();
        assert_eq!(eng.poll_pwr_raw(true, 10), ButtonEvent::None);
        assert_eq!(eng.poll_pwr_raw(true, 20), ButtonEvent::Single);
        assert_eq!(eng.poll_pwr_raw(true, 30), ButtonEvent::None);
        assert_eq!(eng.poll_pwr_raw(false, 40), ButtonEvent::None);
        assert_eq!(eng.poll_pwr_raw(true, 50), ButtonEvent::None);
        assert_eq!(eng.poll_pwr_raw(true, 60), ButtonEvent::Single);
    }

    #[test]
    fn rec_long_fires_after_threshold() {
        let mut eng = ButtonEngine::with_timing(350, 600, 200);
        assert_eq!(eng.poll_rec_raw(true, 0), ButtonEvent::None);
        assert_eq!(eng.poll_rec_raw(true, 10), ButtonEvent::None);
        assert_eq!(eng.poll_rec_raw(true, 700), ButtonEvent::Long);
    }

    #[test]
    fn rec_single_after_double_window() {
        let mut eng = ButtonEngine::with_timing(350, 600, 200);
        eng.poll_rec_raw(true, 0);
        eng.poll_rec_raw(true, 10);
        eng.poll_rec_raw(false, 50);
        assert_eq!(eng.poll_rec_release(100), ButtonEvent::None);
        assert_eq!(eng.poll_rec_release(260), ButtonEvent::Single);
    }

    #[test]
    fn rec_double_within_window() {
        let mut eng = ButtonEngine::with_timing(350, 600, 200);
        eng.poll_rec_raw(true, 0);
        eng.poll_rec_raw(true, 10);
        eng.poll_rec_raw(false, 50);
        assert_eq!(eng.poll_rec_double(true, 100), ButtonEvent::None);
        assert_eq!(eng.poll_rec_double(true, 120), ButtonEvent::Double);
    }

    #[test]
    fn idle_rec_hold_requires_duration() {
        let mut eng = ButtonEngine::with_timing(350, 600, 200);
        assert!(!eng.poll_idle_rec_hold(true, 0));
        assert!(!eng.poll_idle_rec_hold(true, 100));
        assert!(eng.poll_idle_rec_hold(true, 400));
    }
}
