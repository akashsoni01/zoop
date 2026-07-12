//! Typed UPI payment state machine for Zoop Pay.

use crate::error::{CoreError, CoreResult};

/// Application screens — UPI collect / history / settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppState {
    /// Home — fullscreen any-amount QR
    Idle,
    /// Pick a predefined price
    PricePick,
    /// Customer-facing UPI QR for a selected amount
    ShowQr,
    /// Awaiting bank / backend confirmation
    Waiting,
    /// Payment confirmed
    Success,
    Menu,
    History,
    HistoryDetail,
    CancelConfirm,
    Settings,
    DeviceInfo,
    /// Merchant VPA profile
    Merchant,
    Error,
}

/// Physical button events (REC / PWR).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ButtonEvent {
    None,
    Single,
    Long,
    Double,
}

/// High-level transition events for the payment state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Transition {
    HoldRec,
    ReleaseRec,
    RecSingle,
    RecLong,
    RecDouble,
    PwrSingle,
    PwrDouble,
    /// ShowQr → Waiting
    PaymentPending,
    /// Waiting → Success
    PaymentSuccess,
    /// Waiting / ShowQr → Error
    PaymentFailed,
    /// Success → Idle
    PaymentDone,
    MenuBack,
    OpenCollect,
    OpenPrices,
    OpenHistory,
    OpenMerchant,
    OpenSettings,
    OpenDeviceInfo,
    ExitMerchant,
    /// PricePick → ShowQr after amount chosen
    SelectPrice,
    CancelConfirmed,
    CancelDismissed,
    ErrorDismissed,
    WakeToMenu,
    WakeToCollect,
}

/// Testable state machine with explicit transitions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateMachine {
    state: AppState,
    /// Last payment / order number
    pub last_txn_num: i32,
    pub activity_reset: bool,
}

impl StateMachine {
    pub fn new() -> Self {
        Self {
            state: AppState::Idle,
            last_txn_num: -1,
            activity_reset: false,
        }
    }

    pub fn state(&self) -> AppState {
        self.state
    }

    pub fn set_last_txn_num(&mut self, num: i32) {
        self.last_txn_num = num;
    }

