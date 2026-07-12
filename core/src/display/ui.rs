//! UPI payment screens for 200×200 e-Paper — calm, QR-first collect flow.
//!
//! Maps existing `AppState` names to payment UX (hardware buttons unchanged):
//! Idle → home · Recording → show QR · Saved → waiting · TagSelect → success · …

use crate::display::draw::{
    draw_battery_ring, draw_check, draw_header, draw_hints, draw_select_row, draw_soft_header,
    draw_str, draw_str_centered, fill_circle, stroke_circle, text_width, BLACK, HEIGHT, WIDTH,
};
use crate::display::qr::draw_qr_centered;
use crate::state::AppState;
use crate::upi::format_amount_label;

pub const HEADER_H: i32 = 28;
pub const HINTS_Y: i32 = 180;
pub const MARGIN: i32 = 12;
pub const CONTENT_TOP: i32 = 36;

pub fn clear_screen(buf: &mut [u8]) {
    buf.fill(0xFF);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenId {
    Idle,
    /// Show UPI QR for customer to scan
    ShowQr,
    /// Awaiting bank confirmation
    Waiting,
    /// Payment received
    Success,
    Menu,
    Settings,
    DeviceInfo,
    HistoryList,
    HistoryDetail,
    CancelConfirm,
    Merchant,
    Error,
    BatteryLow,
    UltraSleep,
    WifiConnecting,
    Syncing,
    /// Legacy aliases kept for call sites
    Recording,
    Saved,
    TagSelect,
    NoteList,
    NoteDetail,
    DeleteConfirm,
    Transfer,
    Transcribing,
    WifiConnectingAlias,
}

pub struct UiContext<'a> {
    pub buf: &'a mut [u8],
    pub battery_pct: Option<u8>,
    pub firmware_version: &'a str,
    /// Merchant / payee display name
    pub merchant_name: &'a str,
    /// VPA e.g. akash@oksbi
    pub upi_vpa: &'a str,
    /// Amount string e.g. "250.00" (empty = any)
    pub amount_inr: &'a str,
    /// Full UPI URI for QR (pre-built)
    pub upi_uri: &'a str,
    /// Last / current txn id or order note
    pub txn_note: &'a str,
    pub txn_count: usize,
    pub menu_index: usize,
    pub settings_index: usize,
    pub history_index: usize,
    pub history_lines: &'a [String],
    pub error_msg: &'a str,
    pub device_rtc: &'a str,
    pub sounds_on: bool,
    pub sync_done: usize,
    pub sync_pending: usize,
    // Kept so older call sites compile during transition
    pub note_count: usize,
    pub tag_index: usize,
    pub tags: &'a [String],
    pub list_filter: &'a str,
    pub list_scroll: usize,
    pub detail_num: i32,
    pub detail_tag: &'a str,
    pub detail_lines: &'a [String],
    pub detail_page: usize,
    pub transfer_ip: &'a str,
    pub transcribe_done: usize,
    pub transcribe_pending: usize,
}

