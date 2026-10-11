use pvz_core::{Command, DeterministicRng, Event, GameState, InitialState, SimulationClock};
use pvz_data::{LevelId, RulesVersion};

// 按锁定 oorandom 11.1.5 合同独立计算并由下面的固定向量验证。
const RNG_INCREMENT: u128 = 0x5fc1_c2d3_ff7a_0dc6_b786_0f7a_9a5f_029f;
const RNG_SEED7_STATE: (u128, u128) = (0x5557_6224_b31e_24e5_5fc5_b510_b8ce_3d5d, RNG_INCREMENT);

// 关卡、命令和期望事件为原创的最小开发夹具，不是原版行为记录。
fn initial_state(level: &str, sun: u32) -> InitialState {
    InitialState::new(LevelId::new(level).unwrap(), sun)
}

#[test]
fn 先消费后增加会拒绝消费且反序结果不同() {
    let mut spend_first = GameState::new(7, initial_state("test-order", 0));
    let mut add_first = GameState::new(7, initial_state("test-order", 0));
    let spend_events = spend_first.step(&[
        Command::SpendSun { amount: 10 },
        Command::AddSun { amount: 10 },
    ]);
    let add_events = add_first.step(&[
        Command::AddSun { amount: 10 },
        Command::SpendSun { amount: 10 },
    ]);
    assert_eq!(
        spend_events.as_slice(),
        &[
            Event::TickStarted { tick: 1 },
            Event::SunSpendRejected {
                amount: 10,
                available: 0
            },
            Event::SunAdded {
                amount: 10,
                total: 10
            },
            Event::TickCompleted { tick: 1 },
        ]
    );
    assert_eq!(
        add_events.as_slice(),
        &[
            Event::TickStarted { tick: 1 },
            Event::SunAdded {
                amount: 10,
                total: 10
            },
            Event::SunSpent {
                amount: 10,
                total: 0
            },
            Event::TickCompleted { tick: 1 },
        ]
    );
    assert_eq!(spend_first.sun(), 10);
    assert_eq!(add_first.sun(), 0);
    assert_ne!(spend_first.state_digest(), add_first.state_digest());
}

#[test]
fn 同tick同种消费命令保留提交顺序及重复命令() {
    let mut game = GameState::new(7, initial_state("test-order", 10));
    let events = game.step(&[
        Command::SpendSun { amount: 10 },
        Command::SpendSun { amount: 5 },
        Command::SpendSun { amount: 5 },
    ]);
    assert_eq!(
        events.as_slice(),
        &[
            Event::TickStarted { tick: 1 },
            Event::SunSpent {
                amount: 10,
                total: 0
            },
            Event::SunSpendRejected {
                amount: 5,
                available: 0
            },
            Event::SunSpendRejected {
                amount: 5,
                available: 0
            },
            Event::TickCompleted { tick: 1 },
        ]
    );
    assert_eq!(game.sun(), 0);
}

#[test]
fn 同tick同种增加命令保留提交顺序() {
    let mut game = GameState::new(7, initial_state("test-order", u32::MAX - 10));
    let events = game.step(&[
        Command::AddSun { amount: 10 },
        Command::AddSun { amount: 5 },
    ]);
    assert_eq!(
        events.as_slice(),
        &[
            Event::TickStarted { tick: 1 },
            Event::SunAdded {
                amount: 10,
                total: u32::MAX
            },
            Event::SunAddRejected {
                amount: 5,
                available: u32::MAX
            },
            Event::TickCompleted { tick: 1 },
        ]
    );
    assert_eq!(game.sun(), u32::MAX);
}

#[test]
fn 不同关卡必须产生不同摘要() {
    let mut left = GameState::new(7, initial_state("test-level-a", 50));
    let mut right = GameState::new(7, initial_state("test-level-b", 50));
    assert_ne!(left.state_digest(), right.state_digest());
    left.step(&[]);
    right.step(&[]);
    assert_ne!(left.state_digest(), right.state_digest());
}

