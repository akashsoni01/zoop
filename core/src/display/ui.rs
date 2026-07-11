//! E-Ink screen render functions — ports `ui.cpp` `show*()` (custom immediate-mode).

use crate::display::draw::{
    draw_battery_ring, draw_header, draw_hints, draw_str, draw_str_centered, fill_circle,
    fill_rect, stroke_circle, text_width, BLACK, HEIGHT, WHITE, WIDTH,
};
use crate::state::AppState;

pub const HEADER_H: i32 = 28;
pub const HINTS_Y: i32 = 180;

/// Clear framebuffer to white.
pub fn clear_screen(buf: &mut [u8]) {
    buf.fill(0xFF);
}

/// Which screen was last rendered (for tests).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenId {
    Idle,
    Recording,
    Saved,
    TagSelect,
    Menu,
    Settings,
    DeviceInfo,
    NoteList,
    NoteDetail,
    DeleteConfirm,
    Transfer,
    BatteryLow,
    Error,
    UltraSleep,
    WifiConnecting,
    Transcribing,
}

pub struct UiContext<'a> {
    pub buf: &'a mut [u8],
    pub battery_pct: Option<u8>,
    pub firmware_version: &'a str,
    pub note_count: usize,
    pub menu_index: usize,
    pub settings_index: usize,
    pub tag_index: usize,
    pub tags: &'a [String],
    pub list_filter: &'a str,
    pub list_scroll: usize,
    pub detail_num: i32,
    pub detail_tag: &'a str,
    pub detail_lines: &'a [String],
    pub detail_page: usize,
    pub error_msg: &'a str,
    pub device_rtc: &'a str,
    pub transfer_ip: &'a str,
    pub transcribe_done: usize,
    pub transcribe_pending: usize,
    pub sounds_on: bool,
}

