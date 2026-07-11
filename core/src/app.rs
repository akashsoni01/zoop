//! Application engine — state machine + traits driving offline flows.

use crate::battery::{battery_percent_from_adc_samples, BatteryCurve};
use crate::buttons::ButtonPoller;
use crate::display::ui::{ScreenId, UiContext};
use crate::error::CoreResult;
use crate::io::{Audio, BatteryAdc, ButtonEvents, Clock, Display, SoundKind, TimeSource};
use crate::paths::note_path;
use crate::record::{finalize_new_note, RecordOutcome, RecordSession};
use crate::sleep::ActivityTimer;
use crate::sounds::SoundsPolicy;
use crate::state::{AppState, ButtonEvent, StateMachine, Transition};
use crate::storage::{
    delete_note, load_index, load_tags, next_note_number, FileStorage, IndexStore, TagStore,
};

pub const FIRMWARE_VERSION: &str = "v1.0";
pub const LOCAL_TIME_OFFSET_MIN: i32 = 120;

/// High-level app context wiring core logic to BSP traits.
pub struct App<'a, S, D, A, B, C, T, ADC>
where
    B: crate::io::Buttons,
{
    pub storage: &'a S,
    pub display: &'a mut D,
    pub audio: &'a mut A,
    pub buttons: &'a mut ButtonPoller<B>,
    pub clock: &'a C,
    pub time: &'a mut T,
    pub battery_adc: &'a mut ADC,
    pub index: &'a mut IndexStore,
    pub tags: &'a mut TagStore,
    pub state: StateMachine,
    pub activity: ActivityTimer,
    pub sounds: SoundsPolicy,
    pub menu_index: usize,
    pub settings_index: usize,
    pub tag_index: usize,
    pub list_filter: String,
    pub detail_num: i32,
    pub detail_page: usize,
    pub transfer_ip: String,
    pub error_msg: String,
    pub record: Option<RecordSession>,
    pub last_screen: ScreenId,
}

