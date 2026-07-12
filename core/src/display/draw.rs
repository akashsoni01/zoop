//! 200×200 monochrome framebuffer — ports `draw.cpp` primitives.

pub const WIDTH: u16 = 200;
pub const HEIGHT: u16 = 200;
pub const BYTES: usize = (WIDTH as usize * HEIGHT as usize) / 8;

pub const BLACK: u8 = 0;
pub const WHITE: u8 = 1;

/// Set one pixel (MSB-first within each byte, row-major).
pub fn set_pixel(buf: &mut [u8], x: i32, y: i32, color: u8) {
    if x < 0 || y < 0 || x >= WIDTH as i32 || y >= HEIGHT as i32 {
        return;
    }
    let idx = (y as usize) * (WIDTH as usize / 8) + (x as usize / 8);
    let bit = 7 - (x as u8 % 8);
    if color == WHITE {
        buf[idx] |= 1 << bit;
    } else {
        buf[idx] &= !(1 << bit);
    }
}

pub fn get_pixel(buf: &[u8], x: i32, y: i32) -> u8 {
    if x < 0 || y < 0 || x >= WIDTH as i32 || y >= HEIGHT as i32 {
        return WHITE;
    }
    let idx = (y as usize) * (WIDTH as usize / 8) + (x as usize / 8);
    let bit = 7 - (x as u8 % 8);
    if buf[idx] & (1 << bit) != 0 {
        WHITE
    } else {
        BLACK
    }
}

pub fn fill_rect(buf: &mut [u8], x: i32, y: i32, w: i32, h: i32, color: u8) {
    for dy in 0..h {
        for dx in 0..w {
            set_pixel(buf, x + dx, y + dy, color);
        }
    }
}

pub fn stroke_rect(buf: &mut [u8], x: i32, y: i32, w: i32, h: i32, thickness: i32, color: u8) {
    let t = thickness.max(1);
    fill_rect(buf, x, y, w, t, color);
    fill_rect(buf, x, y + h - t, w, t, color);
    fill_rect(buf, x, y, t, h, color);
    fill_rect(buf, x + w - t, y, t, h, color);
}

pub fn hline(buf: &mut [u8], x: i32, y: i32, w: i32, color: u8) {
    for dx in 0..w {
        set_pixel(buf, x + dx, y, color);
    }
}

pub fn vline(buf: &mut [u8], x: i32, y: i32, h: i32, color: u8) {
    for dy in 0..h {
        set_pixel(buf, x, y + dy, color);
    }
}

