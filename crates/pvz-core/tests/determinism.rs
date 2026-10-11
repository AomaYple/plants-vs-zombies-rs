use pvz_core::{
    Command, DeterministicRng, Event, GameState, InitialState, SimulationClock, StateDigest,
};
use pvz_data::LevelId;

fn initial_state() -> InitialState {
    InitialState::new(LevelId::new("day-1").unwrap(), 50)
}

#[test]
fn 相同输入产生相同状态摘要和事件序列() {
    let commands = [
        Command::SpendSun { amount: 25 },
        Command::AddSun { amount: 10 },
    ];
    let mut left = GameState::new(7, initial_state());
    let mut right = GameState::new(7, initial_state());

    let left_events = left.step(&commands);
    let right_events = right.step(&commands);

    assert_eq!(left_events, right_events);
    assert_eq!(left.state_digest(), right.state_digest());
    assert_eq!(left.sun(), 35);
    assert_eq!(left.tick(), 1);
    assert_ne!(left.state_digest(), StateDigest::initial(50));
}

#[test]
fn 命令按规范顺序执行并保留事件顺序() {
    let commands = [
        Command::SpendSun { amount: 25 },
        Command::AddSun { amount: 10 },
    ];
    let mut game = GameState::new(1, initial_state());

    let events = game.step(&commands);

    assert_eq!(events.tick(), 1);
    assert_eq!(
        events.as_slice(),
        &[
            Event::TickStarted { tick: 1 },
            Event::SunAdded {
                amount: 10,
                total: 60
            },
            Event::SunSpent {
                amount: 25,
                total: 35
            },
            Event::TickCompleted { tick: 1 },
        ]
    );
}

#[test]
fn 空棋盘推进仍然推进可替换时钟() {
    assert!(SimulationClock::new(0).is_err());
    let mut clock = SimulationClock::new(30).unwrap();
    assert_eq!(clock.ticks_per_second(), 30);
    assert_eq!(clock.current_tick(), 0);
    assert_eq!(clock.advance(), 1);
    assert_eq!(clock.advance(), 2);

    let mut game = GameState::with_clock(3, initial_state(), clock);
    game.step(&[]);
    assert_eq!(game.tick(), 3);
}

#[test]
fn 无效命令产生明确拒绝事件() {
    let mut game = GameState::new(1, initial_state());
    let events = game.step(&[
        Command::SpendSun { amount: 100 },
        Command::AddSun { amount: u32::MAX },
    ]);

    assert_eq!(
        events.as_slice(),
        &[
            Event::TickStarted { tick: 1 },
            Event::SunAddRejected {
                amount: u32::MAX,
                available: 50,
            },
            Event::SunSpendRejected {
                amount: 100,
                available: 50,
            },
            Event::TickCompleted { tick: 1 },
        ]
    );
    assert_eq!(game.sun(), 50);
}

#[test]
fn rng使用固定向量且不会依赖系统时间() {
    let mut rng = DeterministicRng::new(0x0123_4567_89ab_cdef);
    assert_eq!(rng.next_u64(), 0x7c94_8247_2cb6_708c);
    assert_eq!(rng.next_u64(), 0xd570_5692_bf1f_28de);
}

#[test]
fn 零种子也能推进确定性状态() {
    let mut rng = DeterministicRng::new(0);
    assert_ne!(rng.next_u64(), 0);
    assert_ne!(rng.state(), 0);
}
