//! Typed app state machine — ports `types.h` and `pala_note.ino` transitions.

use crate::error::{CoreError, CoreResult};

/// Application screens — mirrors `AppState` in `types.h`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppState {
    Idle,
    Recording,
    Saved,
    TagSelect,
    Menu,
    TagBrowser,
    NoteList,
    NoteDetail,
    DeleteConfirm,
    Settings,
    DeviceInfo,
    Transfer,
    Error,
}

/// Button events from `buttons.cpp`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ButtonEvent {
    None,
    Single,
    Long,
    Double,
}

/// High-level transition events for the state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Transition {
    HoldRec,
    ReleaseRec,
    RecSingle,
    RecLong,
    RecDouble,
    PwrSingle,
    PwrDouble,
    RecordSuccess,
    RecordFail,
    TagSaved,
    MenuSelect,
    MenuBack,
    OpenNotes,
    OpenTags,
    OpenSync,
    OpenSettings,
    OpenDeviceInfo,
    OpenTransfer,
    ExitTransfer,
    DeleteConfirmed,
    DeleteCancelled,
    ErrorDismissed,
    WakeToMenu,
    WakeToRec,
}

/// Testable state machine with explicit transitions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateMachine {
    state: AppState,
    pub last_rec_num: i32,
    pub activity_reset: bool,
}

impl StateMachine {
    pub fn new() -> Self {
        Self {
            state: AppState::Idle,
            last_rec_num: -1,
            activity_reset: false,
        }
    }

    pub fn state(&self) -> AppState {
        self.state
    }

    pub fn set_last_rec_num(&mut self, num: i32) {
        self.last_rec_num = num;
    }

    pub fn apply(&mut self, event: Transition) -> CoreResult<AppState> {
        self.activity_reset = true;
        let next = match (self.state, event) {
            (AppState::Idle, Transition::HoldRec) => AppState::Recording,
            (AppState::Idle, Transition::PwrSingle) => AppState::Menu,
            (AppState::Recording, Transition::ReleaseRec) => AppState::Saved,
            (AppState::Recording, Transition::RecordFail) => AppState::Error,
            (AppState::Saved, Transition::RecordSuccess) => AppState::TagSelect,
            (AppState::TagSelect, Transition::TagSaved) => AppState::Idle,
            (AppState::TagSelect, Transition::RecSingle | Transition::RecLong) => AppState::Idle,
            (AppState::Menu, Transition::OpenNotes) => AppState::NoteList,
            (AppState::Menu, Transition::OpenTags) => AppState::TagBrowser,
            (AppState::Menu, Transition::OpenSync) => AppState::Menu,
            (AppState::Menu, Transition::OpenSettings) => AppState::Settings,
            (
                AppState::Menu,
                Transition::MenuBack | Transition::RecLong | Transition::RecDouble,
            ) => AppState::Idle,
            (AppState::NoteList, Transition::RecSingle) => AppState::NoteDetail,
            (
                AppState::NoteList,
                Transition::MenuBack | Transition::RecLong | Transition::RecDouble,
            ) => AppState::Menu,
            (AppState::NoteDetail, Transition::RecLong) => AppState::DeleteConfirm,
            (AppState::NoteDetail, Transition::RecDouble | Transition::MenuBack) => {
                AppState::NoteList
            }
            (AppState::DeleteConfirm, Transition::RecSingle) => AppState::NoteList,
            (
                AppState::DeleteConfirm,
                Transition::MenuBack | Transition::RecLong | Transition::RecDouble,
            ) => AppState::NoteDetail,
            (AppState::Settings, Transition::OpenTransfer) => AppState::Transfer,
            (
                AppState::Settings,
                Transition::MenuBack | Transition::RecLong | Transition::RecDouble,
            ) => AppState::Menu,
            (AppState::Settings, Transition::OpenDeviceInfo) => AppState::DeviceInfo,
            (
                AppState::DeviceInfo,
                Transition::MenuBack | Transition::RecLong | Transition::RecDouble,
            ) => AppState::Settings,
            (
                AppState::Transfer,
                Transition::ExitTransfer | Transition::RecLong | Transition::RecDouble,
            ) => AppState::Settings,
            (AppState::Error, Transition::ErrorDismissed) => AppState::Idle,
            (
                AppState::TagBrowser,
                Transition::MenuBack | Transition::RecLong | Transition::RecDouble,
            ) => AppState::Menu,
            (AppState::TagBrowser, Transition::RecSingle) => AppState::NoteList,
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
    fn idle_hold_rec_starts_recording() {
        let mut sm = StateMachine::new();
        assert_eq!(
            sm.apply(Transition::HoldRec).expect("ok"),
            AppState::Recording
        );
    }

    #[test]
    fn idle_pwr_opens_menu() {
        let mut sm = StateMachine::new();
        assert_eq!(sm.apply(Transition::PwrSingle).expect("ok"), AppState::Menu);
    }

    #[test]
    fn recording_to_tag_select_flow() {
        let mut sm = StateMachine::new();
        sm.apply(Transition::HoldRec).expect("rec");
        sm.apply(Transition::ReleaseRec).expect("release");
        sm.state = AppState::Saved;
        assert_eq!(
            sm.apply(Transition::RecordSuccess).expect("ok"),
            AppState::TagSelect
        );
        assert_eq!(sm.apply(Transition::TagSaved).expect("ok"), AppState::Idle);
    }

    #[test]
    fn menu_to_note_list() {
        let mut sm = StateMachine::new();
        sm.apply(Transition::PwrSingle).expect("menu");
        assert_eq!(
            sm.apply(Transition::OpenNotes).expect("ok"),
            AppState::NoteList
        );
    }

    #[test]
    fn invalid_transition_errors() {
        let mut sm = StateMachine::new();
        assert!(sm.apply(Transition::ReleaseRec).is_err());
    }

    #[test]
    fn transfer_exit_returns_settings() {
        let mut sm = StateMachine::new();
        sm.state = AppState::Transfer;
        assert_eq!(
            sm.apply(Transition::ExitTransfer).expect("ok"),
            AppState::Settings
        );
    }
}
