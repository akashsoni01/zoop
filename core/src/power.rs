//! Power rail bring-up sequence — ports `board_power_bsp` ordering.

use crate::io::PowerRails;

/// Expected power-on order from `pala_note` `setup()`.
pub const POWER_ON_SEQUENCE: &[&str] = &[
    "battery_hold_on",
    "epd_power_on",
    "delay_200ms",
    "audio_power_on",
];

/// Power-off / sleep prep order.
pub const POWER_SLEEP_SEQUENCE: &[&str] = &[
    "audio_power_off",
    "epd_power_off",
    "battery_hold_on",
];

/// Run the standard boot power sequence.
pub fn power_on_sequence(power: &mut impl PowerRails) {
    power.battery_hold_on();
    power.epd_power_on();
    power.audio_power_on();
}

/// Run sleep prep sequence (rails except battery hold).
pub fn power_sleep_sequence(power: &mut impl PowerRails) {
    power.audio_power_off();
    power.epd_power_off();
    power.battery_hold_on();
}

/// Verify a logged sequence matches expected order (host tests).
pub fn sequence_matches(log: &[&str], expected: &[&str]) -> bool {
    let mut idx = 0;
    for step in log {
        if idx < expected.len() && *step == expected[idx] {
            idx += 1;
        }
    }
    idx == expected.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::MockPowerRails;

    #[test]
    fn power_on_order_matches_reference() {
        let mut p = MockPowerRails::new();
        power_on_sequence(&mut p);
        assert!(sequence_matches(
            p.logged_sequence(),
            &["battery_hold_on", "epd_power_on", "audio_power_on"]
        ));
    }

    #[test]
    fn sleep_sequence_order() {
        let mut p = MockPowerRails::new();
        power_sleep_sequence(&mut p);
        assert!(sequence_matches(
            p.logged_sequence(),
            &["audio_power_off", "epd_power_off", "battery_hold_on"]
        ));
    }
}
