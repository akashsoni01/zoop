//! Payment domain models for Zoop Pay (UPI collect).

use crate::upi::{build_upi_uri, format_amount_label};

/// Merchant identity shown on home / merchant screen and encoded in QR.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MerchantProfile {
    pub name: String,
    pub vpa: String,
}

impl Default for MerchantProfile {
    fn default() -> Self {
        Self {
            name: "Akash Soni".into(),
            vpa: "akash@oksbi".into(),
        }
    }
}

/// Default preset amounts (INR) for the price-pick screens.
pub const DEFAULT_PRICE_INR: &[&str] = &[
    "50.00",
    "100.00",
    "200.00",
    "500.00",
    "1000.00",
    "2000.00",
];

/// Catalog of fixed collect amounts the merchant can offer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PriceCatalog {
    /// Amount strings like `"100.00"`.
    pub amounts: Vec<String>,
}

impl Default for PriceCatalog {
    fn default() -> Self {
        Self::from_defaults()
    }
}

impl PriceCatalog {
    pub fn from_defaults() -> Self {
        Self {
            amounts: DEFAULT_PRICE_INR
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
        }
    }

    pub fn len(&self) -> usize {
        self.amounts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.amounts.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<&str> {
        self.amounts.get(index).map(|s| s.as_str())
    }

    /// Labels for the price list UI (`Rs 100`, …).
    pub fn labels(&self) -> Vec<String> {
        self.amounts
            .iter()
            .map(|a| format_amount_label(a))
            .collect()
    }
}

/// Status of a collect attempt / stored payment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PaymentStatus {
    Draft,
    QrShown,
    Pending,
    Paid,
    Failed,
    Cancelled,
}

impl PaymentStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::QrShown => "qr",
            Self::Pending => "pending",
            Self::Paid => "paid",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }
}

/// Active collect request (drives QR + waiting UI).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaymentRequest {
    pub amount_inr: String,
    pub note: String,
    pub uri: String,
    pub status: PaymentStatus,
}

impl PaymentRequest {
    pub fn new(merchant: &MerchantProfile, amount_inr: &str, note: &str) -> Self {
        let uri = build_upi_uri(&merchant.vpa, &merchant.name, amount_inr, note);
        Self {
            amount_inr: amount_inr.to_string(),
            note: note.to_string(),
            uri,
            status: PaymentStatus::Draft,
        }
    }

    pub fn amount_label(&self) -> String {
        format_amount_label(&self.amount_inr)
    }

    pub fn mark_qr_shown(&mut self) {
        self.status = PaymentStatus::QrShown;
    }

    pub fn mark_pending(&mut self) {
        self.status = PaymentStatus::Pending;
    }

    pub fn mark_paid(&mut self) {
        self.status = PaymentStatus::Paid;
    }

    pub fn mark_failed(&mut self) {
        self.status = PaymentStatus::Failed;
    }

    pub fn mark_cancelled(&mut self) {
        self.status = PaymentStatus::Cancelled;
    }
}

/// Settled or in-progress payment for history list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaymentRecord {
    pub num: i32,
    pub amount_inr: String,
    pub note: String,
    pub status: PaymentStatus,
}

impl PaymentRecord {
    pub fn list_label(&self) -> String {
        format!("#{:03}  {}", self.num, format_amount_label(&self.amount_inr))
    }
}

/// In-memory payment ledger (host + firmware until SD schema lands).
#[derive(Debug, Default, Clone)]
pub struct PaymentLedger {
    pub records: Vec<PaymentRecord>,
    next_num: i32,
}

impl PaymentLedger {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            next_num: 1,
        }
    }

    pub fn next_num(&self) -> i32 {
        self.next_num
    }

    pub fn push_paid(&mut self, amount_inr: &str, note: &str) -> PaymentRecord {
        let rec = PaymentRecord {
            num: self.next_num,
            amount_inr: amount_inr.to_string(),
            note: note.to_string(),
            status: PaymentStatus::Paid,
        };
        self.next_num += 1;
        self.records.push(rec.clone());
        rec
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    pub fn history_labels(&self) -> Vec<String> {
        self.records.iter().rev().map(|r| r.list_label()).collect()
    }

    pub fn recent_labels(&self, max: usize) -> Vec<String> {
        self.records
            .iter()
            .rev()
            .take(max)
            .map(|r| r.list_label())
            .collect()
    }

    pub fn total_paid_label(&self) -> String {
        let paise: i64 = self
            .records
            .iter()
            .map(|r| parse_inr_paise(&r.amount_inr))
            .sum();
        let rupees = paise / 100;
        let rem = (paise % 100).unsigned_abs() as u32;
        if rem == 0 {
            format_amount_label(&format!("{rupees}.00"))
        } else {
            format_amount_label(&format!("{rupees}.{rem:02}"))
        }
    }
}

/// Parse `"100.00"` / `"50"` into paise (×100). Empty / invalid → 0.
pub fn parse_inr_paise(amount_inr: &str) -> i64 {
    let s = amount_inr.trim();
    if s.is_empty() {
        return 0;
    }
    if let Some((whole, frac)) = s.split_once('.') {
        let w: i64 = whole.parse().unwrap_or(0);
        let mut f = frac.chars().take(2).collect::<String>();
        while f.len() < 2 {
            f.push('0');
        }
        let f: i64 = f.parse().unwrap_or(0);
        w * 100 + f
    } else {
        s.parse::<i64>().unwrap_or(0) * 100
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_builds_upi_uri() {
        let m = MerchantProfile::default();
        let req = PaymentRequest::new(&m, "100.00", "Order");
        assert!(req.uri.contains("pa=akash@oksbi"));
        assert!(req.uri.contains("am=100.00"));
        assert_eq!(req.amount_label(), "Rs 100");
    }

    #[test]
    fn ledger_assigns_numbers() {
        let mut led = PaymentLedger::new();
        let a = led.push_paid("50.00", "a");
        let b = led.push_paid("75.00", "b");
        assert_eq!(a.num, 1);
        assert_eq!(b.num, 2);
        assert_eq!(led.len(), 2);
        assert_eq!(led.total_paid_label(), "Rs 125");
        assert_eq!(led.recent_labels(1), vec!["#002  Rs 75".to_string()]);
    }

    #[test]
    fn parse_paise() {
        assert_eq!(parse_inr_paise("100.00"), 10000);
        assert_eq!(parse_inr_paise("50"), 5000);
        assert_eq!(parse_inr_paise(""), 0);
    }

    #[test]
    fn price_catalog_defaults() {
        let c = PriceCatalog::from_defaults();
        assert_eq!(c.len(), 6);
        assert_eq!(c.get(1), Some("100.00"));
        assert!(c.labels()[0].starts_with("Rs "));
    }
}
