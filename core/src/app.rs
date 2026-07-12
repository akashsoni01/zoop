//! Application engine — UPI payment state machine + BSP traits.

use crate::battery::{battery_percent_from_adc_samples, BatteryCurve};
use crate::buttons::ButtonPoller;
use crate::display::ui::{ScreenId, UiContext};
use crate::error::CoreResult;
use crate::io::{Audio, BatteryAdc, ButtonEvents, Clock, Display, SoundKind, TimeSource};
use crate::payment::{MerchantProfile, PaymentLedger, PaymentRequest, PriceCatalog};
use crate::sleep::ActivityTimer;
use crate::sounds::SoundsPolicy;
use crate::state::{AppState, ButtonEvent, StateMachine, Transition};
use crate::storage::FileStorage;

pub const FIRMWARE_VERSION: &str = "v1.0";

/// High-level app context wiring core logic to BSP traits.
pub struct App<'a, S, D, A, T, ADC> {
    pub storage: &'a S,
    pub display: &'a mut D,
    pub audio: &'a mut A,
    pub time: &'a mut T,
    pub battery_adc: &'a mut ADC,
    pub state: StateMachine,
    pub activity: ActivityTimer,
    pub sounds: SoundsPolicy,
    pub menu_index: usize,
    pub settings_index: usize,
    pub history_index: usize,
    pub price_index: usize,
    pub detail_txn: i32,
    pub detail_page: usize,
    pub error_msg: String,
    pub merchant: MerchantProfile,
    pub prices: PriceCatalog,
    pub ledger: PaymentLedger,
    pub active: Option<PaymentRequest>,
    /// Default collect amount on home QR (`""` = any amount).
    pub default_amount_inr: String,
    pub waiting_since_ms: Option<u64>,
    pub last_screen: ScreenId,
}

