use pvz_core::{Event, GameStatus, PlantKind};

#[path = "../benches/support/day_scenario.rs"]
mod day_scenario;

#[test]
fn 基准预热成功种植45株且100只是未来波次总量() {
    let state = day_scenario::prepared_day();
    let board = state.board().unwrap();
    assert_eq!(state.tick(), 45);
    assert_eq!(state.sun(), 5_750);
    assert_eq!(board.plants().len(), 45);
    assert_eq!(
        board
            .plants()
            .iter()
            .filter(|p| p.kind == PlantKind::Sunflower)
            .count(),
        5
    );
    assert_eq!(
        board
            .plants()
            .iter()
            .filter(|p| p.kind == PlantKind::Peashooter)
            .count(),
        40
    );
    assert_eq!(board.rules().wave.len(), 100);
    assert_eq!(board.spawned(), 0);
    assert!(board.zombies().is_empty());
    assert_eq!(board.status(), GameStatus::Running);
}

#[test]
fn 基准1000次调用实际推进1000tick且仍在运行() {
    let mut state = day_scenario::prepared_day();
    assert_eq!(day_scenario::measured_steps(&mut state), 1_000);
    assert_eq!(state.tick(), 1_045);
    assert_eq!(state.board().unwrap().status(), GameStatus::Running);
    assert_eq!(state.board().unwrap().spawned(), 100);
}

#[test]
fn 同一基准场景终局后的调用不再计为推进() {
    let mut state = day_scenario::prepared_day();
    for _ in 0..10_000 {
        if state.board().unwrap().status() != GameStatus::Running {
            break;
        }
        let previous = state.tick();
        let events = state.step(&[]);
        assert_eq!(state.tick(), previous + 1);
        assert!(
            events
                .as_slice()
                .contains(&Event::TickCompleted { tick: previous + 1 })
        );
    }
    assert_eq!(state.board().unwrap().status(), GameStatus::Won);
    assert_eq!(state.board().unwrap().spawned(), 100);
    let before = state.state_digest();
    assert_eq!(day_scenario::measured_steps(&mut state), 0);
    assert_eq!(state.state_digest(), before);
    assert!(state.step(&[]).as_slice().is_empty());
}
