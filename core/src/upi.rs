//! UPI deep-link helpers for Bharat QR / UPI intent strings.

/// Build a UPI payment URI (NPCI intent format).
///
/// Example: `upi://pay?pa=merchant@upi&pn=Akash%20Soni&am=100.00&cu=INR&tn=Order`
pub fn build_upi_uri(vpa: &str, payee_name: &str, amount_inr: &str, note: &str) -> String {
    let mut q = format!(
        "upi://pay?pa={}&pn={}&cu=INR",
        url_encode(vpa),
        url_encode(payee_name)
    );
    if !amount_inr.is_empty() {
        q.push_str("&am=");
        q.push_str(&url_encode(amount_inr));
    }
    if !note.is_empty() {
        q.push_str("&tn=");
        q.push_str(&url_encode(note));
    }
    q
}

/// Minimal percent-encoding for UPI query values.
fn url_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'@' => {
                out.push(b as char);
            }
            b' ' => out.push_str("%20"),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// Format paise/rupees display: `"100.00"` → `"Rs 100"` for e-Ink (no rupee glyph in 5×7 font).
pub fn format_amount_label(amount_inr: &str) -> String {
    if amount_inr.is_empty() {
        return "any amount".to_string();
    }
    // Strip trailing .00 for calmer UI
    let clean = amount_inr.trim_end_matches(".00").trim_end_matches(".0");
    format!("Rs {clean}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_upi_uri() {
        let u = build_upi_uri("akash@upi", "Akash Soni", "250.00", "Zoop");
        assert!(u.starts_with("upi://pay?"));
        assert!(u.contains("pa=akash@upi"));
        assert!(u.contains("pn=Akash%20Soni"));
        assert!(u.contains("am=250.00"));
        assert!(u.contains("cu=INR"));
        assert!(u.contains("tn=Zoop"));
    }

    #[test]
    fn formats_amount() {
        assert_eq!(format_amount_label("100.00"), "Rs 100");
        assert_eq!(format_amount_label(""), "any amount");
    }
}