impl UiContext<'_> {
    pub fn render(&mut self, state: AppState) -> ScreenId {
        clear_screen(self.buf);
        match state {
            AppState::Idle => self.show_home(),
            AppState::Recording => self.show_qr(),
            AppState::Saved => self.show_waiting(),
            AppState::TagSelect => self.show_success(),
            AppState::Menu => self.show_menu(),
            AppState::Settings => self.show_settings(),
            AppState::DeviceInfo => self.show_device_info(),
            AppState::NoteList | AppState::TagBrowser => self.show_history(),
            AppState::NoteDetail => self.show_history_detail(),
            AppState::DeleteConfirm => self.show_cancel_confirm(),
            AppState::Transfer => self.show_merchant(),
            AppState::Error => self.show_error_screen(self.error_msg),
        }
    }

    fn amount_label(&self) -> String {
        format_amount_label(self.amount_inr)
    }

    /// Home — ready to collect
    fn show_home(&mut self) -> ScreenId {
        let cx = (WIDTH / 2) as i32;
        if let Some(pct) = self.battery_pct {
            draw_battery_ring(self.buf, WIDTH as i32 - 28, 28, pct);
            let label = format!("{pct}%");
            let lw = text_width(&label, 1);
            draw_str(self.buf, WIDTH as i32 - 28 - lw / 2, 46, &label, 1, BLACK);
        }
        draw_str_centered(self.buf, cx, 58, "ZOOP PAY", 1, BLACK);
        draw_str_centered(self.buf, cx, 78, self.merchant_name, 1, BLACK);
        draw_str_centered(self.buf, cx, 108, &self.amount_label(), 2, BLACK);
        draw_str_centered(self.buf, cx, 138, "hold REC for QR", 1, BLACK);
        let n = if self.txn_count == 1 {
            "1 payment".to_string()
        } else {
            format!("{} payments", self.txn_count)
        };
        draw_str_centered(self.buf, cx, 156, &n, 1, BLACK);
        draw_hints(self.buf, "Show QR", "Menu");
        ScreenId::Idle
    }

    /// Customer scans this UPI QR
    fn show_qr(&mut self) -> ScreenId {
        draw_soft_header(self.buf, "SCAN UPI", Some(&self.amount_label()));
        let uri = if self.upi_uri.is_empty() {
            "upi://pay?pa=demo@upi&pn=Zoop&cu=INR"
        } else {
            self.upi_uri
        };
        // QR sits under soft header; leave strip for amount already in header
        if draw_qr_centered(self.buf, uri, 30).is_err() {
            draw_str_centered(self.buf, (WIDTH / 2) as i32, 100, "QR too long", 1, BLACK);
        }
        draw_hints(self.buf, "Wait", "Cancel");
        ScreenId::ShowQr
    }

    fn show_waiting(&mut self) -> ScreenId {
        draw_soft_header(self.buf, "WAITING", None);
        let cx = (WIDTH / 2) as i32;
        stroke_circle(self.buf, cx, 88, 28, 2, BLACK);
        fill_circle(self.buf, cx, 88, 6, BLACK);
        draw_str_centered(self.buf, cx, 128, "checking payment", 1, BLACK);
        draw_str_centered(self.buf, cx, 148, &self.amount_label(), 1, BLACK);
        draw_hints(self.buf, "", "Cancel");
        ScreenId::Waiting
    }

    fn show_success(&mut self) -> ScreenId {
        draw_soft_header(self.buf, "PAID", None);
        let cx = (WIDTH / 2) as i32;
        draw_check(self.buf, cx, 88);
        draw_str_centered(self.buf, cx, 132, &self.amount_label(), 1, BLACK);
        if !self.txn_note.is_empty() {
            draw_str_centered(self.buf, cx, 150, self.txn_note, 1, BLACK);
        }
        draw_hints(self.buf, "Done", "");
        ScreenId::Success
    }

    fn show_menu(&mut self) -> ScreenId {
        const ITEMS: [&str; 4] = ["Collect", "History", "Merchant", "Settings"];
        draw_soft_header(self.buf, "MENU", None);
        let y0 = CONTENT_TOP + 4;
        for (i, item) in ITEMS.iter().enumerate() {
            draw_select_row(self.buf, y0 + i as i32 * 30, item, i == self.menu_index);
        }
        draw_hints(self.buf, "Open", "Next");
        ScreenId::Menu
    }

    fn show_settings(&mut self) -> ScreenId {
        let sounds = if self.sounds_on { "on" } else { "off" };
        let items = [
            format!("Sounds  {sounds}"),
            "Device".to_string(),
            "About".to_string(),
        ];
        draw_soft_header(self.buf, "SETTINGS", None);
        let y0 = CONTENT_TOP + 8;
        for (i, item) in items.iter().enumerate() {
            draw_select_row(self.buf, y0 + i as i32 * 32, item, i == self.settings_index);
        }
        draw_hints(self.buf, "Select", "Next");
        ScreenId::Settings
    }

    fn show_device_info(&mut self) -> ScreenId {
        draw_soft_header(self.buf, "DEVICE", None);
        let lines = [
            format!("FW {}", self.firmware_version),
            self.battery_pct
                .map(|p| format!("Battery  {p}%"))
                .unwrap_or_else(|| "Battery  --".to_string()),
            format!("Clock  {}", self.device_rtc),
            format!("Txns  {}", self.txn_count),
        ];
        let mut y = CONTENT_TOP + 8;
        for line in &lines {
            draw_str(self.buf, MARGIN + 4, y, line, 1, BLACK);
            y += 26;
        }
        draw_hints(self.buf, "Back", "");
        ScreenId::DeviceInfo
    }

    fn show_history(&mut self) -> ScreenId {
        draw_soft_header(self.buf, "HISTORY", None);
        let cx = (WIDTH / 2) as i32;
        if self.history_lines.is_empty() {
            draw_str_centered(self.buf, cx, 90, "no payments yet", 1, BLACK);
        } else {
            let mut y = CONTENT_TOP + 4;
            let start = self.history_index.min(self.history_lines.len().saturating_sub(1));
            for line in self.history_lines.iter().skip(start).take(5) {
                draw_str(self.buf, MARGIN, y, line, 1, BLACK);
                y += 22;
            }
        }
        draw_hints(self.buf, "Open", "Next");
        ScreenId::HistoryList
    }

    fn show_history_detail(&mut self) -> ScreenId {
        draw_soft_header(self.buf, "TXN", Some(&self.amount_label()));
        let mut y = CONTENT_TOP + 4;
        let start = self.detail_page * 6;
        let lines = if self.history_lines.is_empty() {
            self.detail_lines
        } else {
            self.history_lines
        };
        for line in lines.iter().skip(start).take(6) {
            draw_str(self.buf, MARGIN, y, line, 1, BLACK);
            y += 20;
        }
        draw_hints(self.buf, "Back", "Scroll");
        ScreenId::HistoryDetail
    }

    fn show_cancel_confirm(&mut self) -> ScreenId {
        draw_soft_header(self.buf, "CANCEL", None);
        let cx = (WIDTH / 2) as i32;
        draw_str_centered(self.buf, cx, 80, "stop this QR?", 1, BLACK);
        draw_str_centered(self.buf, cx, 108, &self.amount_label(), 1, BLACK);
        draw_hints(self.buf, "Yes", "Back");
        ScreenId::CancelConfirm
    }

    fn show_merchant(&mut self) -> ScreenId {
        draw_soft_header(self.buf, "MERCHANT", None);
        let cx = (WIDTH / 2) as i32;
        draw_str_centered(self.buf, cx, 56, self.merchant_name, 1, BLACK);
        draw_str_centered(self.buf, cx, 84, self.upi_vpa, 1, BLACK);
        draw_str_centered(self.buf, cx, 120, "static VPA", 1, BLACK);
        draw_str_centered(self.buf, cx, 142, "use Collect for QR", 1, BLACK);
        draw_hints(self.buf, "Back", "");
        ScreenId::Merchant
    }

    pub fn show_error_screen(&mut self, msg: &str) -> ScreenId {
        clear_screen(self.buf);
        draw_header(self.buf, "ERROR", None);
        let cx = (WIDTH / 2) as i32;
        draw_str_centered(self.buf, cx, 88, msg, 1, BLACK);
        draw_str_centered(self.buf, cx, 120, "try again", 1, BLACK);
        draw_hints(self.buf, "OK", "");
        ScreenId::Error
    }

    pub fn show_battery_low(&mut self) -> ScreenId {
        clear_screen(self.buf);
        draw_soft_header(self.buf, "BATTERY", None);
        let cx = (WIDTH / 2) as i32;
        stroke_circle(self.buf, cx, 90, 26, 2, BLACK);
        draw_str_centered(self.buf, cx, 86, "!", 2, BLACK);
        draw_str_centered(self.buf, cx, 130, "charge soon", 1, BLACK);
        ScreenId::BatteryLow
    }

    pub fn show_ultra_sleep(&mut self) -> ScreenId {
        clear_screen(self.buf);
        let cx = (WIDTH / 2) as i32;
        let cy = (HEIGHT / 2) as i32 - 8;
        stroke_circle(self.buf, cx, cy, 20, 1, BLACK);
        draw_str_centered(self.buf, cx, cy + 36, "resting", 1, BLACK);
        ScreenId::UltraSleep
    }

    pub fn show_wifi_connecting(&mut self, attempt: u32, max: u32) -> ScreenId {
        clear_screen(self.buf);
        draw_soft_header(self.buf, "WIFI", None);
        let cx = (WIDTH / 2) as i32;
        draw_str_centered(self.buf, cx, 80, "connecting", 1, BLACK);
        draw_str_centered(self.buf, cx, 110, &format!("{attempt} / {max}"), 1, BLACK);
        ScreenId::WifiConnecting
    }

    pub fn show_transcribing(&mut self) -> ScreenId {
        // Reuse as payment sync progress
        clear_screen(self.buf);
        draw_soft_header(self.buf, "SYNC", None);
        let cx = (WIDTH / 2) as i32;
        let line = format!("{}/{}", self.sync_done, self.sync_pending);
        draw_str_centered(self.buf, cx, 88, "confirming", 1, BLACK);
        draw_str_centered(self.buf, cx, 118, &line, 2, BLACK);
        ScreenId::Syncing
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::display::draw::{get_pixel, BYTES};
    use crate::state::AppState;
    use crate::upi::build_upi_uri;

    fn ctx<'a>(
        buf: &'a mut [u8],
        uri: &'a str,
        history: &'a [String],
        tags: &'a [String],
    ) -> UiContext<'a> {
        UiContext {
            buf,
            battery_pct: Some(80),
            firmware_version: "v1.0",
            merchant_name: "Akash Soni",
            upi_vpa: "akash@oksbi",
            amount_inr: "100.00",
            upi_uri: uri,
            txn_note: "order 12",
            txn_count: 3,
            menu_index: 0,
            settings_index: 0,
            history_index: 0,
            history_lines: history,
            error_msg: "PAY FAIL",
            device_rtc: "set",
            sounds_on: true,
            sync_done: 1,
            sync_pending: 2,
            note_count: 3,
            tag_index: 0,
            tags,
            list_filter: "All",
            list_scroll: 0,
            detail_num: 1,
            detail_tag: "",
            detail_lines: &[],
            detail_page: 0,
            transfer_ip: "",
            transcribe_done: 0,
            transcribe_pending: 0,
        }
    }

    #[test]
    fn home_renders_idle() {
        let mut buf = vec![0xFF; BYTES];
        let uri = build_upi_uri("akash@oksbi", "Akash Soni", "100.00", "Zoop");
        let history = vec![];
        let tags = vec![];
        let mut ui = ctx(&mut buf, &uri, &history, &tags);
        assert_eq!(ui.render(AppState::Idle), ScreenId::Idle);
    }

    #[test]
    fn qr_screen_paints_modules() {
        let mut buf = vec![0xFF; BYTES];
        let uri = build_upi_uri("akash@oksbi", "Akash Soni", "100.00", "Zoop");
        let history = vec![];
        let tags = vec![];
        let mut ui = ctx(&mut buf, &uri, &history, &tags);
        assert_eq!(ui.render(AppState::Recording), ScreenId::ShowQr);
        let mut dark = 0;
        for y in 30..170 {
            for x in 30..170 {
                if get_pixel(&buf, x, y) == BLACK {
                    dark += 1;
                }
            }
        }
        assert!(dark > 200);
    }

    #[test]
    fn menu_outline_selects_row() {
        let mut buf = vec![0xFF; BYTES];
        let uri = "";
        let history = vec![];
        let tags = vec![];
        let mut ui = ctx(&mut buf, uri, &history, &tags);
        ui.menu_index = 1;
        assert_eq!(ui.render(AppState::Menu), ScreenId::Menu);
        assert_eq!(get_pixel(&buf, 12, 75), BLACK);
    }
}
