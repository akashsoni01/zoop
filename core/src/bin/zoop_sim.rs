//! Offline dev harness — scripted demo against mock BSP (no ESP32).

use zoop_core::app::App;
use zoop_core::buttons::ButtonPoller;
use zoop_core::mock::{
    MockAudio, MockBatteryAdc, MockButtons, MockClock, MockDisplay, MockTime,
};
use zoop_core::state::AppState;
use zoop_core::storage::{IndexStore, MockStorage, TagStore};

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
    let mut index = IndexStore::new();
    let mut tags = TagStore::new();

    let mut app = App::new(
        &storage,
        &mut display,
        &mut audio,
        &mut time,
        &mut adc,
        &mut index,
        &mut tags,
    );

    println!("=== Zoop sim (host) ===");
    app.boot(&clock).expect("boot");
    println!("Boot state: {:?}", app.state.state());

    buttons.pins_mut().pwr = true;
    clock.now_ms = 10;
    app.tick(&mut buttons, &clock).expect("tick");
    buttons.pins_mut().pwr = false;
    clock.now_ms = 20;
    app.tick(&mut buttons, &clock).expect("tick");
    println!("After PWR tap: {:?}", app.state.state());

    buttons.pins_mut().rec = true;
    for t in (100..=500).step_by(50) {
        clock.now_ms = t;
        app.tick(&mut buttons, &clock).expect("tick");
    }
    buttons.pins_mut().rec = false;
    for t in (600..=2000).step_by(200) {
        clock.now_ms = t;
        app.tick(&mut buttons, &clock).expect("tick");
        if app.state.state() == AppState::TagSelect {
            break;
        }
    }
    println!("After record: {:?}, notes={}", app.state.state(), app.index.len());

    if app.state.state() == AppState::TagSelect {
        buttons.pins_mut().rec = true;
        clock.now_ms = 2100;
        app.tick(&mut buttons, &clock).expect("tick");
        buttons.pins_mut().rec = false;
        clock.now_ms = 2200;
        app.tick(&mut buttons, &clock).expect("tick");
        println!("After tag save: {:?}, notes={}", app.state.state(), app.index.len());
    }

    println!("Display flushes: {}", display.flush_count);
    println!("Done. Run `cargo test` for full verification.");
}
