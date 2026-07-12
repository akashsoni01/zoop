//! Export every E-Ink screen to BMP + HTML gallery for visual review (no board needed).

use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

use zoop_core::display::draw::{get_pixel, BYTES, HEIGHT, WIDTH, BLACK};
use zoop_core::display::ui::{UiContext, ScreenId};
use zoop_core::state::AppState;

fn write_bmp(path: &std::path::Path, buf: &[u8]) -> std::io::Result<()> {
    let w = WIDTH as u32;
    let h = HEIGHT as u32;
    // 24-bit BMP, rows padded to 4 bytes, bottom-up
    let row_stride = ((w * 3 + 3) / 4) * 4;
    let pixel_size = row_stride * h;
    let file_size = 54 + pixel_size;

    let mut f = File::create(path)?;
    // BITMAPFILEHEADER
    f.write_all(b"BM")?;
    f.write_all(&(file_size as u32).to_le_bytes())?;
    f.write_all(&0u16.to_le_bytes())?;
    f.write_all(&0u16.to_le_bytes())?;
    f.write_all(&54u32.to_le_bytes())?;
    // BITMAPINFOHEADER
    f.write_all(&40u32.to_le_bytes())?;
    f.write_all(&(w as i32).to_le_bytes())?;
    f.write_all(&(h as i32).to_le_bytes())?;
    f.write_all(&1u16.to_le_bytes())?;
    f.write_all(&24u16.to_le_bytes())?;
    f.write_all(&0u32.to_le_bytes())?;
    f.write_all(&pixel_size.to_le_bytes())?;
    f.write_all(&2835i32.to_le_bytes())?;
    f.write_all(&2835i32.to_le_bytes())?;
    f.write_all(&0u32.to_le_bytes())?;
    f.write_all(&0u32.to_le_bytes())?;

    let mut row = vec![0u8; row_stride as usize];
    for y in (0..h as i32).rev() {
        row.fill(0);
        for x in 0..w as i32 {
            let v = if get_pixel(buf, x, y) == BLACK {
                0u8
            } else {
                255u8
            };
            let o = (x as usize) * 3;
            row[o] = v;
            row[o + 1] = v;
            row[o + 2] = v;
        }
        f.write_all(&row)?;
    }
    Ok(())
}

fn make_ctx<'a>(
    buf: &'a mut [u8],
    tags: &'a [String],
    detail_lines: &'a [String],
) -> UiContext<'a> {
    UiContext {
        buf,
        battery_pct: Some(78),
        firmware_version: "v1.0",
        note_count: 3,
        menu_index: 1,
        settings_index: 0,
        tag_index: 1,
        tags,
        list_filter: "Work",
        list_scroll: 0,
        detail_num: 12,
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

fn main() {
    let out = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../target/ui-preview");
    fs::create_dir_all(&out).expect("mkdir ui-preview");

    let tags = vec![
        "Note".into(),
        "Work".into(),
        "Idea".into(),
        "Buy".into(),
        "Private".into(),
    ];
    let detail_lines = vec![
        "Meeting notes from".into(),
        "standup - ship the".into(),
        "WAV recorder next".into(),
        "and wire e-paper".into(),
        "partial refresh.".into(),
        "Also fix font gaps".into(),
        "for lowercase.".into(),
        "Page two line.".into(),
    ];

    let screens: Vec<(&str, Box<dyn Fn(&mut UiContext<'_>) -> ScreenId>)> = vec![
        ("01_idle", Box::new(|ui| ui.render(AppState::Idle))),
        ("02_recording", Box::new(|ui| ui.render(AppState::Recording))),
        ("03_saved", Box::new(|ui| ui.render(AppState::Saved))),
        ("04_tag_select", Box::new(|ui| ui.render(AppState::TagSelect))),
        ("05_menu", Box::new(|ui| ui.render(AppState::Menu))),
        ("06_settings", Box::new(|ui| ui.render(AppState::Settings))),
        ("07_device_info", Box::new(|ui| ui.render(AppState::DeviceInfo))),
        ("08_note_list", Box::new(|ui| ui.render(AppState::NoteList))),
        ("09_note_detail", Box::new(|ui| ui.render(AppState::NoteDetail))),
        (
            "10_delete_confirm",
            Box::new(|ui| ui.render(AppState::DeleteConfirm)),
        ),
        ("11_transfer", Box::new(|ui| ui.render(AppState::Transfer))),
        ("12_error", Box::new(|ui| ui.render(AppState::Error))),
        ("13_battery_low", Box::new(|ui| ui.show_battery_low())),
        ("14_ultra_sleep", Box::new(|ui| ui.show_ultra_sleep())),
        (
            "15_wifi_connecting",
            Box::new(|ui| ui.show_wifi_connecting(4, 20)),
        ),
        ("16_transcribing", Box::new(|ui| ui.show_transcribing())),
    ];

    let mut cards = String::new();
    println!("=== Zoop UI preview (200×200 e-Paper) ===");
    println!("Output: {}", out.display());

    for (name, render) in &screens {
        let mut buf = vec![0xFFu8; BYTES];
        let mut ui = make_ctx(&mut buf, &tags, &detail_lines);
        let id = render(&mut ui);
        let bmp = out.join(format!("{name}.bmp"));
        write_bmp(&bmp, &buf).expect("write bmp");
        println!("  wrote {} ({:?})", bmp.file_name().unwrap().to_string_lossy(), id);
        cards.push_str(&format!(
            r#"<figure>
  <img src="{name}.bmp" width="400" height="400" alt="{name}" style="image-rendering:pixelated;border:1px solid #222;background:#f4f1ea;" />
  <figcaption>{name} · {id:?}</figcaption>
</figure>
"#
        ));
    }

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <title>Zoop UI preview — 200×200 e-Paper</title>
  <style>
    :root {{ font-family: ui-sans-serif, system-ui, sans-serif; color: #111; background: #e8e4dc; }}
    body {{ margin: 0; padding: 24px; }}
    h1 {{ font-size: 28px; letter-spacing: -0.03em; margin: 0 0 8px; }}
    .sub {{ color: #555; margin-bottom: 24px; max-width: 640px; line-height: 1.4; }}
    .grid {{ display: grid; grid-template-columns: repeat(auto-fill, minmax(220px, 1fr)); gap: 20px; }}
    figure {{ margin: 0; background: #fff; padding: 12px; border: 1px solid #111; box-shadow: 4px 4px 0 #111; }}
    figcaption {{ margin-top: 8px; font-size: 12px; text-transform: uppercase; letter-spacing: 0.06em; color: #444; }}
    img {{ display: block; width: 100%; height: auto; aspect-ratio: 1; }}
  </style>
</head>
<body>
  <h1>Zoop UI preview</h1>
  <p class="sub">
    Host render of the 200×200 monochrome framebuffer (same pixels the Waveshare e-Paper will show).
    Scale is 2× for review. Run again with <code>cargo run -p zoop-core --bin zoop-ui-preview</code>.
  </p>
  <div class="grid">
{cards}
  </div>
</body>
</html>
"#
    );

    let index = out.join("index.html");
    fs::write(&index, html).expect("write html");
    println!("Gallery: {}", index.display());

    // Open in default browser on macOS
    let _ = Command::new("open").arg(&index).status();
    println!("Opened in browser. Inspect each screen for layout/typography.");
}