#[test]
fn 不同时钟频率必须产生不同摘要() {
    let mut left = GameState::with_clock(
        7,
        initial_state("test-clock", 50),
        SimulationClock::new(30).unwrap(),
    );
    let mut right = GameState::with_clock(
        7,
        initial_state("test-clock", 50),
        SimulationClock::new(60).unwrap(),
    );
    assert_ne!(left.state_digest(), right.state_digest());
    left.step(&[]);
    right.step(&[]);
    assert_ne!(left.state_digest(), right.state_digest());
}

#[test]
fn 不同tick种子和阳光必须产生不同摘要() {
    let baseline = GameState::new(7, initial_state("test-digest", 50));
    let other_seed = GameState::new(8, initial_state("test-digest", 50));
    let other_sun = GameState::new(7, initial_state("test-digest", 51));
    let mut later = GameState::new(7, initial_state("test-digest", 50));
    later.step(&[]);
    assert_ne!(baseline.state_digest(), other_seed.state_digest());
    assert_ne!(baseline.state_digest(), other_sun.state_digest());
    assert_ne!(baseline.state_digest(), later.state_digest());
}

#[test]
fn 摘要保留完整初始状态和当前开发规则合同() {
    let mut clock = SimulationClock::new(30).unwrap();
    clock.advance();
    let game = GameState::with_clock(
        u64::MAX,
        initial_state("test-digest-contract", u32::MAX),
        clock,
    );
    let digest = game.state_digest();
    assert_eq!(digest.level().as_str(), "test-digest-contract");
    assert_eq!(digest.clock(), clock);
    assert_eq!(digest.tick(), 1);
    assert_eq!(digest.sun(), u32::MAX);
    assert_eq!(
        digest.rng_state(),
        (0x7dd5_d960_54b8_2e08_4396_b9eb_ba66_8b35, RNG_INCREMENT,)
    );
    assert_eq!(digest.rules_version(), RulesVersion::new(1, 0).unwrap());
}

#[test]
fn 空tick和阳光命令不消耗没有消费者的随机状态() {
    for (seed, expected_state) in [
        (
            0,
            (0x5db0_e700_d4b1_9567_871c_9950_5a33_817a, RNG_INCREMENT),
        ),
        (7, RNG_SEED7_STATE),
        (
            u64::MAX,
            (0x7dd5_d960_54b8_2e08_4396_b9eb_ba66_8b35, RNG_INCREMENT),
        ),
    ] {
        let mut game = GameState::new(seed, initial_state("test-rng", 0));
        let before = game.state_digest();
        assert_eq!(before.rng_state(), expected_state);
        let events = game.step(&[]);
        assert_eq!(
            events.as_slice(),
            &[
                Event::TickStarted { tick: 1 },
                Event::TickCompleted { tick: 1 }
            ]
        );
        game.step(&[Command::AddSun { amount: 1 }]);
        assert_eq!(game.state_digest().rng_state(), expected_state);
        assert_ne!(game.state_digest(), before);
    }
}

#[test]
fn 连续多tick回放逐步核对状态和事件() {
    let steps: &[(&[Command], u32, &[Event])] = &[
        (&[], 0, &[]),
        (
            &[
                Command::SpendSun { amount: 10 },
                Command::AddSun { amount: 10 },
            ],
            10,
            &[
                Event::SunSpendRejected {
                    amount: 10,
                    available: 0,
                },
                Event::SunAdded {
                    amount: 10,
                    total: 10,
                },
            ],
        ),
        (
            &[
                Command::SpendSun { amount: 10 },
                Command::SpendSun { amount: 5 },
            ],
            0,
            &[
                Event::SunSpent {
                    amount: 10,
                    total: 0,
                },
                Event::SunSpendRejected {
                    amount: 5,
                    available: 0,
                },
            ],
        ),
        (
            &[
                Command::AddSun { amount: u32::MAX },
                Command::AddSun { amount: 1 },
            ],
            u32::MAX,
            &[
                Event::SunAdded {
                    amount: u32::MAX,
                    total: u32::MAX,
                },
                Event::SunAddRejected {
                    amount: 1,
                    available: u32::MAX,
                },
            ],
        ),
        (
            &[
                Command::SpendSun { amount: u32::MAX },
                Command::AddSun { amount: 0 },
                Command::SpendSun { amount: 0 },
            ],
            0,
            &[
                Event::SunSpent {
                    amount: u32::MAX,
                    total: 0,
                },
                Event::SunAdded {
                    amount: 0,
                    total: 0,
                },
                Event::SunSpent {
                    amount: 0,
                    total: 0,
                },
            ],
        ),
    ];
    let mut left = GameState::new(7, initial_state("test-replay", 0));
    let mut right = GameState::new(7, initial_state("test-replay", 0));
    for (index, (commands, expected_sun, command_events)) in steps.iter().enumerate() {
        let tick = index as u64 + 1;
        let mut expected_events = vec![Event::TickStarted { tick }];
        expected_events.extend_from_slice(command_events);
        expected_events.push(Event::TickCompleted { tick });
        let left_events = left.step(commands);
        let right_events = right.step(commands);
        assert_eq!(left_events.tick(), tick);
        assert_eq!(left_events.as_slice(), expected_events);
        assert_eq!(left_events, right_events);
        assert_eq!(left.state_digest(), right.state_digest());
        assert_eq!(left.state_digest().tick(), tick);
        assert_eq!(left.state_digest().sun(), *expected_sun);
        assert_eq!(left.state_digest().rng_state(), RNG_SEED7_STATE);
    }
}

