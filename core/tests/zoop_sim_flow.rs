//! Regression test for `zoop-sim` scripted record → tag flow.

use zoop_core::app::App;
use zoop_core::buttons::ButtonPoller;
use zoop_core::mock::{MockAudio, MockBatteryAdc, MockButtons, MockClock, MockDisplay, MockTime};
use zoop_core::state::AppState;
use zoop_core::storage::{IndexStore, MockStorage, TagStore};

fn advance(
    clock: &mut MockClock,
    buttons: &mut ButtonPoller<MockButtons>,
    app: &mut App<'_, MockStorage, MockDisplay, MockAudio, MockTime, MockBatteryAdc>,
    ms: u64,
) {
    clock.now_ms = ms;
    app.tick(buttons, clock).expect("tick");
}

#[test]
fn sim_record_hold_and_tag_save() {
    let storage = MockStorage::new().expect("storage");
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
    app.boot(&clock).expect("boot");

    let mut t = 0u64;

    // PWR tap → menu (hold through debounce)
    buttons.pins_mut().pwr = true;
    t += 10;
    advance(&mut clock, &mut buttons, &mut app, t);
    t += 10;
    advance(&mut clock, &mut buttons, &mut app, t);
    buttons.pins_mut().pwr = false;
    t += 50;
    advance(&mut clock, &mut buttons, &mut app, t);
    assert_eq!(app.state.state(), AppState::Menu);

    // Hold REC long enough to start recording and capture ≥500 ms audio
    buttons.pins_mut().rec = true;
    for _ in 0..35 {
        t += 50;
        advance(&mut clock, &mut buttons, &mut app, t);
    }
    assert_eq!(app.state.state(), AppState::Recording);

    buttons.pins_mut().rec = false;
    for _ in 0..12 {
        t += 100;
        advance(&mut clock, &mut buttons, &mut app, t);
        if app.state.state() == AppState::TagSelect {
            break;
        }
    }
    assert_eq!(app.state.state(), AppState::TagSelect);

    // Save tag (hold REC until Idle)
    buttons.pins_mut().rec = true;
    for _ in 0..15 {
        t += 50;
        advance(&mut clock, &mut buttons, &mut app, t);
        if app.state.state() == AppState::Idle {
            buttons.pins_mut().rec = false;
            break;
        }
    }
    if app.state.state() != AppState::Idle {
        buttons.pins_mut().rec = false;
        for _ in 0..10 {
            t += 50;
            advance(&mut clock, &mut buttons, &mut app, t);
        }
    }

    assert_eq!(app.state.state(), AppState::Idle);
    assert_eq!(app.index.len(), 1, "note should be in index after tag save");
}