impl UiContext<'_> {
    pub fn render(&mut self, state: AppState) -> ScreenId {
        clear_screen(self.buf);
        match state {
            AppState::Idle => self.show_idle(),
            AppState::Recording => self.show_recording(),
            AppState::Saved => self.show_saved(),
            AppState::TagSelect => self.show_tag_select(),
            AppState::Menu => self.show_menu(),
            AppState::Settings => self.show_settings(),
            AppState::DeviceInfo => self.show_device_info(),
            AppState::NoteList | AppState::TagBrowser => self.show_note_list(),
            AppState::NoteDetail => self.show_note_detail(),
            AppState::DeleteConfirm => self.show_delete_confirm(),
            AppState::Transfer => self.show_transfer(),
            AppState::Error => self.show_error_screen(self.error_msg),
        }
    }

    fn show_idle(&mut self) -> ScreenId {
        if let Some(pct) = self.battery_pct {
            draw_battery_ring(self.buf, WIDTH as i32 - 30, 50, pct);
            let label = format!("{pct}%");
            draw_str(self.buf, WIDTH as i32 - 48, 72, &label, 1, BLACK);
        }
        draw_str_centered(self.buf, (WIDTH / 2) as i32, 90, "ZOOP", 1, BLACK);
        draw_str_centered(
            self.buf,
            (WIDTH / 2) as i32,
            110,
            &format!("{} notes", self.note_count),
            1,
            BLACK,
        );
        fill_circle(self.buf, (WIDTH / 2) as i32, 145, 20, BLACK);
        stroke_circle(self.buf, (WIDTH / 2) as i32, 145, 28, 2, BLACK);
        draw_hints(self.buf, "Hold REC", "Menu");
        ScreenId::Idle
    }

    fn show_recording(&mut self) -> ScreenId {
        draw_header(self.buf, "RECORDING", None);
        fill_circle(self.buf, (WIDTH / 2) as i32, 100, 36, BLACK);
        stroke_circle(self.buf, (WIDTH / 2) as i32, 100, 52, 3, BLACK);
        draw_str_centered(self.buf, (WIDTH / 2) as i32, 155, "Release to stop", 1, BLACK);
        draw_hints(self.buf, "", "");
        ScreenId::Recording
    }

    fn show_saved(&mut self) -> ScreenId {
        draw_header(self.buf, "SAVED", None);
        fill_circle(self.buf, (WIDTH / 2) as i32, 100, 40, BLACK);
        draw_str_centered(self.buf, (WIDTH / 2) as i32, 155, "Pick a tag", 1, BLACK);
        draw_hints(self.buf, "Save", "Next tag");
        ScreenId::Saved
    }

    fn show_tag_select(&mut self) -> ScreenId {
        draw_header(self.buf, "TAG", None);
        let tag = self
            .tags
            .get(self.tag_index)
            .map(|s| s.as_str())
            .unwrap_or("Note");
        draw_str_centered(self.buf, (WIDTH / 2) as i32, 95, tag, 1, BLACK);
        draw_str_centered(self.buf, (WIDTH / 2) as i32, 120, "REC to save", 1, BLACK);
        draw_hints(self.buf, "Save", "Cycle");
        ScreenId::TagSelect
    }

    fn show_menu(&mut self) -> ScreenId {
        const ITEMS: [&str; 4] = ["Notes", "Tags", "Sync", "Settings"];
        draw_header(self.buf, "MENU", None);
        let y0 = 40;
        for (i, item) in ITEMS.iter().enumerate() {
            let y = y0 + i as i32 * 28;
            if i == self.menu_index {
                fill_rect(self.buf, 8, y - 2, (WIDTH - 16) as i32, 22, BLACK);
                draw_str(self.buf, 16, y + 4, item, 1, WHITE);
            } else {
                draw_str(self.buf, 16, y + 4, item, 1, BLACK);
            }
        }
        draw_hints(self.buf, "Select", "Next");
        ScreenId::Menu
    }

    fn show_settings(&mut self) -> ScreenId {
        let sounds = if self.sounds_on { "ON" } else { "OFF" };
        let items = [
            format!("Sounds: {sounds}"),
            "Transfer".to_string(),
            "Device".to_string(),
        ];
        draw_header(self.buf, "SETTINGS", None);
        let y0 = 40;
        for (i, item) in items.iter().enumerate() {
            let y = y0 + i as i32 * 28;
            if i == self.settings_index {
                fill_rect(self.buf, 8, y - 2, (WIDTH - 16) as i32, 22, BLACK);
                draw_str(self.buf, 16, y + 4, item, 1, WHITE);
            } else {
                draw_str(self.buf, 16, y + 4, item, 1, BLACK);
            }
        }
        draw_hints(self.buf, "Toggle", "Next");
        ScreenId::Settings
    }

    fn show_device_info(&mut self) -> ScreenId {
        draw_header(self.buf, "DEVICE", None);
        let lines = [
            format!("FW {}", self.firmware_version),
            self.battery_pct
                .map(|p| format!("Battery {p}%"))
                .unwrap_or_else(|| "Battery --".to_string()),
            format!("RTC {}", self.device_rtc),
            format!("Notes {}", self.note_count),
        ];
        let mut y = 40;
        for line in &lines {
            draw_str(self.buf, 12, y, line, 1, BLACK);
            y += 22;
        }
        draw_hints(self.buf, "Back", "");
        ScreenId::DeviceInfo
    }

    fn show_note_list(&mut self) -> ScreenId {
        let title = if self.list_filter.is_empty() || self.list_filter == "All" {
            "NOTES".to_string()
        } else {
            self.list_filter.to_string()
        };
        draw_header(self.buf, &title, None);
        draw_str(self.buf, 12, 40, &format!("#{:03}", self.detail_num.max(1)), 1, BLACK);
        draw_str(self.buf, 12, 58, "Voice note", 1, BLACK);
        draw_hints(self.buf, "Open", "Next");
        ScreenId::NoteList
    }

    fn show_note_detail(&mut self) -> ScreenId {
        let hdr = format!("#{:03}", self.detail_num);
        draw_header(self.buf, &hdr, Some(self.detail_tag));
        let mut y = 36;
        let start = self.detail_page * 7;
        for line in self.detail_lines.iter().skip(start).take(7) {
            draw_str(self.buf, 8, y, line, 1, BLACK);
            y += 18;
        }
        draw_hints(self.buf, "Play", "Scroll");
        ScreenId::NoteDetail
    }

    fn show_delete_confirm(&mut self) -> ScreenId {
        draw_header(self.buf, "DELETE?", None);
        draw_str_centered(
            self.buf,
            (WIDTH / 2) as i32,
            80,
            &format!("#{:03}", self.detail_num),
            1,
            BLACK,
        );
        draw_str_centered(self.buf, (WIDTH / 2) as i32, 110, "REC confirm", 1, BLACK);
        draw_hints(self.buf, "Yes", "Back");
        ScreenId::DeleteConfirm
    }

    fn show_transfer(&mut self) -> ScreenId {
        draw_header(self.buf, "TRANSFER", None);
        draw_str_centered(self.buf, (WIDTH / 2) as i32, 70, "Portal active", 1, BLACK);
        if !self.transfer_ip.is_empty() {
            draw_str_centered(self.buf, (WIDTH / 2) as i32, 100, self.transfer_ip, 1, BLACK);
        }
        draw_hints(self.buf, "Exit", "");
        ScreenId::Transfer
    }

    pub fn show_error_screen(&mut self, msg: &str) -> ScreenId {
        clear_screen(self.buf);
        draw_header(self.buf, "ERROR", None);
        draw_str_centered(self.buf, (WIDTH / 2) as i32, 90, msg, 1, BLACK);
        draw_hints(self.buf, "Dismiss", "");
        ScreenId::Error
    }

    pub fn show_battery_low(&mut self) -> ScreenId {
        clear_screen(self.buf);
        draw_header(self.buf, "BATTERY", None);
        draw_str_centered(self.buf, (WIDTH / 2) as i32, 90, "Low battery", 1, BLACK);
        draw_str_centered(self.buf, (WIDTH / 2) as i32, 115, "Charge soon", 1, BLACK);
        ScreenId::BatteryLow
    }

    pub fn show_ultra_sleep(&mut self) -> ScreenId {
        clear_screen(self.buf);
        draw_str_centered(self.buf, (WIDTH / 2) as i32, (HEIGHT / 2) as i32 - 10, "Sleeping", 1, BLACK);
        ScreenId::UltraSleep
    }

    pub fn show_wifi_connecting(&mut self, attempt: u32, max: u32) -> ScreenId {
        clear_screen(self.buf);
        draw_header(self.buf, "WIFI", None);
        draw_str_centered(self.buf, (WIDTH / 2) as i32, 90, "Connecting", 1, BLACK);
        let status = format!("{attempt}/{max}");
        let sw = text_width(&status, 1);
        draw_str(self.buf, (WIDTH as i32 - sw) / 2, 115, &status, 1, BLACK);
        ScreenId::WifiConnecting
    }

    pub fn show_transcribing(&mut self) -> ScreenId {
        clear_screen(self.buf);
        draw_header(self.buf, "SYNC", None);
        let line = format!("{}/{}", self.transcribe_done, self.transcribe_pending);
        draw_str_centered(self.buf, (WIDTH / 2) as i32, 100, "Transcribing", 1, BLACK);
        draw_str_centered(self.buf, (WIDTH / 2) as i32, 125, &line, 1, BLACK);
        ScreenId::Transcribing
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::display::draw::{get_pixel, BYTES};
    use crate::state::AppState;

    fn ctx<'a>(buf: &'a mut [u8], tags: &'a [String], detail_lines: &'a [String]) -> UiContext<'a> {
        UiContext {
            buf,
            battery_pct: Some(80),
            firmware_version: "v1.0",
            note_count: 3,
            menu_index: 0,
            settings_index: 0,
            tag_index: 0,
            tags,
            list_filter: "All",
            list_scroll: 0,
            detail_num: 1,
            detail_tag: "Work",
            detail_lines,
            detail_page: 0,
            error_msg: "SD ERR",
            device_rtc: "2026-07-11 12:00",
            transfer_ip: "192.168.1.42",
            transcribe_done: 1,
            transcribe_pending: 3,
            sounds_on: true,
        }
    }

    #[test]
    fn idle_screen_renders_header_region() {
        let mut buf = vec![0xFF; BYTES];
        let tags = vec!["Work".into(), "Idea".into()];
        let lines = vec!["Hello".into()];
        let mut ui = ctx(&mut buf, &tags, &lines);
        assert_eq!(ui.render(AppState::Idle), ScreenId::Idle);
        assert_eq!(get_pixel(&buf, 100, 145), BLACK);
    }

    #[test]
    fn menu_highlights_selected_row() {
        let mut buf = vec![0xFF; BYTES];
        let tags = vec!["Work".into()];
        let lines = vec!["Hello".into()];
        let mut ui = ctx(&mut buf, &tags, &lines);
        ui.menu_index = 1;
        assert_eq!(ui.render(AppState::Menu), ScreenId::Menu);
        assert_eq!(get_pixel(&buf, 20, 66), BLACK);
    }

    #[test]
    fn error_screen_sets_message() {
        let mut buf = vec![0xFF; BYTES];
        let tags = vec!["Work".into()];
        let lines = vec!["Hello".into()];
        let mut ui = ctx(&mut buf, &tags, &lines);
        assert_eq!(ui.show_error_screen("SD ERR"), ScreenId::Error);
        assert_ne!(buf, vec![0xFF; BYTES]);
    }

    #[test]
    fn framebuffer_hash_stable_for_idle() {
        let mut a = vec![0xFF; BYTES];
        let mut b = vec![0xFF; BYTES];
        let tags = vec!["Work".into(), "Idea".into()];
        let lines = vec!["Hello".into()];
        ctx(&mut a, &tags, &lines).render(AppState::Idle);
        ctx(&mut b, &tags, &lines).render(AppState::Idle);
        assert_eq!(a, b);
    }
}
