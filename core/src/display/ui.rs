//! E-Ink screen render — calm, paper-like layouts for 200×200 monochrome panels.
//!
//! Design rules (e-Paper):
//! - Prefer white “paper” with sparse black ink (less ghosting / eyestrain)
//! - Soft headers (text + rule) instead of full black bars where possible
//! - Outline selection instead of inverted slabs
//! - Generous margins and one visual focus per screen

use crate::display::draw::{
    draw_battery_ring, draw_calm_disc, draw_check, draw_header, draw_hints, draw_select_row,
    draw_soft_header, draw_str, draw_str_centered, fill_circle, hline, stroke_circle, text_width,
    vline, BLACK, HEIGHT, WIDTH,
};
use crate::state::AppState;

pub const HEADER_H: i32 = 28;
pub const HINTS_Y: i32 = 180;
pub const MARGIN: i32 = 12;
pub const CONTENT_TOP: i32 = 36;

/// Clear framebuffer to white (paper).
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
        let cx = (WIDTH / 2) as i32;

        // Quiet status — battery only, top-right
        if let Some(pct) = self.battery_pct {
            draw_battery_ring(self.buf, WIDTH as i32 - 28, 28, pct);
            let label = format!("{pct}%");
            let lw = text_width(&label, 1);
            draw_str(self.buf, WIDTH as i32 - 28 - lw / 2, 46, &label, 1, BLACK);
        }

        // Brand breathing room
        draw_str_centered(self.buf, cx, 68, "ZOOP", 2, BLACK);
        let notes = if self.note_count == 1 {
            "1 note".to_string()
        } else {
            format!("{} notes", self.note_count)
        };
        draw_str_centered(self.buf, cx, 98, &notes, 1, BLACK);
        draw_str_centered(self.buf, cx, 116, "ready", 1, BLACK);

        // Open mic ring — calm focus, not a solid blot
        draw_calm_disc(self.buf, cx, 148, 22);

        draw_hints(self.buf, "Hold REC", "Menu");
        ScreenId::Idle
    }

    fn show_recording(&mut self) -> ScreenId {
        draw_soft_header(self.buf, "RECORDING", None);
        let cx = (WIDTH / 2) as i32;
        // Outline + small core — less ink than a huge filled disc
        stroke_circle(self.buf, cx, 100, 40, 2, BLACK);
        stroke_circle(self.buf, cx, 100, 28, 1, BLACK);
        fill_circle(self.buf, cx, 100, 10, BLACK);
        draw_str_centered(self.buf, cx, 152, "listening...", 1, BLACK);
        draw_hints(self.buf, "Release", "");
        ScreenId::Recording
    }

    fn show_saved(&mut self) -> ScreenId {
        draw_soft_header(self.buf, "SAVED", None);
        let cx = (WIDTH / 2) as i32;
        draw_check(self.buf, cx, 95);
        draw_str_centered(self.buf, cx, 140, "choose a tag", 1, BLACK);
        draw_hints(self.buf, "Save", "Next");
        ScreenId::Saved
    }

    fn show_tag_select(&mut self) -> ScreenId {
        draw_soft_header(self.buf, "TAG", None);
        let tag = self
            .tags
            .get(self.tag_index)
            .map(|s| s.as_str())
            .unwrap_or("Note");
        let cx = (WIDTH / 2) as i32;
        let tw = text_width(tag, 1).max(48);
        let box_w = tw + 24;
        let box_x = cx - box_w / 2;
        let box_y = 78;
        hline(self.buf, box_x, box_y, box_w, BLACK);
        hline(self.buf, box_x, box_y + 28, box_w, BLACK);
        vline(self.buf, box_x, box_y, 29, BLACK);
        vline(self.buf, box_x + box_w - 1, box_y, 29, BLACK);
        draw_str_centered(self.buf, cx, 88, tag, 1, BLACK);
        draw_str_centered(self.buf, cx, 140, "press REC to keep", 1, BLACK);
        draw_hints(self.buf, "Save", "Cycle");
        ScreenId::TagSelect
    }

    fn show_menu(&mut self) -> ScreenId {
        const ITEMS: [&str; 4] = ["Notes", "Tags", "Sync", "Settings"];
        draw_soft_header(self.buf, "MENU", None);
        let y0 = CONTENT_TOP + 4;
        for (i, item) in ITEMS.iter().enumerate() {
            let y = y0 + i as i32 * 30;
            draw_select_row(self.buf, y, item, i == self.menu_index);
        }
        draw_hints(self.buf, "Open", "Next");
        ScreenId::Menu
    }

    fn show_settings(&mut self) -> ScreenId {
        let sounds = if self.sounds_on { "on" } else { "off" };
        let items = [
            format!("Sounds  {sounds}"),
            "Transfer".to_string(),
            "Device".to_string(),
        ];
        draw_soft_header(self.buf, "SETTINGS", None);
        let y0 = CONTENT_TOP + 8;
        for (i, item) in items.iter().enumerate() {
            let y = y0 + i as i32 * 32;
            draw_select_row(self.buf, y, item, i == self.settings_index);
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
            format!("Notes  {}", self.note_count),
        ];
        let mut y = CONTENT_TOP + 8;
        for line in &lines {
            draw_str(self.buf, MARGIN + 4, y, line, 1, BLACK);
            y += 26;
        }
        draw_hints(self.buf, "Back", "");
        ScreenId::DeviceInfo
    }

    fn show_note_list(&mut self) -> ScreenId {
        let title = if self.list_filter.is_empty() || self.list_filter == "All" {
            "NOTES"
        } else {
            self.list_filter
        };
        draw_soft_header(self.buf, title, None);
        let cx = (WIDTH / 2) as i32;
        let num = format!("#{:03}", self.detail_num.max(1));
        draw_str_centered(self.buf, cx, 70, &num, 2, BLACK);
        draw_str_centered(self.buf, cx, 108, "voice note", 1, BLACK);
        if !self.detail_tag.is_empty() {
            draw_str_centered(self.buf, cx, 130, self.detail_tag, 1, BLACK);
        }
        draw_hints(self.buf, "Open", "Next");
        ScreenId::NoteList
    }

    fn show_note_detail(&mut self) -> ScreenId {
        let hdr = format!("#{:03}", self.detail_num);
        draw_soft_header(self.buf, &hdr, Some(self.detail_tag));
        // 6 lines with airier spacing — easier on e-Ink eyes
        let mut y = CONTENT_TOP + 4;
        let start = self.detail_page * 6;
        for line in self.detail_lines.iter().skip(start).take(6) {
            draw_str(self.buf, MARGIN, y, line, 1, BLACK);
            y += 20;
        }
        draw_hints(self.buf, "Play", "Scroll");
        ScreenId::NoteDetail
    }

    fn show_delete_confirm(&mut self) -> ScreenId {
        draw_soft_header(self.buf, "DELETE", None);
        let cx = (WIDTH / 2) as i32;
        draw_str_centered(
            self.buf,
            cx,
            72,
            &format!("#{:03}", self.detail_num),
            2,
            BLACK,
        );
        draw_str_centered(self.buf, cx, 110, "remove this note?", 1, BLACK);
        draw_str_centered(self.buf, cx, 132, "cannot undo", 1, BLACK);
        draw_hints(self.buf, "Yes", "Back");
        ScreenId::DeleteConfirm
    }

    fn show_transfer(&mut self) -> ScreenId {
        draw_soft_header(self.buf, "TRANSFER", None);
        let cx = (WIDTH / 2) as i32;
        draw_str_centered(self.buf, cx, 64, "portal open", 1, BLACK);
        // Soft box for IP
        if !self.transfer_ip.is_empty() {
            draw_select_row(self.buf, 88, self.transfer_ip, true);
        }
        draw_str_centered(self.buf, cx, 140, "open in browser", 1, BLACK);
        draw_hints(self.buf, "Exit", "");
        ScreenId::Transfer
    }

    pub fn show_error_screen(&mut self, msg: &str) -> ScreenId {
        clear_screen(self.buf);
        // Keep solid header for errors — clear visual priority
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
        // Crescent-ish: open ring, minimal ink
        stroke_circle(self.buf, cx, cy, 20, 1, BLACK);
        draw_str_centered(self.buf, cx, cy + 36, "resting", 1, BLACK);
        ScreenId::UltraSleep
    }

    pub fn show_wifi_connecting(&mut self, attempt: u32, max: u32) -> ScreenId {
        clear_screen(self.buf);
        draw_soft_header(self.buf, "WIFI", None);
        let cx = (WIDTH / 2) as i32;
        draw_str_centered(self.buf, cx, 80, "connecting", 1, BLACK);
        let status = format!("{attempt} / {max}");
        draw_str_centered(self.buf, cx, 110, &status, 1, BLACK);
        // Simple progress ticks
        let ticks = ((attempt.min(max) as i32 * 5) / max.max(1) as i32).clamp(0, 5);
        let start_x = cx - 28;
        for i in 0..5 {
            let x = start_x + i * 12;
            if i < ticks {
                fill_circle(self.buf, x, 140, 3, BLACK);
            } else {
                stroke_circle(self.buf, x, 140, 3, 1, BLACK);
            }
        }
        ScreenId::WifiConnecting
    }

    pub fn show_transcribing(&mut self) -> ScreenId {
        clear_screen(self.buf);
        draw_soft_header(self.buf, "SYNC", None);
        let cx = (WIDTH / 2) as i32;
        let line = format!("{}/{}", self.transcribe_done, self.transcribe_pending);
        draw_str_centered(self.buf, cx, 88, "writing words", 1, BLACK);
        draw_str_centered(self.buf, cx, 118, &line, 2, BLACK);
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
            device_rtc: "set",
            transfer_ip: "192.168.1.42",
            transcribe_done: 1,
            transcribe_pending: 3,
            sounds_on: true,
        }
    }

    #[test]
    fn idle_screen_renders_calm_disc() {
        let mut buf = vec![0xFF; BYTES];
        let tags = vec!["Work".into(), "Idea".into()];
        let lines = vec!["Akash Soni".into()];
        let mut ui = ctx(&mut buf, &tags, &lines);
        assert_eq!(ui.render(AppState::Idle), ScreenId::Idle);
        // Core of calm disc at center
        assert_eq!(get_pixel(&buf, 100, 148), BLACK);
    }

    #[test]
    fn menu_outline_selects_row() {
        let mut buf = vec![0xFF; BYTES];
        let tags = vec!["Work".into()];
        let lines = vec!["Akash Soni".into()];
        let mut ui = ctx(&mut buf, &tags, &lines);
        ui.menu_index = 1;
        assert_eq!(ui.render(AppState::Menu), ScreenId::Menu);
        // Left accent of selected outline row (Tags at y≈70)
        assert_eq!(get_pixel(&buf, 12, 75), BLACK);
    }

    #[test]
    fn error_screen_sets_message() {
        let mut buf = vec![0xFF; BYTES];
        let tags = vec!["Work".into()];
        let lines = vec!["Akash Soni".into()];
        let mut ui = ctx(&mut buf, &tags, &lines);
        assert_eq!(ui.show_error_screen("SD ERR"), ScreenId::Error);
        assert_ne!(buf, vec![0xFF; BYTES]);
    }

    #[test]
    fn framebuffer_hash_stable_for_idle() {
        let mut a = vec![0xFF; BYTES];
        let mut b = vec![0xFF; BYTES];
        let tags = vec!["Work".into(), "Idea".into()];
        let lines = vec!["Akash Soni".into()];
        ctx(&mut a, &tags, &lines).render(AppState::Idle);
        ctx(&mut b, &tags, &lines).render(AppState::Idle);
        assert_eq!(a, b);
    }
}
