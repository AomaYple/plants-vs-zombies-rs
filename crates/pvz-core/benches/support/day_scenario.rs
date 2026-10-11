use pvz_core::{Cell, Command, Event, GameState, InitialState, PlantKind, SliceRules, Spawn};
use pvz_data::LevelId;
use std::hint::black_box;

// 基准和回归共享同一原创场景；此模块不进入库的公开 API。
pub fn prepared_day() -> GameState {
    let mut rules = SliceRules::development();
    rules.card_cooldown = 1;
    rules.wave = (0..100)
        .map(|i| Spawn {
            at: 46 + i * 4,
            row: (i % 5) as u8,
            x: 79_000,
        })
        .collect();
    let mut state = GameState::day(
        7,
        InitialState::new(
            LevelId::new("benchmark-day").expect("原创基准关卡有效"),
            10_000,
        ),
        rules,
    )
    .expect("基准规则有效");
    for row in 0..5 {
        for column in 0..9 {
            let kind = if column == 0 {
                PlantKind::Sunflower
            } else {
                PlantKind::Peashooter
            };
            let cell = Cell { row, column };
            let events = state.step(&[Command::SelectCard { kind }, Command::Plant { cell }]);
            assert!(
                events.as_slice().iter().any(|event| {
                    matches!(event, Event::Planted { kind: actual_kind, cell: actual_cell, .. }
                    if *actual_kind == kind && *actual_cell == cell)
                }),
                "基准预热必须成功种植每株植物"
            );
        }
    }
    state
}

pub fn measured_steps(state: &mut GameState) -> u64 {
    let start = state.tick();
    for _ in 0..1_000 {
        black_box(state.step(black_box(&[])));
    }
    state.tick() - start
}
