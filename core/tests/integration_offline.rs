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

    buttons.pins_mut().rec = true;
    for t in (0..=500).step_by(50) {
        advance(&mut clock, &mut buttons, &mut app, t);
    }
    assert_eq!(app.state.state(), AppState::Recording);

    buttons.pins_mut().rec = false;
    for t in (600..=2000).step_by(200) {
        advance(&mut clock, &mut buttons, &mut app, t);
        if app.state.state() == AppState::TagSelect {
            break;
        }
    }
    assert_eq!(app.state.state(), AppState::TagSelect);

    buttons.pins_mut().rec = true;
    advance(&mut clock, &mut buttons, &mut app, 2100);
    buttons.pins_mut().rec = false;
    advance(&mut clock, &mut buttons, &mut app, 2200);
    assert_eq!(app.state.state(), AppState::Idle);
    assert_eq!(app.index.len(), 1);

    buttons.pins_mut().pwr = true;
    advance(&mut clock, &mut buttons, &mut app, 2300);
    buttons.pins_mut().pwr = false;
    advance(&mut clock, &mut buttons, &mut app, 2320);
    assert_eq!(app.state.state(), AppState::Menu);

    buttons.pins_mut().rec = true;
    advance(&mut clock, &mut buttons, &mut app, 2400);
    buttons.pins_mut().rec = false;
    advance(&mut clock, &mut buttons, &mut app, 2420);
    assert_eq!(app.state.state(), AppState::NoteList);

    buttons.pins_mut().rec = true;
    advance(&mut clock, &mut buttons, &mut app, 2500);
    buttons.pins_mut().rec = false;
    advance(&mut clock, &mut buttons, &mut app, 2520);
    assert_eq!(app.state.state(), AppState::NoteDetail);

    buttons.pins_mut().rec = true;
    advance(&mut clock, &mut buttons, &mut app, 2600);

    buttons.pins_mut().rec = false;
    advance(&mut clock, &mut buttons, &mut app, 2620);

    buttons.pins_mut().rec = true;
    for t in (2700..=2900).step_by(50) {
        advance(&mut clock, &mut buttons, &mut app, t);
        if app.state.state() == AppState::DeleteConfirm {
            break;
        }
    }
    buttons.pins_mut().rec = false;
    advance(&mut clock, &mut buttons, &mut app, 3000);

    if app.state.state() == AppState::DeleteConfirm {
        buttons.pins_mut().rec = true;
        advance(&mut clock, &mut buttons, &mut app, 3100);
        buttons.pins_mut().rec = false;
        advance(&mut clock, &mut buttons, &mut app, 3120);
    }

    let mut reloaded = IndexStore::new();
    load_index(&storage, &mut reloaded).expect("reload");
    assert!(reloaded.is_empty());
    assert!(!storage.exists(&note_path(1, "wav")).expect("exists"));
}