    pub fn apply(&mut self, event: Transition) -> CoreResult<AppState> {
        self.activity_reset = true;
        let next = match (self.state, event) {
            (
                AppState::Idle,
                Transition::RecSingle
                    | Transition::HoldRec
                    | Transition::OpenHistory
                    | Transition::RecLong
                    | Transition::RecDouble,
            ) => AppState::History,
            (AppState::Idle, Transition::PwrSingle) => AppState::Menu,
            (AppState::Idle, Transition::WakeToMenu) => AppState::Menu,
            (
                AppState::Idle,
                Transition::WakeToCollect | Transition::OpenCollect | Transition::OpenPrices,
            ) => AppState::PricePick,

            (AppState::PricePick, Transition::SelectPrice | Transition::RecSingle) => {
                AppState::ShowQr
            }
            (
                AppState::PricePick,
                Transition::MenuBack | Transition::RecLong | Transition::RecDouble,
            ) => AppState::Menu,
            (AppState::PricePick, Transition::PwrSingle) => AppState::PricePick,

            (AppState::ShowQr, Transition::ReleaseRec | Transition::PaymentPending) => {
                AppState::Waiting
            }
            (AppState::ShowQr, Transition::RecLong | Transition::RecDouble) => {
                AppState::CancelConfirm
            }
            (AppState::ShowQr, Transition::PaymentFailed) => AppState::Error,
            (AppState::ShowQr, Transition::MenuBack | Transition::PwrSingle) => {
                AppState::PricePick
            }
            (AppState::ShowQr, Transition::OpenHistory) => AppState::History,
            // REC on priced QR = confirm / start waiting (not history)
            (AppState::ShowQr, Transition::RecSingle) => AppState::Waiting,

            (AppState::Waiting, Transition::PaymentSuccess) => AppState::Success,
            (AppState::Waiting, Transition::PaymentFailed) => AppState::Error,
            (
                AppState::Waiting,
                Transition::RecLong | Transition::RecDouble | Transition::MenuBack,
            ) => AppState::CancelConfirm,

            (AppState::Success, Transition::PaymentDone | Transition::RecSingle) => AppState::Idle,

            (AppState::Menu, Transition::OpenCollect | Transition::OpenPrices) => {
                AppState::PricePick
            }
            (AppState::Menu, Transition::OpenHistory) => AppState::History,
            (AppState::Menu, Transition::OpenMerchant) => AppState::Merchant,
            (AppState::Menu, Transition::OpenSettings) => AppState::Settings,
            (
                AppState::Menu,
                Transition::MenuBack | Transition::RecLong | Transition::RecDouble,
            ) => AppState::Idle,

            (
                AppState::History,
                Transition::RecSingle
                    | Transition::MenuBack
                    | Transition::RecLong
                    | Transition::RecDouble,
            ) => AppState::Idle,
            (AppState::History, Transition::OpenHistory) => AppState::HistoryDetail,

            (AppState::HistoryDetail, Transition::RecLong) => AppState::CancelConfirm,
            (
                AppState::HistoryDetail,
                Transition::RecDouble | Transition::MenuBack | Transition::RecSingle,
            ) => AppState::History,

            (AppState::CancelConfirm, Transition::CancelConfirmed | Transition::RecSingle) => {
                AppState::Idle
            }
            (
                AppState::CancelConfirm,
                Transition::CancelDismissed
                    | Transition::MenuBack
                    | Transition::RecLong
                    | Transition::RecDouble,
            ) => AppState::PricePick,

            (AppState::Settings, Transition::OpenDeviceInfo) => AppState::DeviceInfo,
            (
                AppState::Settings,
                Transition::MenuBack | Transition::RecLong | Transition::RecDouble,
            ) => AppState::Menu,

            (
                AppState::DeviceInfo,
                Transition::MenuBack
                    | Transition::RecLong
                    | Transition::RecDouble
                    | Transition::RecSingle
                    | Transition::PwrSingle,
            ) => AppState::Settings,

            (
                AppState::Merchant,
                Transition::ExitMerchant
                    | Transition::MenuBack
                    | Transition::RecLong
                    | Transition::RecDouble
                    | Transition::RecSingle,
            ) => AppState::Menu,

            (AppState::Error, Transition::ErrorDismissed) => AppState::Idle,

            _ => {
                return Err(CoreError::InvalidTransition {
                    from: self.state,
                    event,
                });
            }
        };
        self.state = next;
        Ok(next)
    }

    pub fn take_activity_reset(&mut self) -> bool {
        let v = self.activity_reset;
        self.activity_reset = false;
        v
    }
}

impl Default for StateMachine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idle_rec_opens_history() {
        let mut sm = StateMachine::new();
        assert_eq!(
            sm.apply(Transition::RecSingle).expect("ok"),
            AppState::History
        );
        assert_eq!(sm.apply(Transition::MenuBack).expect("ok"), AppState::Idle);
    }

    #[test]
    fn price_pick_to_qr_to_success() {
        let mut sm = StateMachine::new();
        sm.apply(Transition::OpenPrices).expect("prices");
        assert_eq!(sm.state(), AppState::PricePick);
        sm.apply(Transition::SelectPrice).expect("qr");
        assert_eq!(sm.state(), AppState::ShowQr);
        sm.apply(Transition::PaymentPending).expect("wait");
        assert_eq!(
            sm.apply(Transition::PaymentSuccess).expect("ok"),
            AppState::Success
        );
    }

    #[test]
    fn menu_opens_prices() {
        let mut sm = StateMachine::new();
        sm.apply(Transition::PwrSingle).expect("menu");
        assert_eq!(
            sm.apply(Transition::OpenPrices).expect("ok"),
            AppState::PricePick
        );
    }

    #[test]
    fn invalid_transition_errors() {
        let mut sm = StateMachine::new();
        assert!(sm.apply(Transition::ReleaseRec).is_err());
    }
}