pub fn line(buf: &mut [u8], x0: i32, y0: i32, x1: i32, y1: i32, color: u8) {
    let mut x = x0;
    let mut y = y0;
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    loop {
        set_pixel(buf, x, y, color);
        if x == x1 && y == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}

pub fn fill_circle(buf: &mut [u8], cx: i32, cy: i32, r: i32, color: u8) {
    for y in -r..=r {
        for x in -r..=r {
            if x * x + y * y <= r * r {
                set_pixel(buf, cx + x, cy + y, color);
            }
        }
    }
}

pub fn stroke_circle(buf: &mut [u8], cx: i32, cy: i32, r: i32, thickness: i32, color: u8) {
    for t in 0..thickness {
        let rr = r - t;
        for angle in 0..360 {
            let rad = angle as f32 * std::f32::consts::PI / 180.0;
            let x = cx + (rr as f32 * rad.cos()).round() as i32;
            let y = cy + (rr as f32 * rad.sin()).round() as i32;
            set_pixel(buf, x, y, color);
        }
    }
}

/// Minimal 5×7 bitmap font (ASCII printable — rows, 5 MSBs used).
const FONT_W: i32 = 5;

fn glyph(c: char) -> [u8; 7] {
    match c {
        'A' => [0x0E, 0x11, 0x11, 0x1F, 0x11, 0x11, 0x11],
        'B' => [0x1E, 0x11, 0x11, 0x1E, 0x11, 0x11, 0x1E],
        'C' => [0x0E, 0x11, 0x10, 0x10, 0x10, 0x11, 0x0E],
        'D' => [0x1E, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1E],
        'E' => [0x1F, 0x10, 0x10, 0x1E, 0x10, 0x10, 0x1F],
        'F' => [0x1F, 0x10, 0x10, 0x1E, 0x10, 0x10, 0x10],
        'G' => [0x0E, 0x11, 0x10, 0x17, 0x11, 0x11, 0x0E],
        'H' => [0x11, 0x11, 0x11, 0x1F, 0x11, 0x11, 0x11],
        'I' => [0x0E, 0x04, 0x04, 0x04, 0x04, 0x04, 0x0E],
        'J' => [0x01, 0x01, 0x01, 0x01, 0x11, 0x11, 0x0E],
        'K' => [0x11, 0x12, 0x14, 0x18, 0x14, 0x12, 0x11],
        'L' => [0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x1F],
        'M' => [0x11, 0x1B, 0x15, 0x15, 0x11, 0x11, 0x11],
        'N' => [0x11, 0x19, 0x15, 0x13, 0x11, 0x11, 0x11],
        'O' => [0x0E, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E],
        'P' => [0x1E, 0x11, 0x11, 0x1E, 0x10, 0x10, 0x10],
        'Q' => [0x0E, 0x11, 0x11, 0x11, 0x15, 0x12, 0x0D],
        'R' => [0x1E, 0x11, 0x11, 0x1E, 0x14, 0x12, 0x11],
        'S' => [0x0E, 0x11, 0x10, 0x0E, 0x01, 0x11, 0x0E],
        'T' => [0x1F, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04],
        'U' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E],
        'V' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x0A, 0x04],
        'W' => [0x11, 0x11, 0x11, 0x15, 0x15, 0x1B, 0x11],
        'X' => [0x11, 0x11, 0x0A, 0x04, 0x0A, 0x11, 0x11],
        'Y' => [0x11, 0x11, 0x0A, 0x04, 0x04, 0x04, 0x04],
        'Z' => [0x1F, 0x01, 0x02, 0x04, 0x08, 0x10, 0x1F],
        '0' => [0x0E, 0x11, 0x13, 0x15, 0x19, 0x11, 0x0E],
        '1' => [0x04, 0x0C, 0x04, 0x04, 0x04, 0x04, 0x0E],
        '2' => [0x0E, 0x11, 0x01, 0x06, 0x08, 0x10, 0x1F],
        '3' => [0x1F, 0x01, 0x02, 0x06, 0x01, 0x11, 0x0E],
        '4' => [0x02, 0x06, 0x0A, 0x12, 0x1F, 0x02, 0x02],
        '5' => [0x1F, 0x10, 0x1E, 0x01, 0x01, 0x11, 0x0E],
        '6' => [0x06, 0x08, 0x10, 0x1E, 0x11, 0x11, 0x0E],
        '7' => [0x1F, 0x01, 0x02, 0x04, 0x08, 0x08, 0x08],
        '8' => [0x0E, 0x11, 0x11, 0x0E, 0x11, 0x11, 0x0E],
        '9' => [0x0E, 0x11, 0x11, 0x0F, 0x01, 0x02, 0x0C],
        '.' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x0C, 0x0C],
        ':' => [0x00, 0x0C, 0x0C, 0x00, 0x0C, 0x0C, 0x00],
        '-' => [0x00, 0x00, 0x00, 0x1F, 0x00, 0x00, 0x00],
        '%' => [0x19, 0x1A, 0x02, 0x04, 0x08, 0x13, 0x03],
        '#' => [0x0A, 0x0A, 0x1F, 0x0A, 0x1F, 0x0A, 0x0A],
        '/' => [0x01, 0x01, 0x02, 0x04, 0x08, 0x10, 0x10],
        '?' => [0x0E, 0x11, 0x01, 0x02, 0x04, 0x00, 0x04],
        '!' => [0x04, 0x04, 0x04, 0x04, 0x04, 0x00, 0x04],
        '+' => [0x00, 0x04, 0x04, 0x1F, 0x04, 0x04, 0x00],
        '=' => [0x00, 0x00, 0x1F, 0x00, 0x1F, 0x00, 0x00],
        '_' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x1F],
        ' ' => [0; 7],
        _ => [0x1F, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1F],
    }
}

pub fn text_width(s: &str, scale: i32) -> i32 {
    let chars = s.chars().count() as i32;
    if chars == 0 {
        return 0;
    }
    chars * (FONT_W + 1) * scale - scale
}

pub fn draw_str(buf: &mut [u8], x: i32, y: i32, s: &str, scale: i32, color: u8) {
    let mut cx = x;
    for ch in s.chars() {
        // Bitmap font is uppercase/digit focused — map lowercase so UI labels stay readable.
        let ch = if ch.is_ascii_lowercase() {
            ch.to_ascii_uppercase()
        } else {
            ch
        };
        let g = glyph(ch);
        for (row, &bits) in g.iter().enumerate() {
            for col in 0..FONT_W {
                if bits & (1 << (4 - col)) != 0 {
                    for sy in 0..scale {
                        for sx in 0..scale {
                            set_pixel(
                                buf,
                                cx + col * scale + sx,
                                y + row as i32 * scale + sy,
                                color,
                            );
                        }
                    }
                }
            }
        }
        cx += (FONT_W + 1) * scale;
    }
}

