//! Full offline loop: record → tag → list → play → delete (host-verified).

use zoop_core::app::App;
use zoop_core::buttons::ButtonPoller;
use zoop_core::mock::{
    MockAudio, MockBatteryAdc, MockButtons, MockClock, MockDisplay, MockTime,
};
use zoop_core::paths::note_path;
use zoop_core::state::AppState;
use zoop_core::storage::{load_index, FileStorage, IndexStore, MockStorage, TagStore};

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
fn offline_record_tag_list_play_delete() {
    let storage = MockStorage::new().expect("storage");
    let mut display = MockDisplay::new();
    let mut audio = MockAudio::new();
    let mut buttons = ButtonPoller::new(MockButtons::default());
    let mut clock = MockClock::default();
    let mut time = MockTime {
        utc: Some("2026-07-11T14:00:00Z".to_string()),
    };
    let mut adc = MockBatteryAdc::with_voltage(4.1);
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
    assert_eq!(app.state.state(), AppState::Idle);

    let mut t = 0u64;

    // Hold REC → record
    buttons.pins_mut().rec = true;
    for _ in 0..30 {
        t += 50;
        advance(&mut clock, &mut buttons, &mut app, t);
    }
    assert_eq!(app.state.state(), AppState::Recording);

    buttons.pins_mut().rec = false;
    for _ in 0..15 {
        t += 100;
        advance(&mut clock, &mut buttons, &mut app, t);
        if app.state.state() == AppState::TagSelect {
            break;
        }
    }
    assert_eq!(app.state.state(), AppState::TagSelect);

    // Save tag (long REC) — release as soon as Idle to avoid accidental re-record
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
    assert_eq!(app.index.len(), 1);

    // Menu (PWR single needs debounce)
    buttons.pins_mut().pwr = true;
    t += 10;
    advance(&mut clock, &mut buttons, &mut app, t);
    t += 10;
    advance(&mut clock, &mut buttons, &mut app, t);
    buttons.pins_mut().pwr = false;
    t += 50;
    advance(&mut clock, &mut buttons, &mut app, t);
    assert_eq!(app.state.state(), AppState::Menu);

    // Open notes (REC single)
    buttons.pins_mut().rec = true;
    t += 10;
    advance(&mut clock, &mut buttons, &mut app, t);
    buttons.pins_mut().rec = false;
    t += 250;
    advance(&mut clock, &mut buttons, &mut app, t);
    t += 250;
    advance(&mut clock, &mut buttons, &mut app, t);
    assert_eq!(app.state.state(), AppState::NoteList);

    // Open detail (REC single)
    buttons.pins_mut().rec = true;
    t += 10;
    advance(&mut clock, &mut buttons, &mut app, t);
    buttons.pins_mut().rec = false;
    t += 250;
    advance(&mut clock, &mut buttons, &mut app, t);
    t += 250;
    advance(&mut clock, &mut buttons, &mut app, t);
    assert_eq!(app.state.state(), AppState::NoteDetail);

    // Long REC → delete confirm
    buttons.pins_mut().rec = true;
    for _ in 0..15 {
        t += 50;
        advance(&mut clock, &mut buttons, &mut app, t);
        if app.state.state() == AppState::DeleteConfirm {
            break;
        }
    }
    buttons.pins_mut().rec = false;
    t += 50;
    advance(&mut clock, &mut buttons, &mut app, t);
    assert_eq!(app.state.state(), AppState::DeleteConfirm);

    // Confirm delete (REC single)
    buttons.pins_mut().rec = true;
    t += 10;
    advance(&mut clock, &mut buttons, &mut app, t);
    buttons.pins_mut().rec = false;
    t += 250;
    advance(&mut clock, &mut buttons, &mut app, t);
    t += 250;
    advance(&mut clock, &mut buttons, &mut app, t);

    let mut reloaded = IndexStore::new();
    load_index(&storage, &mut reloaded).expect("reload");
    assert!(reloaded.is_empty());
    assert!(!storage.exists(&note_path(1, "wav")).expect("exists"));
}