impl<'a, S, D, A, B, C, T, ADC> App<'a, S, D, A, B, C, T, ADC>
where
    S: FileStorage,
    D: Display,
    A: Audio,
    B: crate::io::Buttons,
    C: Clock,
    T: TimeSource,
    ADC: BatteryAdc,
{
    pub fn new(
        storage: &'a S,
        display: &'a mut D,
        audio: &'a mut A,
        buttons: &'a mut ButtonPoller<B>,
        clock: &'a C,
        time: &'a mut T,
        battery_adc: &'a mut ADC,
        index: &'a mut IndexStore,
        tags: &'a mut TagStore,
    ) -> Self {
        Self {
            storage,
            display,
            audio,
            buttons,
            clock,
            time,
            battery_adc,
            index,
            tags,
            state: StateMachine::new(),
            activity: ActivityTimer::default(),
            sounds: SoundsPolicy::default(),
            menu_index: 0,
            settings_index: 0,
            tag_index: 0,
            list_filter: "All".to_string(),
            detail_num: 1,
            detail_page: 0,
            transfer_ip: String::new(),
            error_msg: String::new(),
            record: None,
            last_screen: ScreenId::Idle,
        }
    }

    pub fn boot(&mut self) -> CoreResult<()> {
        load_index(self.storage, self.index)?;
        load_tags(self.storage, self.tags)?;
        self.activity.reset_activity(self.clock.now_ms());
        self.redraw()?;
        Ok(())
    }

    pub fn tick(&mut self) -> CoreResult<()> {
        let now = self.clock.now_ms();
        self.handle_buttons(now)?;
        self.check_ultra_sleep(now)?;
        self.redraw()?;
        Ok(())
    }

    fn handle_buttons(&mut self, now_ms: u64) -> CoreResult<()> {
        if self.state.state() == AppState::Idle
            && self.buttons.idle_rec_hold_started(now_ms)
        {
            self.start_record(now_ms)?;
            return Ok(());
        }

        let rec = self.buttons.poll_rec(now_ms);
        let pwr = self.buttons.poll_pwr(now_ms);
        if rec != ButtonEvent::None || pwr != ButtonEvent::None {
            self.activity.reset_activity(now_ms);
            self.state.activity_reset = true;
        }

        match self.state.state() {
            AppState::Idle => self.handle_idle(rec, pwr)?,
            AppState::Recording => self.handle_recording(rec, now_ms)?,
            AppState::Saved | AppState::TagSelect => self.handle_tag_select(rec, pwr, now_ms)?,
            AppState::Menu => self.handle_menu(rec, pwr)?,
            AppState::NoteList | AppState::TagBrowser => self.handle_note_list(rec, pwr)?,
            AppState::NoteDetail => self.handle_note_detail(rec, pwr)?,
            AppState::DeleteConfirm => self.handle_delete_confirm(rec, pwr)?,
            AppState::Settings => self.handle_settings(rec, pwr)?,
            AppState::DeviceInfo => self.handle_nav_back(rec, Transition::MenuBack)?,
            AppState::Transfer => self.handle_transfer(rec)?,
            AppState::Error => {
                if rec == ButtonEvent::Single || pwr == ButtonEvent::Single {
                    let _ = self.state.apply(Transition::ErrorDismissed);
                }
            }
        }
        Ok(())
    }

    fn handle_idle(&mut self, rec: ButtonEvent, pwr: ButtonEvent) -> CoreResult<()> {
        if pwr == ButtonEvent::Single {
            self.sounds.play(self.audio, SoundKind::Select);
            let _ = self.state.apply(Transition::PwrSingle);
        }
        let _ = (rec,);
        Ok(())
    }

    fn start_record(&mut self, now_ms: u64) -> CoreResult<()> {
        let num = next_note_number(self.index);
        let mut session = RecordSession::start(num);
        session.begin(self.audio, self.storage, now_ms)?;
        self.record = Some(session);
        self.state.set_last_rec_num(num);
        let _ = self.state.apply(Transition::HoldRec);
        Ok(())
    }

    fn handle_recording(&mut self, rec: ButtonEvent, now_ms: u64) -> CoreResult<()> {
        if let Some(session) = self.record.as_mut() {
            session.pump(self.audio, self.storage)?;
            if rec == ButtonEvent::None {
                return Ok(());
            }
            let outcome = session.stop(self.audio, self.storage, now_ms);
            self.record = None;
            match outcome {
                RecordOutcome::Success { num, .. } => {
                    self.sounds.play(self.audio, SoundKind::Saved);
                    self.detail_num = num;
                    let _ = self.state.apply(Transition::ReleaseRec);
                    let _ = self.state.apply(Transition::RecordSuccess);
                }
                RecordOutcome::TooShort => {
                    self.error_msg = "REC FAIL".into();
                    let _ = self.state.apply(Transition::RecordFail);
                }
                RecordOutcome::WriteFailed(msg) => {
                    self.error_msg = msg;
                    let _ = self.state.apply(Transition::RecordFail);
                }
            }
        }
        Ok(())
    }

    fn handle_tag_select(
        &mut self,
        rec: ButtonEvent,
        pwr: ButtonEvent,
        now_ms: u64,
    ) -> CoreResult<()> {
        if self.state.state() == AppState::Saved {
            let _ = self.state.apply(Transition::RecordSuccess);
        }
        if pwr == ButtonEvent::Single {
            self.sounds.play(self.audio, SoundKind::Next);
            if !self.tags.tags().is_empty() {
                self.tag_index = (self.tag_index + 1) % self.tags.tags().len();
            }
        }
        if rec == ButtonEvent::Single || rec == ButtonEvent::Long {
            self.save_current_tag(now_ms)?;
        }
        Ok(())
    }

    fn save_current_tag(&mut self, _now_ms: u64) -> CoreResult<()> {
        let num = self.state.last_rec_num;
        let tag = self
            .tags
            .tags()
            .get(self.tag_index)
            .cloned()
            .unwrap_or_else(|| "Note".to_string());
        let utc = self
            .time
            .utc_iso()
            .unwrap_or_else(|| "2026-01-01T00:00:00Z".to_string());
        finalize_new_note(self.storage, self.index, num, &tag, &utc)?;
        self.sounds.play(self.audio, SoundKind::Success);
        let _ = self.state.apply(Transition::TagSaved);
        self.activity.reset_activity(self.clock.now_ms());
        Ok(())
    }

    fn handle_menu(&mut self, rec: ButtonEvent, pwr: ButtonEvent) -> CoreResult<()> {
        if pwr == ButtonEvent::Single {
            self.sounds.play(self.audio, SoundKind::Next);
            self.menu_index = (self.menu_index + 1) % 4;
        }
        if rec == ButtonEvent::Single {
            self.sounds.play(self.audio, SoundKind::Select);
            match self.menu_index {
                0 => {
                    self.list_filter = "All".into();
                    let _ = self.state.apply(Transition::OpenNotes);
                }
                1 => {
                    let _ = self.state.apply(Transition::OpenTags);
                }
                2 => {
                    let _ = self.state.apply(Transition::OpenSync);
                }
                _ => {
                    let _ = self.state.apply(Transition::OpenSettings);
                }
            }
        }
        if rec == ButtonEvent::Long || rec == ButtonEvent::Double {
            self.handle_nav_back(rec, Transition::MenuBack)?;
        }
        Ok(())
    }

    fn handle_note_list(&mut self, rec: ButtonEvent, pwr: ButtonEvent) -> CoreResult<()> {
        if pwr == ButtonEvent::Single {
            self.sounds.play(self.audio, SoundKind::Next);
        }
        if rec == ButtonEvent::Single {
            self.sounds.play(self.audio, SoundKind::Select);
            let _ = self.state.apply(Transition::RecSingle);
        }
        if rec == ButtonEvent::Long || rec == ButtonEvent::Double {
            self.handle_nav_back(rec, Transition::MenuBack)?;
        }
        Ok(())
    }

    fn handle_note_detail(&mut self, rec: ButtonEvent, pwr: ButtonEvent) -> CoreResult<()> {
        if rec == ButtonEvent::Single {
            self.sounds.play(self.audio, SoundKind::Select);
            let path = note_path(self.detail_num, "wav");
            let _ = self.audio.start_playback(&path);
        }
        if rec == ButtonEvent::Long {
            let _ = self.state.apply(Transition::RecLong);
        }
        if rec == ButtonEvent::Double {
            self.handle_nav_back(rec, Transition::MenuBack)?;
        }
        if pwr == ButtonEvent::Single {
            self.detail_page += 1;
        }
        if self.audio.is_playing() && rec == ButtonEvent::Single {
            self.audio.stop_playback();
        }
        let _ = pwr;
        Ok(())
    }

    fn handle_delete_confirm(&mut self, rec: ButtonEvent, pwr: ButtonEvent) -> CoreResult<()> {
        if rec == ButtonEvent::Single {
            self.sounds.play(self.audio, SoundKind::Delete);
            delete_note(self.storage, self.index, self.detail_num)?;
            let _ = self.state.apply(Transition::RecSingle);
        }
        if rec == ButtonEvent::Long || rec == ButtonEvent::Double || pwr == ButtonEvent::Single {
            let _ = self.state.apply(Transition::DeleteCancelled);
        }
        Ok(())
    }

    fn handle_settings(&mut self, rec: ButtonEvent, pwr: ButtonEvent) -> CoreResult<()> {
        if pwr == ButtonEvent::Single {
            self.sounds.play(self.audio, SoundKind::Next);
            self.settings_index = (self.settings_index + 1) % 3;
        }
        if rec == ButtonEvent::Single {
            match self.settings_index {
                0 => {
                    self.sounds.enabled = !self.sounds.enabled;
                    self.sounds.set_enabled(self.sounds.enabled);
                }
                1 => {
                    self.transfer_ip = "192.168.0.42".into();
                    let _ = self.state.apply(Transition::OpenTransfer);
                }
                2 => {
                    let _ = self.state.apply(Transition::OpenDeviceInfo);
                }
                _ => {}
            }
        }
        if rec == ButtonEvent::Long || rec == ButtonEvent::Double {
            self.handle_nav_back(rec, Transition::MenuBack)?;
        }
        Ok(())
    }

    fn handle_transfer(&mut self, rec: ButtonEvent) -> CoreResult<()> {
        if rec == ButtonEvent::Long || rec == ButtonEvent::Double {
            self.transfer_ip.clear();
            let _ = self.state.apply(Transition::ExitTransfer);
        }
        Ok(())
    }

    fn handle_nav_back(&mut self, rec: ButtonEvent, ev: Transition) -> CoreResult<()> {
        if rec == ButtonEvent::Long || rec == ButtonEvent::Double {
            self.sounds.play(self.audio, SoundKind::Back);
            let _ = self.state.apply(ev);
        }
        Ok(())
    }

    fn check_ultra_sleep(&mut self, now_ms: u64) -> CoreResult<()> {
        if self.activity.should_ultra_sleep(self.state.state(), now_ms) {
            self.last_screen = ScreenId::UltraSleep;
        }
        Ok(())
    }

    pub fn battery_percent(&mut self) -> Option<u8> {
        let samples = self.battery_adc.read_mv_samples(16).ok()?;
        battery_percent_from_adc_samples(&samples)
    }

    pub fn redraw(&mut self) -> CoreResult<()> {
        let pct = self.battery_percent();
        if let Some(p) = pct {
            self.activity.update_battery_warning(
                Some(p),
                BatteryCurve::LOW_THRESHOLD,
                BatteryCurve::RECOVER_THRESHOLD,
                self.clock.now_ms(),
            );
        }

        let state = self.state.state();
        let warn = self.activity.battery_warning_active(self.clock.now_ms());
        let error_msg = self.error_msg.clone();

        let detail_tag = self
            .index
            .find(self.detail_num)
            .map(|e| e.tag.clone())
            .unwrap_or_default();

        let buf = self.display.framebuffer_mut();
        let mut ui = UiContext {
            buf,
            battery_pct: pct,
            firmware_version: FIRMWARE_VERSION,
            note_count: self.index.len(),
            menu_index: self.menu_index,
            settings_index: self.settings_index,
            tag_index: self.tag_index,
            tags: self.tags.tags(),
            list_filter: &self.list_filter,
            list_scroll: 0,
            detail_num: self.detail_num,
            detail_tag: &detail_tag,
            detail_lines: &[],
            detail_page: self.detail_page,
            error_msg: &error_msg,
            device_rtc: "time not set",
            transfer_ip: &self.transfer_ip,
            transcribe_done: 0,
            transcribe_pending: 0,
            sounds_on: self.sounds.enabled,
        };

        self.last_screen = if warn {
            ui.show_battery_low()
        } else if state == AppState::Error {
            ui.show_error_screen(&error_msg)
        } else {
            ui.render(state)
        };
        self.display.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::{MockAudio, MockBatteryAdc, MockButtons, MockClock, MockDisplay, MockTime};
    use crate::storage::index::{add_to_index, IndexStore};

    fn test_app() -> (
        MockStorage,
        MockDisplay,
        MockAudio,
        ButtonPoller<MockButtons>,
        MockClock,
        MockTime,
        MockBatteryAdc,
        IndexStore,
        TagStore,
    ) {
        (
            MockStorage::new().expect("s"),
            MockDisplay::new(),
            MockAudio::new(),
            ButtonPoller::new(MockButtons::default()),
            MockClock::default(),
            MockTime::default(),
            MockBatteryAdc::with_voltage(4.1),
            IndexStore::new(),
            TagStore::new(),
        )
    }

    use crate::storage::MockStorage;

    #[test]
    fn boot_loads_stores_and_renders_idle() {
        let (storage, mut display, mut audio, mut buttons, clock, mut time, mut adc, mut index, mut tags) =
            test_app();
        let mut app = App::new(
            &storage, &mut display, &mut audio, &mut buttons, &clock, &mut time, &mut adc,
            &mut index, &mut tags,
        );
        app.boot().expect("boot");
        assert_eq!(app.state.state(), AppState::Idle);
        assert_eq!(app.last_screen, ScreenId::Idle);
    }

    #[test]
    fn pwr_from_idle_opens_menu() {
        let (storage, mut display, mut audio, mut buttons, mut clock, mut time, mut adc, mut index, mut tags) =
            test_app();
        let mut app = App::new(
            &storage, &mut display, &mut audio, &mut buttons, &clock, &mut time, &mut adc,
            &mut index, &mut tags,
        );
        app.boot().expect("boot");
        buttons.pins_mut().pwr = true;
        clock.now_ms = 10;
        app.tick().expect("tick");
        buttons.pins_mut().pwr = false;
        clock.now_ms = 20;
        app.tick().expect("tick");
        assert_eq!(app.state.state(), AppState::Menu);
    }
}