#[test]
fn 阳光零与最大值边界不环绕() {
    let mut game = GameState::new(0, initial_state("test-sun-boundary", 0));
    let events = game.step(&[
        Command::SpendSun { amount: 0 },
        Command::SpendSun { amount: 1 },
        Command::AddSun { amount: u32::MAX },
        Command::AddSun { amount: 0 },
        Command::AddSun { amount: 1 },
        Command::SpendSun { amount: u32::MAX },
    ]);
    assert_eq!(
        events.as_slice(),
        &[
            Event::TickStarted { tick: 1 },
            Event::SunSpent {
                amount: 0,
                total: 0
            },
            Event::SunSpendRejected {
                amount: 1,
                available: 0
            },
            Event::SunAdded {
                amount: u32::MAX,
                total: u32::MAX
            },
            Event::SunAdded {
                amount: 0,
                total: u32::MAX
            },
            Event::SunAddRejected {
                amount: 1,
                available: u32::MAX
            },
            Event::SunSpent {
                amount: u32::MAX,
                total: 0
            },
            Event::TickCompleted { tick: 1 },
        ]
    );
    assert_eq!(game.sun(), 0);
}

#[test]
fn 空棋盘推进仍然推进可替换时钟() {
    assert!(SimulationClock::new(0).is_err());
    let mut clock = SimulationClock::new(30).unwrap();
    assert_eq!(clock.advance(), 1);
    assert_eq!(clock.advance(), 2);
    let mut game = GameState::with_clock(3, initial_state("test-clock", 50), clock);
    game.step(&[]);
    assert_eq!(game.tick(), 3);
    assert_eq!(game.clock().ticks_per_second(), 30);
}

#[test]
fn rng使用固定向量且不会依赖系统时间() {
    let mut rng = DeterministicRng::new(0x0123_4567_89ab_cdef);
    assert_eq!(
        rng.state(),
        (0xc52e_5f4f_fc2f_f158_50fd_d8af_18d9_ace5, RNG_INCREMENT)
    );
    assert_eq!(rng.next_u64(), 0xe93a_aba1_64f6_25cb);
    assert_eq!(
        rng.state(),
        (0x8838_276e_d3c1_2c5b_fabc_bcd9_7aaa_aa58, RNG_INCREMENT)
    );
    assert_eq!(rng.next_u64(), 0x2001_72d6_4382_76a9);
    assert_eq!(
        rng.state(),
        (0x4fbd_0bb9_8ccc_21a2_5720_e425_102f_7c57, RNG_INCREMENT)
    );
}

#[test]
fn 零种子也能推进确定性状态() {
    let mut rng = DeterministicRng::new(0);
    assert_eq!(
        rng.state(),
        (0x5db0_e700_d4b1_9567_871c_9950_5a33_817a, RNG_INCREMENT)
    );
    assert_eq!(rng.next_u64(), 0x56b0_90d8_7381_1c9b);
    assert_eq!(
        rng.state(),
        (0xa38d_b701_1cca_ba49_fde6_50ee_5be4_2481, RNG_INCREMENT)
    );
}
