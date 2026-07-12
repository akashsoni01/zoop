//! Offline dev harness — fullscreen QR home + REC history (no ESP32).

use zoop_core::app::App;
use zoop_core::buttons::ButtonPoller;
use zoop_core::mock::{MockAudio, MockBatteryAdc, MockButtons, MockClock, MockDisplay, MockTime};
use zoop_core::state::AppState;
use zoop_core::storage::MockStorage;

fn advance(
    clock: &mut MockClock,
    buttons: &mut ButtonPoller<MockButtons>,
    app: &mut App<'_, MockStorage, MockDisplay, MockAudio, MockTime, MockBatteryAdc>,
    ms: u64,
) {
    clock.now_ms = ms;
    app.tick(buttons, clock).expect("tick");
}

fn main() {
    let storage = MockStorage::new().expect("temp storage");
    let mut display = MockDisplay::new();
    let mut audio = MockAudio::new();
    let mut buttons = ButtonPoller::new(MockButtons::default());
    let mut clock = MockClock::default();
    let mut time = MockTime {
        utc: Some("2026-07-11T10:00:00Z".to_string()),
    };
    let mut adc = MockBatteryAdc::with_voltage(4.0);

    let mut app = App::new(&storage, &mut display, &mut audio, &mut time, &mut adc);
    app.ledger.push_paid("250.00", "order");
    app.ledger.push_paid("80.00", "tip");

    println!("=== Zoop Pay sim (host) ===");
    app.boot(&clock).expect("boot");
    println!("Boot: {:?} (fullscreen QR home)", app.state.state());

    let mut t = 0u64;

    // REC → history
    buttons.pins_mut().rec = true;
    t += 10;
    advance(&mut clock, &mut buttons, &mut app, t);
    t += 10;
    advance(&mut clock, &mut buttons, &mut app, t);
    buttons.pins_mut().rec = false;
    for _ in 0..15 {
        t += 50;
        advance(&mut clock, &mut buttons, &mut app, t);
        if app.state.state() == AppState::History {
            break;
        }
    }
    println!(
        "After REC: {:?}, total={}",
        app.state.state(),
        app.ledger.total_paid_label()
    );

    // REC → back home
    buttons.pins_mut().rec = true;
    t += 10;
    advance(&mut clock, &mut buttons, &mut app, t);
    buttons.pins_mut().rec = false;
    for _ in 0..15 {
        t += 50;
        advance(&mut clock, &mut buttons, &mut app, t);
        if app.state.state() == AppState::Idle {
            break;
        }
    }
    println!("Back home: {:?}", app.state.state());
    println!("Display flushes: {}", display.flush_count);
}
