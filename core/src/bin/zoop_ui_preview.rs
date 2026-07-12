//! Export every UPI payment e-Ink screen to BMP + HTML gallery.

use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

use zoop_core::display::draw::{get_pixel, BYTES, HEIGHT, WIDTH, BLACK};
use zoop_core::display::ui::{ScreenId, UiContext};
use zoop_core::state::AppState;
use zoop_core::upi::build_upi_uri;

fn write_bmp(path: &std::path::Path, buf: &[u8]) -> std::io::Result<()> {
    let w = WIDTH as u32;
    let h = HEIGHT as u32;
    let row_stride = ((w * 3 + 3) / 4) * 4;
    let pixel_size = row_stride * h;
    let file_size = 54 + pixel_size;

    let mut f = File::create(path)?;
    f.write_all(b"BM")?;
    f.write_all(&(file_size as u32).to_le_bytes())?;
    f.write_all(&0u16.to_le_bytes())?;
    f.write_all(&0u16.to_le_bytes())?;
    f.write_all(&54u32.to_le_bytes())?;
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
    uri: &'a str,
    history: &'a [String],
    prices: &'a [String],
    tags: &'a [String],
) -> UiContext<'a> {
    UiContext {
        buf,
        battery_pct: Some(78),
        firmware_version: "v1.0",
        merchant_name: "Akash Soni",
        upi_vpa: "akash@oksbi",
        amount_inr: "250.00",
        upi_uri: uri,
        txn_note: "order 42",
        txn_count: 5,
        menu_index: 0,
        settings_index: 0,
        history_index: 0,
        history_lines: history,
        history_total: "Rs 1530",
        price_labels: prices,
        price_index: 2,
        error_msg: "PAY FAIL",
        device_rtc: "set",
        sounds_on: true,
        sync_done: 1,
        sync_pending: 3,
        note_count: 5,
        tag_index: 0,
        tags,
        list_filter: "All",
        list_scroll: 0,
        detail_num: 12,
        detail_tag: "PAID",
        detail_lines: &[],
        detail_page: 0,
        transfer_ip: "",
        transcribe_done: 0,
        transcribe_pending: 0,
    }
}

fn main() {
    let out = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../target/ui-preview");
    fs::create_dir_all(&out).expect("mkdir ui-preview");

    let uri = build_upi_uri("akash@oksbi", "Akash Soni", "250.00", "Zoop Pay");
    let history = vec![
        "#005  Rs 250".into(),
        "#004  Rs 80".into(),
        "#003  Rs 1200".into(),
        "#002  Rs 40".into(),
        "#001  Rs 15".into(),
    ];
    let prices = vec![
        "Rs 50".into(),
        "Rs 100".into(),
        "Rs 200".into(),
        "Rs 500".into(),
        "Rs 1000".into(),
        "Rs 2000".into(),
    ];
    let tags: Vec<String> = vec![];

    let screens: Vec<(&str, Box<dyn Fn(&mut UiContext<'_>) -> ScreenId>)> = vec![
        ("01_home_qr", Box::new(|ui| ui.render(AppState::Idle))),
        ("02_price_pick", Box::new(|ui| ui.render(AppState::PricePick))),
        ("03_priced_qr", Box::new(|ui| ui.render(AppState::ShowQr))),
        ("04_waiting", Box::new(|ui| ui.render(AppState::Waiting))),
        ("05_success", Box::new(|ui| ui.render(AppState::Success))),
        ("06_recent_txns", Box::new(|ui| ui.render(AppState::History))),
        ("07_menu", Box::new(|ui| ui.render(AppState::Menu))),
        ("08_cancel", Box::new(|ui| ui.render(AppState::CancelConfirm))),
        ("09_merchant", Box::new(|ui| ui.render(AppState::Merchant))),
        ("10_settings", Box::new(|ui| ui.render(AppState::Settings))),
        ("11_device", Box::new(|ui| ui.render(AppState::DeviceInfo))),
        ("12_error", Box::new(|ui| ui.render(AppState::Error))),
    ];

    let mut cards = String::new();
    println!("=== Zoop Pay UI preview (UPI / e-Paper) ===");
    println!("UPI URI: {uri}");
    println!("Output: {}", out.display());

    for (name, render) in &screens {
        let mut buf = vec![0xFFu8; BYTES];
        let mut ui = make_ctx(&mut buf, &uri, &history, &prices, &tags);
        let id = render(&mut ui);
        let bmp = out.join(format!("{name}.bmp"));
        write_bmp(&bmp, &buf).expect("write bmp");
        println!(
            "  wrote {} ({:?})",
            bmp.file_name().unwrap().to_string_lossy(),
            id
        );
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
  <title>Zoop Pay — UPI e-Paper UI</title>
  <style>
    :root {{ font-family: ui-sans-serif, system-ui, sans-serif; color: #111; background: #e8e4dc; }}
    body {{ margin: 0; padding: 24px; }}
    h1 {{ font-size: 28px; letter-spacing: -0.03em; margin: 0 0 8px; }}
    .sub {{ color: #555; margin-bottom: 24px; max-width: 720px; line-height: 1.45; }}
    code {{ background: #fff; padding: 1px 6px; border: 1px solid #ccc; }}
    .grid {{ display: grid; grid-template-columns: repeat(auto-fill, minmax(220px, 1fr)); gap: 20px; }}
    figure {{ margin: 0; background: #fff; padding: 12px; border: 1px solid #111; box-shadow: 4px 4px 0 #111; }}
    figcaption {{ margin-top: 8px; font-size: 12px; text-transform: uppercase; letter-spacing: 0.06em; color: #444; }}
    img {{ display: block; width: 100%; height: auto; aspect-ratio: 1; }}
  </style>
</head>
<body>
  <h1>Zoop Pay — UPI UI</h1>
  <p class="sub">
    Home any-amount QR · Menu → Prices → pick → amount QR → Done.
    Preview: <code>cargo run -p zoop-core --bin zoop-ui-preview</code>
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
    let _ = Command::new("open").arg(&index).status();
}
