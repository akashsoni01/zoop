//! QR scan → string state machine (OLED UI track).

use crate::error::{CoreError, CoreResult};

/// Screens for the QR scan product.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QrAppState {
    Idle,
    Aiming,
    Decoded,
    Fail,
    Export,
}

/// Transition events for the QR scan state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QrTransition {
    StartScan,
    DebounceAccepted,
    ScanTimeout,
    Accept,
    Retry,
    DismissFail,
    ExportDone,
}

/// Testable QR scan state machine with explicit transitions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QrStateMachine {
    state: QrAppState,
}

impl QrStateMachine {
    pub fn new() -> Self {
        Self {
            state: QrAppState::Idle,
        }
    }

    pub fn state(&self) -> QrAppState {
        self.state
    }

    pub fn apply(&mut self, event: QrTransition) -> CoreResult<QrAppState> {
        let next = match (self.state, event) {
            (QrAppState::Idle | QrAppState::Fail, QrTransition::StartScan) => QrAppState::Aiming,
            (QrAppState::Aiming, QrTransition::DebounceAccepted) => QrAppState::Decoded,
            (QrAppState::Aiming, QrTransition::ScanTimeout) => QrAppState::Fail,
            (QrAppState::Decoded, QrTransition::Accept) => QrAppState::Export,
            (QrAppState::Export, QrTransition::ExportDone) => QrAppState::Idle,
            (QrAppState::Decoded | QrAppState::Fail, QrTransition::Retry) => QrAppState::Aiming,
            (QrAppState::Fail, QrTransition::DismissFail) => QrAppState::Idle,
            (from, event) => {
                return Err(CoreError::InvalidQrTransition { from, event });
            }
        };
        self.state = next;
        Ok(self.state)
    }
}

impl Default for QrStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn happy_path_idle_to_export_to_idle() {
        let mut sm = QrStateMachine::new();
        assert_eq!(sm.state(), QrAppState::Idle);
        assert_eq!(sm.apply(QrTransition::StartScan).unwrap(), QrAppState::Aiming);
        assert_eq!(
            sm.apply(QrTransition::DebounceAccepted).unwrap(),
            QrAppState::Decoded
        );
        assert_eq!(sm.apply(QrTransition::Accept).unwrap(), QrAppState::Export);
        assert_eq!(sm.apply(QrTransition::ExportDone).unwrap(), QrAppState::Idle);
    }

    #[test]
    fn timeout_and_retry() {
        let mut sm = QrStateMachine::new();
        sm.apply(QrTransition::StartScan).unwrap();
        assert_eq!(sm.apply(QrTransition::ScanTimeout).unwrap(), QrAppState::Fail);
        assert_eq!(sm.apply(QrTransition::Retry).unwrap(), QrAppState::Aiming);
    }

    #[test]
    fn dismiss_fail_to_idle() {
        let mut sm = QrStateMachine::new();
        sm.apply(QrTransition::StartScan).unwrap();
        sm.apply(QrTransition::ScanTimeout).unwrap();
        assert_eq!(sm.apply(QrTransition::DismissFail).unwrap(), QrAppState::Idle);
    }

    #[test]
    fn invalid_transition_errors() {
        let mut sm = QrStateMachine::new();
        let err = sm.apply(QrTransition::Accept).unwrap_err();
        assert_eq!(
            err,
            CoreError::InvalidQrTransition {
                from: QrAppState::Idle,
                event: QrTransition::Accept,
            }
        );
    }
}
