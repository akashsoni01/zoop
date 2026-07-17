//! Consecutive-match debounce for live QR scan results.

/// Outcome of feeding one frame's decode into [`QrDebouncer`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DebounceOutcome {
    /// Still collecting identical matches (`matched` of `need`).
    Pending { matched: usize, need: usize },
    /// `need` consecutive identical payloads accepted.
    Accepted(String),
    /// Miss / clear — counter reset (e.g. `None` or empty decode).
    Cleared,
}

/// Requires `need` consecutive identical payloads before [`DebounceOutcome::Accepted`].
#[derive(Debug, Clone)]
pub struct QrDebouncer {
    need: usize,
    last: Option<String>,
    matched: usize,
}

impl QrDebouncer {
    /// Create a debouncer that accepts after `need` identical consecutive payloads.
    ///
    /// `need` of `0` is treated as `1` (immediate accept on first hit).
    pub fn new(need: usize) -> Self {
        Self {
            need: need.max(1),
            last: None,
            matched: 0,
        }
    }

    /// Clear streak state.
    pub fn reset(&mut self) {
        self.last = None;
        self.matched = 0;
    }

    /// Push one frame result (`None` = no QR / miss).
    pub fn push(&mut self, payload: Option<&str>) -> DebounceOutcome {
        let Some(payload) = payload else {
            self.reset();
            return DebounceOutcome::Cleared;
        };

        if self.last.as_deref() == Some(payload) {
            self.matched += 1;
        } else {
            self.last = Some(payload.to_string());
            self.matched = 1;
        }

        if self.matched >= self.need {
            let accepted = self.last.clone().unwrap_or_else(|| payload.to_string());
            self.reset();
            DebounceOutcome::Accepted(accepted)
        } else {
            DebounceOutcome::Pending {
                matched: self.matched,
                need: self.need,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_after_n() {
        let mut d = QrDebouncer::new(3);
        assert_eq!(
            d.push(Some("a")),
            DebounceOutcome::Pending {
                matched: 1,
                need: 3
            }
        );
        assert_eq!(
            d.push(Some("a")),
            DebounceOutcome::Pending {
                matched: 2,
                need: 3
            }
        );
        assert_eq!(d.push(Some("a")), DebounceOutcome::Accepted("a".into()));
    }

    #[test]
    fn flicker_never_accepts() {
        let mut d = QrDebouncer::new(3);
        for _ in 0..10 {
            assert!(matches!(
                d.push(Some("a")),
                DebounceOutcome::Pending { .. }
            ));
            assert!(matches!(
                d.push(Some("b")),
                DebounceOutcome::Pending { .. }
            ));
        }
        assert!(d.last.is_some());
        assert_eq!(d.matched, 1);
    }

    #[test]
    fn miss_clears() {
        let mut d = QrDebouncer::new(3);
        d.push(Some("a"));
        d.push(Some("a"));
        assert_eq!(d.push(None), DebounceOutcome::Cleared);
        assert_eq!(
            d.push(Some("a")),
            DebounceOutcome::Pending {
                matched: 1,
                need: 3
            }
        );
    }

    #[test]
    fn need_one_immediate() {
        let mut d = QrDebouncer::new(1);
        assert_eq!(d.push(Some("x")), DebounceOutcome::Accepted("x".into()));
    }
}