pub fn draw_str_centered(buf: &mut [u8], cx: i32, y: i32, s: &str, scale: i32, color: u8) {
    let w = text_width(s, scale);
    draw_str(buf, cx - w / 2, y, s, scale, color);
}

/// Solid header bar (legacy / high-emphasis screens).
pub fn draw_header(buf: &mut [u8], title: &str, right: Option<&str>) {
    fill_rect(buf, 0, 0, WIDTH as i32, 28, BLACK);
    draw_str_centered(buf, (WIDTH / 2) as i32, 10, title, 1, WHITE);
    if let Some(r) = right {
        let rw = text_width(r, 1);
        draw_str(buf, WIDTH as i32 - 8 - rw, 10, r, 1, WHITE);
    }
}

/// Paper-like title: black text + thin rule — less ink, calmer on e-Paper.
pub fn draw_soft_header(buf: &mut [u8], title: &str, right: Option<&str>) {
    draw_str(buf, 12, 10, title, 1, BLACK);
    if let Some(r) = right {
        let rw = text_width(r, 1);
        draw_str(buf, WIDTH as i32 - 12 - rw, 10, r, 1, BLACK);
    }
    hline(buf, 12, 24, WIDTH as i32 - 24, BLACK);
}

/// Footer hints with breathing room (e-Ink friendly).
pub fn draw_hints(buf: &mut [u8], rec_label: &str, pwr_label: &str) {
    hline(buf, 12, 178, WIDTH as i32 - 24, BLACK);
    draw_str(buf, 12, 186, rec_label, 1, BLACK);
    let rw = text_width(pwr_label, 1);
    draw_str(buf, WIDTH as i32 - 12 - rw, 186, pwr_label, 1, BLACK);
}

/// Outline selection row — avoids large inverted black blocks (ghosting / harshness).
pub fn draw_select_row(buf: &mut [u8], y: i32, label: &str, selected: bool) {
    let x = 12;
    let w = WIDTH as i32 - 24;
    let h = 24;
    if selected {
        // Outer frame
        hline(buf, x, y, w, BLACK);
        hline(buf, x, y + h - 1, w, BLACK);
        vline(buf, x, y, h, BLACK);
        vline(buf, x + w - 1, y, h, BLACK);
        // Soft left accent
        fill_rect(buf, x, y, 3, h, BLACK);
        draw_str(buf, x + 12, y + 8, label, 1, BLACK);
    } else {
        draw_str(buf, x + 12, y + 8, label, 1, BLACK);
    }
}

pub fn draw_battery_ring(buf: &mut [u8], cx: i32, cy: i32, percent: u8) {
    stroke_circle(buf, cx, cy, 14, 1, BLACK);
    let filled = (percent as i32 * 360 / 100).min(360);
    for angle in (0..filled).step_by(3) {
        let rad = (angle - 90) as f32 * std::f32::consts::PI / 180.0;
        let x = cx + (12.0 * rad.cos()).round() as i32;
        let y = cy + (12.0 * rad.sin()).round() as i32;
        set_pixel(buf, x, y, BLACK);
    }
}

/// Open ring + small core — quieter than a solid disc.
pub fn draw_calm_disc(buf: &mut [u8], cx: i32, cy: i32, outer_r: i32) {
    stroke_circle(buf, cx, cy, outer_r, 2, BLACK);
    stroke_circle(buf, cx, cy, outer_r - 8, 1, BLACK);
    fill_circle(buf, cx, cy, 5, BLACK);
}

/// Simple check mark (saved / success).
pub fn draw_check(buf: &mut [u8], cx: i32, cy: i32) {
    stroke_circle(buf, cx, cy, 28, 2, BLACK);
    // check
    line(buf, cx - 12, cy, cx - 4, cy + 10, BLACK);
    line(buf, cx - 4, cy + 10, cx + 14, cy - 12, BLACK);
    line(buf, cx - 12, cy + 1, cx - 4, cy + 11, BLACK);
    line(buf, cx - 4, cy + 11, cx + 14, cy - 11, BLACK);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn framebuffer_dimensions() {
        let mut buf = vec![0xFF; BYTES];
        fill_rect(&mut buf, 0, 0, 200, 200, BLACK);
        assert_eq!(get_pixel(&buf, 0, 0), BLACK);
        assert_eq!(get_pixel(&buf, 199, 199), BLACK);
    }

    #[test]
    fn draw_str_changes_buffer() {
        let a = vec![0xFF; BYTES];
        let mut b = a.clone();
        draw_str(&mut b, 10, 10, "HI", 1, BLACK);
        assert_ne!(a, b);
    }

    #[test]
    fn header_bar_is_black_top_strip() {
        let mut buf = vec![0xFF; BYTES];
        draw_header(&mut buf, "MENU", None);
        assert_eq!(get_pixel(&buf, 10, 10), BLACK);
    }
}
