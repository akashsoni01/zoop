//! Host-only QR demo: encode a string, decode via `zoop_core::qr`, print JSON line.

mod board;

use qrcode::QrCode;
use zoop_core::qr::decode_grayscale;

fn render_qr_grayscale(payload: &str, module_px: usize, quiet: usize) -> (usize, Vec<u8>) {
    let code = QrCode::new(payload.as_bytes()).expect("encode QR");
    let modules = code.width();
    let total = modules + quiet * 2;
    let size = total * module_px;
    let mut pixels = vec![255u8; size * size];
    for y in 0..modules {
        for x in 0..modules {
            if code[(x, y)] == qrcode::Color::Dark {
                let ox = (x + quiet) * module_px;
                let oy = (y + quiet) * module_px;
                for dy in 0..module_px {
                    for dx in 0..module_px {
                        pixels[(oy + dy) * size + (ox + dx)] = 0;
                    }
                }
            }
        }
    }
    (size, pixels)
}

fn main() {
    let payload = "upi://pay?pa=merchant@oksbi&am=200.00";
    let (size, pixels) = render_qr_grayscale(payload, 4, 4);
    match decode_grayscale(size, size, &pixels) {
        Ok(r) => {
            println!(
                "{{\"ok\":true,\"payload\":\"{}\",\"len\":{}}}",
                r.payload.replace('\\', "\\\\").replace('"', "\\\""),
                r.payload.len()
            );
        }
        Err(e) => {
            println!("{{\"ok\":false,\"error\":\"{e}\"}}");
            std::process::exit(1);
        }
    }
}
