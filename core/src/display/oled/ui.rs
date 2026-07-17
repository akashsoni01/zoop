//! Immediate-mode OLED screens for QR scan → string.

use crate::display::oled::draw::{
    clear, draw_battery_bar, draw_hints, draw_str_centered, BLACK, HEIGHT, WIDTH,
};
use crate::state_qr::QrAppState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OledScreenId {
    Idle,
    Aiming,
    Decoded,
    Fail,
    Export,
}

pub struct OledUiContext<'a> {
    pub buf: &'a mut [u8],
    pub battery_pct: Option<u8>,
    pub payload: &'a str,
    pub fail_reason: &'a str,
}

/// Truncate to `max_chars`, appending `…` when shortened.
pub fn truncate_ellipsis(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        return s.to_string();
    }
    if max_chars == 0 {
        return String::new();
    }
    let take = max_chars.saturating_sub(1);
    let mut out: String = s.chars().take(take).collect();
    out.push('…');
    out
}

impl OledUiContext<'_> {
    pub fn render(&mut self, state: QrAppState) -> OledScreenId {
        clear(self.buf);
        match state {
            QrAppState::Idle => {
                draw_str_centered(
                    self.buf,
                    (WIDTH / 2) as i32,
                    (HEIGHT as i32 / 2) - 4,
                    "ZOOP QR",
                    1,
                    BLACK,
                );
                if let Some(pct) = self.battery_pct {
                    draw_battery_bar(self.buf, pct);
                }
                OledScreenId::Idle
            }
            QrAppState::Aiming => {
                draw_str_centered(
                    self.buf,
                    (WIDTH / 2) as i32,
                    (HEIGHT as i32 / 2) - 4,
                    "Point at QR",
                    1,
                    BLACK,
                );
                OledScreenId::Aiming
            }
            QrAppState::Decoded => {
                let shown = truncate_ellipsis(self.payload, 18);
                draw_str_centered(
                    self.buf,
                    (WIDTH / 2) as i32,
                    20,
                    &shown,
                    1,
                    BLACK,
                );
                draw_hints(self.buf, "REC=OK", "PWR=Retry");
                OledScreenId::Decoded
            }
            QrAppState::Fail => {
                let reason = if self.fail_reason.is_empty() {
                    "No QR"
                } else {
                    self.fail_reason
                };
                draw_str_centered(
                    self.buf,
                    (WIDTH / 2) as i32,
                    20,
                    reason,
                    1,
                    BLACK,
                );
                draw_hints(self.buf, "", "PWR=Retry");
                OledScreenId::Fail
            }
            QrAppState::Export => {
                draw_str_centered(
                    self.buf,
                    (WIDTH / 2) as i32,
                    (HEIGHT as i32 / 2) - 4,
                    "Sent",
                    1,
                    BLACK,
                );
                OledScreenId::Export
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::display::oled::draw::BYTES;

    fn count_dark(buf: &[u8]) -> usize {
        buf.iter().map(|b| b.count_zeros() as usize).sum()
    }

    fn ctx<'a>(
        buf: &'a mut [u8],
        battery_pct: Option<u8>,
        payload: &'a str,
        fail_reason: &'a str,
    ) -> OledUiContext<'a> {
        OledUiContext {
            buf,
            battery_pct,
            payload,
            fail_reason,
        }
    }

    #[test]
    fn truncate_works() {
        assert_eq!(truncate_ellipsis("hi", 5), "hi");
        assert_eq!(truncate_ellipsis("abcdefghij", 5), "abcd…");
        assert_eq!(truncate_ellipsis("exactly18chars!!!!", 18).chars().count(), 18);
    }

    #[test]
    fn each_state_paints_dark_pixels() {
        let states = [
            QrAppState::Idle,
            QrAppState::Aiming,
            QrAppState::Decoded,
            QrAppState::Fail,
            QrAppState::Export,
        ];
        for state in states {
            let mut buf = vec![0xFF; BYTES];
            let mut ui = ctx(
                &mut buf,
                Some(75),
                "upi://pay?pa=merchant@oksbi&am=200.00",
                "Timeout",
            );
            let id = ui.render(state);
            assert!(count_dark(ui.buf) > 0, "{state:?} should paint dark pixels");
            let expected = match state {
                QrAppState::Idle => OledScreenId::Idle,
                QrAppState::Aiming => OledScreenId::Aiming,
                QrAppState::Decoded => OledScreenId::Decoded,
                QrAppState::Fail => OledScreenId::Fail,
                QrAppState::Export => OledScreenId::Export,
            };
            assert_eq!(id, expected);
        }
    }
}