impl<'a, S, D, A, T, ADC> App<'a, S, D, A, T, ADC>
where
    S: FileStorage,
    D: Display,
    A: Audio,
    T: TimeSource,
    ADC: BatteryAdc,
{
    pub fn new(
        storage: &'a S,
        display: &'a mut D,
        audio: &'a mut A,
        time: &'a mut T,
        battery_adc: &'a mut ADC,
    ) -> Self {
        Self {
            storage,
            display,
            audio,
            time,
            battery_adc,
            state: StateMachine::new(),
            activity: ActivityTimer::default(),
            sounds: SoundsPolicy::default(),
            menu_index: 0,
            settings_index: 0,
            history_index: 0,
            price_index: 0,
            detail_txn: -1,
            detail_page: 0,
            error_msg: String::new(),
            merchant: MerchantProfile::default(),
            prices: PriceCatalog::from_defaults(),
            ledger: PaymentLedger::new(),
            active: None,
            default_amount_inr: String::new(),
            waiting_since_ms: None,
            last_screen: ScreenId::Idle,
        }
    }

    pub fn boot<C: Clock>(&mut self, clock: &C) -> CoreResult<()> {
        self.activity.reset_activity(clock.now_ms());
        self.redraw(clock)?;
        Ok(())
    }

    pub fn tick<B: crate::io::Buttons, C: Clock>(
        &mut self,
        buttons: &mut ButtonPoller<B>,
        clock: &C,
    ) -> CoreResult<()> {
        let now = clock.now_ms();
        self.handle_buttons(buttons, now)?;
        self.advance_payment(now)?;
        self.check_ultra_sleep(now)?;
        self.redraw(clock)?;
        Ok(())
    }

    fn handle_buttons<B: crate::io::Buttons>(
        &mut self,
        buttons: &mut ButtonPoller<B>,
        now_ms: u64,
    ) -> CoreResult<()> {
        let rec = buttons.poll_rec(now_ms);
        let pwr = buttons.poll_pwr(now_ms);
        if rec != ButtonEvent::None || pwr != ButtonEvent::None {
            self.activity.reset_activity(now_ms);
            self.state.activity_reset = true;
        }

        match self.state.state() {
            AppState::Idle => self.handle_idle(rec, pwr)?,
            AppState::PricePick => self.handle_price_pick(rec, pwr, now_ms)?,
            AppState::ShowQr => self.handle_show_qr(rec, pwr, now_ms)?,
            AppState::Waiting => self.handle_waiting(rec)?,
            AppState::Success => {
                if rec == ButtonEvent::Single {
                    self.finish_success(now_ms)?;
                }
            }
            AppState::Menu => self.handle_menu(rec, pwr)?,
            AppState::History => self.handle_history(rec, pwr)?,
            AppState::HistoryDetail => self.handle_history_detail(rec, pwr)?,
            AppState::CancelConfirm => self.handle_cancel(rec)?,
            AppState::Settings => self.handle_settings(rec, pwr)?,
            AppState::DeviceInfo => self.handle_nav_back(rec, Transition::MenuBack)?,
            AppState::Merchant => self.handle_nav_back(rec, Transition::ExitMerchant)?,
            AppState::Error => {
                if rec == ButtonEvent::Single || pwr == ButtonEvent::Single {
                    let _ = self.state.apply(Transition::ErrorDismissed);
                    self.active = None;
                    self.error_msg.clear();
                }
            }
        }
        Ok(())
    }

    fn handle_idle(&mut self, rec: ButtonEvent, pwr: ButtonEvent) -> CoreResult<()> {
        if rec == ButtonEvent::Single || rec == ButtonEvent::Long || rec == ButtonEvent::Double {
            self.sounds.play(self.audio, SoundKind::Select);
            self.history_index = 0;
            let _ = self.state.apply(Transition::OpenHistory);
        }
        if pwr == ButtonEvent::Single {
            self.sounds.play(self.audio, SoundKind::Select);
            let _ = self.state.apply(Transition::PwrSingle);
        }
        Ok(())
    }

    fn handle_price_pick(
        &mut self,
        rec: ButtonEvent,
        pwr: ButtonEvent,
        now_ms: u64,
    ) -> CoreResult<()> {
        let n = self.prices.len().max(1);
        if pwr == ButtonEvent::Single {
            self.sounds.play(self.audio, SoundKind::Next);
            self.price_index = (self.price_index + 1) % n;
        }
        if rec == ButtonEvent::Single {
            self.start_priced_collect(now_ms)?;
        }
        if rec == ButtonEvent::Long || rec == ButtonEvent::Double {
            self.active = None;
            self.handle_nav_back(rec, Transition::MenuBack)?;
        }
        Ok(())
    }

    pub(crate) fn start_priced_collect(&mut self, now_ms: u64) -> CoreResult<()> {
        let amount = self
            .prices
            .get(self.price_index)
            .unwrap_or("100.00")
            .to_string();
        let note = format!("Zoop {}", self.ledger.next_num());
        let mut req = PaymentRequest::new(&self.merchant, &amount, &note);
        req.mark_qr_shown();
        self.state.set_last_txn_num(self.ledger.next_num());
        self.active = Some(req);
        self.waiting_since_ms = None;
        self.sounds.play(self.audio, SoundKind::Select);
        let _ = self.state.apply(Transition::SelectPrice);
        self.activity.reset_activity(now_ms);
        Ok(())
    }

    fn handle_show_qr(
        &mut self,
        rec: ButtonEvent,
        pwr: ButtonEvent,
        now_ms: u64,
    ) -> CoreResult<()> {
        if rec == ButtonEvent::Single {
            // Merchant confirms customer paid / advance to waiting
            if let Some(req) = self.active.as_mut() {
                req.mark_pending();
            }
            self.waiting_since_ms = Some(now_ms);
            self.sounds.play(self.audio, SoundKind::Select);
            let _ = self.state.apply(Transition::PaymentPending);
            self.activity.reset_activity(now_ms);
        }
        if rec == ButtonEvent::Long || rec == ButtonEvent::Double {
            let _ = self.state.apply(Transition::RecLong);
        }
        if pwr == ButtonEvent::Single {
            self.active = None;
            let _ = self.state.apply(Transition::MenuBack);
        }
        Ok(())
    }

    fn handle_waiting(&mut self, rec: ButtonEvent) -> CoreResult<()> {
        if rec == ButtonEvent::Long || rec == ButtonEvent::Double {
            let _ = self.state.apply(Transition::RecLong);
        }
        Ok(())
    }

    fn advance_payment(&mut self, now_ms: u64) -> CoreResult<()> {
        match self.state.state() {
            AppState::Waiting => {
                let since = self.waiting_since_ms.unwrap_or(now_ms);
                if now_ms.saturating_sub(since) >= 1_500 {
                    self.complete_payment(now_ms)?;
                }
            }
            AppState::Success => {
                if self.activity.idle_ms(now_ms) >= 2_500 {
                    self.finish_success(now_ms)?;
                }
            }
            AppState::Error => {
                if self.activity.idle_ms(now_ms) >= 2_500 {
                    let _ = self.state.apply(Transition::ErrorDismissed);
                    self.active = None;
                    self.error_msg.clear();
                    self.activity.reset_activity(now_ms);
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn complete_payment(&mut self, now_ms: u64) -> CoreResult<()> {
        if let Some(mut req) = self.active.take() {
            req.mark_paid();
            let rec = self.ledger.push_paid(&req.amount_inr, &req.note);
            self.detail_txn = rec.num;
            self.active = Some(req);
        }
        self.waiting_since_ms = None;
        self.sounds.play(self.audio, SoundKind::Success);
        let _ = self.state.apply(Transition::PaymentSuccess);
        self.activity.reset_activity(now_ms);
        Ok(())
    }

    fn finish_success(&mut self, now_ms: u64) -> CoreResult<()> {
        let _ = self.state.apply(Transition::PaymentDone);
        self.active = None;
        self.activity.reset_activity(now_ms);
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
                    self.price_index = 0;
                    let _ = self.state.apply(Transition::OpenPrices);
                }
                1 => {
                    self.history_index = 0;
                    let _ = self.state.apply(Transition::OpenHistory);
                }
                2 => {
                    let _ = self.state.apply(Transition::OpenMerchant);
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

    fn handle_history(&mut self, rec: ButtonEvent, pwr: ButtonEvent) -> CoreResult<()> {
        if pwr == ButtonEvent::Single {
            self.sounds.play(self.audio, SoundKind::Next);
            let n = self.ledger.recent_labels(8).len().max(1);
            self.history_index = (self.history_index + 1) % n;
        }
        if rec == ButtonEvent::Single || rec == ButtonEvent::Long || rec == ButtonEvent::Double {
            self.sounds.play(self.audio, SoundKind::Back);
            let _ = self.state.apply(Transition::MenuBack);
        }
        Ok(())
    }

    fn handle_history_detail(&mut self, rec: ButtonEvent, pwr: ButtonEvent) -> CoreResult<()> {
        if rec == ButtonEvent::Long {
            let _ = self.state.apply(Transition::RecLong);
        }
        if rec == ButtonEvent::Double || rec == ButtonEvent::Single {
            self.handle_nav_back(rec, Transition::MenuBack)?;
        }
        if pwr == ButtonEvent::Single {
            self.detail_page = self.detail_page.saturating_add(1);
        }
        Ok(())
    }

    fn handle_cancel(&mut self, rec: ButtonEvent) -> CoreResult<()> {
        if rec == ButtonEvent::Single {
            if let Some(req) = self.active.as_mut() {
                req.mark_cancelled();
            }
            self.active = None;
            self.waiting_since_ms = None;
            self.sounds.play(self.audio, SoundKind::Delete);
            let _ = self.state.apply(Transition::CancelConfirmed);
        }
        if rec == ButtonEvent::Long || rec == ButtonEvent::Double {
            let _ = self.state.apply(Transition::CancelDismissed);
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
                1 | 2 => {
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

    fn handle_nav_back(&mut self, rec: ButtonEvent, ev: Transition) -> CoreResult<()> {
        if rec == ButtonEvent::Long || rec == ButtonEvent::Double || rec == ButtonEvent::Single {
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

    pub fn redraw<C: Clock>(&mut self, clock: &C) -> CoreResult<()> {
        let pct = self.battery_percent();
        let now = clock.now_ms();
        if let Some(p) = pct {
            self.activity.update_battery_warning(
                Some(p),
                BatteryCurve::LOW_THRESHOLD,
                BatteryCurve::RECOVER_THRESHOLD,
                now,
            );
        }

        let state = self.state.state();
        let warn = self.activity.battery_warning_active(now);
        let error_msg = self.error_msg.clone();
        let history = self.ledger.recent_labels(5);
        let history_total = self.ledger.total_paid_label();
        let price_labels = self.prices.labels();
        let amount = self
            .active
            .as_ref()
            .map(|r| r.amount_inr.as_str())
            .unwrap_or(self.default_amount_inr.as_str());
        let upi_uri = self
            .active
            .as_ref()
            .map(|r| r.uri.clone())
            .unwrap_or_else(|| {
                crate::upi::build_upi_uri(
                    &self.merchant.vpa,
                    &self.merchant.name,
                    &self.default_amount_inr,
                    "Zoop Pay",
                )
            });
        let txn_note = self
            .active
            .as_ref()
            .map(|r| r.note.clone())
            .unwrap_or_default();
        let merchant_name = self.merchant.name.clone();
        let merchant_vpa = self.merchant.vpa.clone();
        let txn_count = self.ledger.len();
        let menu_index = self.menu_index;
        let settings_index = self.settings_index;
        let history_index = self.history_index;
        let price_index = self.price_index;
        let detail_page = self.detail_page;
        let sounds_on = self.sounds.enabled;
        let empty: [String; 0] = [];

        let buf = self.display.framebuffer_mut();
        let mut ui = UiContext {
            buf,
            battery_pct: pct,
            firmware_version: FIRMWARE_VERSION,
            merchant_name: &merchant_name,
            upi_vpa: &merchant_vpa,
            amount_inr: amount,
            upi_uri: &upi_uri,
            txn_note: &txn_note,
            txn_count,
            menu_index,
            settings_index,
            history_index,
            history_lines: &history,
            history_total: &history_total,
            price_labels: &price_labels,
            price_index,
            error_msg: &error_msg,
            device_rtc: "time not set",
            sounds_on,
            sync_done: 0,
            sync_pending: 0,
            note_count: txn_count,
            tag_index: 0,
            tags: &empty,
            list_filter: "All",
            list_scroll: 0,
            detail_num: self.detail_txn,
            detail_tag: "",
            detail_lines: &empty,
            detail_page,
            transfer_ip: "",
            transcribe_done: 0,
            transcribe_pending: 0,
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
    use crate::storage::MockStorage;

    fn test_app() -> (
        MockStorage,
        MockDisplay,
        MockAudio,
        ButtonPoller<MockButtons>,
        MockClock,
        MockTime,
        MockBatteryAdc,
    ) {
        (
            MockStorage::new().expect("s"),
            MockDisplay::new(),
            MockAudio::new(),
            ButtonPoller::new(MockButtons::default()),
            MockClock::default(),
            MockTime::default(),
            MockBatteryAdc::with_voltage(4.1),
        )
    }

    #[test]
    fn boot_renders_idle() {
        let (storage, mut display, mut audio, _buttons, clock, mut time, mut adc) = test_app();
        let mut app = App::new(&storage, &mut display, &mut audio, &mut time, &mut adc);
        app.boot(&clock).expect("boot");
        assert_eq!(app.state.state(), AppState::Idle);
        assert_eq!(app.last_screen, ScreenId::Idle);
    }

    #[test]
    fn rec_opens_history_from_home() {
        let (storage, mut display, mut audio, mut buttons, mut clock, mut time, mut adc) =
            test_app();
        let mut app = App::new(&storage, &mut display, &mut audio, &mut time, &mut adc);
        app.ledger.push_paid("100.00", "a");
        app.ledger.push_paid("50.00", "b");
        app.boot(&clock).expect("boot");

        // REC tap (press + release through debounce)
        let mut t = 0u64;
        buttons.pins_mut().rec = true;
        t += 10;
        clock.now_ms = t;
        app.tick(&mut buttons, &clock).expect("tick");
        t += 10;
        clock.now_ms = t;
        app.tick(&mut buttons, &clock).expect("tick");
        buttons.pins_mut().rec = false;
        for _ in 0..15 {
            t += 50;
            clock.now_ms = t;
            app.tick(&mut buttons, &clock).expect("tick");
            if app.state.state() == AppState::History {
                break;
            }
        }
        assert_eq!(app.state.state(), AppState::History);
        assert_eq!(app.ledger.total_paid_label(), "Rs 150");
    }

    #[test]
    fn priced_collect_flow() {
        let (storage, mut display, mut audio, mut buttons, mut clock, mut time, mut adc) =
            test_app();
        let mut app = App::new(&storage, &mut display, &mut audio, &mut time, &mut adc);
        app.boot(&clock).expect("boot");

        // Jump to price pick
        let _ = app.state.apply(Transition::OpenPrices);
        app.price_index = 1; // 100.00
        app.start_priced_collect(0).expect("qr");
        assert_eq!(app.state.state(), AppState::ShowQr);
        assert!(app.active.as_ref().unwrap().uri.contains("am=100.00"));

        let mut t = 0u64;
        buttons.pins_mut().rec = true;
        t += 10;
        clock.now_ms = t;
        app.tick(&mut buttons, &clock).expect("tick");
        buttons.pins_mut().rec = false;
        for _ in 0..20 {
            t += 50;
            clock.now_ms = t;
            app.tick(&mut buttons, &clock).expect("tick");
            if app.state.state() == AppState::Waiting || app.state.state() == AppState::Success {
                break;
            }
        }
        assert!(matches!(
            app.state.state(),
            AppState::Waiting | AppState::Success
        ));
    }
}
