//! Battery voltage → percent curve — ports `battery.cpp`.

/// Piecewise voltage curve matching `pala_note`.
pub struct BatteryCurve;

impl BatteryCurve {
    pub const LOW_THRESHOLD: u8 = 15;
    pub const RECOVER_THRESHOLD: u8 = 20;
}

/// Convert battery voltage (V) to percent (0–100). Returns -1 equivalent via `None` for invalid.
pub fn battery_percent_from_voltage(v: f32) -> Option<u8> {
    if v <= 0.1 {
        return None;
    }
    if v >= 4.35 {
        return Some(100);
    }
    if v <= 3.20 {
        return Some(0);
    }
    if v >= 4.20 {
        return Some(100);
    }

    const VOLTS: [f32; 5] = [3.20, 3.40, 3.70, 3.90, 4.20];
    const PCT: [i32; 5] = [0, 25, 50, 75, 100];

    for i in 1..5 {
        if v <= VOLTS[i] {
            let t = (v - VOLTS[i - 1]) / (VOLTS[i] - VOLTS[i - 1]);
            let mut p = PCT[i - 1] + ((PCT[i] - PCT[i - 1]) as f32 * t + 0.5) as i32;
            p = ((p + 2) / 5) * 5;
            return Some(p.clamp(0, 100) as u8);
        }
    }
    Some(100)
}

/// Apply ADC divider: raw millivolts → battery voltage (×2).
pub fn voltage_from_adc_mv(mv: f32) -> f32 {
    (mv / 1000.0) * 2.0
}

/// Average ADC millivolt samples and convert to percent.
pub fn battery_percent_from_adc_samples(samples: &[u32]) -> Option<u8> {
    if samples.is_empty() {
        return None;
    }
    let sum: u64 = samples.iter().map(|&s| s as u64).sum();
    let avg = sum as f32 / samples.len() as f32;
    battery_percent_from_voltage(voltage_from_adc_mv(avg))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_charge_returns_100() {
        assert_eq!(battery_percent_from_voltage(4.25), Some(100));
        assert_eq!(battery_percent_from_voltage(4.35), Some(100));
    }

    #[test]
    fn empty_returns_0() {
        assert_eq!(battery_percent_from_voltage(3.10), Some(0));
        assert_eq!(battery_percent_from_voltage(3.20), Some(0));
    }

    #[test]
    fn invalid_voltage_returns_none() {
        assert_eq!(battery_percent_from_voltage(0.05), None);
    }

    #[test]
    fn midpoint_interpolation_rounds_to_5() {
        let p = battery_percent_from_voltage(3.55).expect("percent");
        assert_eq!(p % 5, 0);
        assert!(p >= 25 && p <= 50);
    }

    #[test]
    fn adc_samples_match_voltage_path() {
        let samples = [2100u32; 16]; // 2.1V ADC → 4.2V battery
        assert_eq!(battery_percent_from_adc_samples(&samples), Some(100));
    }
}
