use pvz_core::{
    BoardGeometry, Cell, Command, Event, GameState, GameStatus, InitialState, InvalidDayLevel,
    PlantKind, Point, Rejection, SliceRules,
};
use pvz_data::{DayLevelDefinition, DayRulesDefinition, DaySpawnDefinition, LevelId, RulesVersion};
use std::error::Error;

fn definition() -> DayLevelDefinition {
    serde_json::from_str(include_str!("../../../content/levels/development-day.json")).unwrap()
}

fn assembled(definition: &DayLevelDefinition) -> GameState {
    GameState::from_day_definition(definition).unwrap()
}

fn plant(kind: PlantKind, cell: Cell) -> [Command; 2] {
    [Command::SelectCard { kind }, Command::Plant { cell }]
}

#[test]
fn 默认定义装配与独立开发工厂的初始完整合同相同() {
    let state = assembled(&definition());
    let old = GameState::day(
        7,
        InitialState::new(LevelId::new("development-day-slice").unwrap(), 150),
        SliceRules::development(),
    )
    .unwrap();
    assert_eq!(state.state_digest(), old.state_digest());
    assert_eq!(state.board().unwrap().rules(), &SliceRules::development());
}

#[test]
fn 严格只接受规则二点零且错误具有中文诊断() {
    for (major, minor) in [(1, 0), (2, 1), (3, 0), (u16::MAX, u16::MAX)] {
        let mut content = definition();
        content.rules_version = RulesVersion::new(major, minor).unwrap();
        let error = GameState::from_day_definition(&content).unwrap_err();
        assert_eq!(
            error,
            InvalidDayLevel::UnsupportedRulesVersion(content.rules_version)
        );
        assert!(error.to_string().contains("规则版本"));
        assert!(error.source().is_none());
    }
}

#[test]
fn 频率闭区间边界与当前初始时钟重开合同() {
    for frequency in [0, 1001, u32::MAX] {
        let mut content = definition();
        content.ticks_per_second = frequency;
        let error = GameState::from_day_definition(&content).unwrap_err();
        assert_eq!(error, InvalidDayLevel::InvalidTickRate(frequency));
        assert!(error.to_string().contains("1..=1000"));
        assert!(error.source().is_none());
    }
    for frequency in [1, 30, 60, 120, 1000] {
        let mut content = definition();
        content.ticks_per_second = frequency;
        let mut state = assembled(&content);
        let original = state.state_digest();
        assert_eq!(state.clock().ticks_per_second(), frequency);
        for _ in 0..11 {
            state.step(&[]);
        }
        assert_eq!(state.tick(), 11);
        assert_eq!(state.clock().ticks_per_second(), frequency);
        state.step(&[Command::Restart]);
        assert_eq!(state.tick(), 0);
        assert_eq!(state.state_digest(), original);
    }
}

#[test]
fn 无效规则由既有校验拒绝并保留错误原因() {
    let mut content = definition();
    content.rules.pea_speed = 0;
    let error = GameState::from_day_definition(&content).unwrap_err();
    assert!(matches!(error, InvalidDayLevel::InvalidRules(_)));
    assert!(error.to_string().contains("规则"));
    assert!(error.source().is_some());
}

#[test]
fn 默认定义六千次调用逐步事件摘要兼容并实测推进() {
    let mut new = assembled(&definition());
    let mut old = GameState::day(
        7,
        InitialState::new(LevelId::new("development-day-slice").unwrap(), 150),
        SliceRules::development(),
    )
    .unwrap();
    let initial = new.state_digest();
    let mut advanced = 0;
    let mut paused_calls = 0;
    let mut planted = 0;
    let mut spawned = 0;
    let mut collected = 0;
    for call in 0..6000 {
        let phase = if call < 3000 {
            "重开前"
        } else {
            "重开后"
        };
        let mut commands = match call {
            0 | 3001 => plant(PlantKind::Peashooter, Cell { row: 2, column: 0 }).to_vec(),
            181 | 3182 => plant(PlantKind::Sunflower, Cell { row: 4, column: 0 }).to_vec(),
            100 => vec![Command::Pause],
            110 => vec![Command::Resume],
            3000 => vec![Command::Restart],
            _ => vec![],
        };
        if call != 3000 {
            commands.extend(
                old.board()
                    .unwrap()
                    .suns()
                    .iter()
                    .map(|sun| Command::CollectSun { id: sun.id }),
            );
        }
        let before = new.tick();
        let new_events = new.step(&commands);
        let old_events = old.step(&commands);
        assert_eq!(new_events, old_events, "{phase}：第 {call} 次调用的事件");
        assert!(
            new.state_digest_matches(&old),
            "{phase}：第 {call} 次调用的摘要"
        );
        if call == 3000 {
            assert_eq!(new.state_digest(), initial);
        } else if (100..110).contains(&call) {
            paused_calls += 1;
            assert_eq!(new.tick(), before);
            assert!(new.board().unwrap().paused());
        } else {
            assert_eq!(new.tick(), before + 1, "{phase}必须实际推进");
            advanced += 1;
        }
        for event in new_events.as_slice() {
            match event {
                Event::Planted { .. } => planted += 1,
                Event::ZombieSpawned { .. } => spawned += 1,
                Event::SunCollected { .. } => collected += 1,
                _ => {}
            }
        }
        assert_eq!(new.board().unwrap().status(), GameStatus::Running);
    }
    assert_eq!(advanced, 5989);
    assert_eq!(paused_calls, 10);
    assert_eq!(new.tick(), 2999);
    assert_eq!(planted, 4);
    assert_eq!(spawned, 14);
    assert!(collected > 20);
}

#[test]
fn 二十三标量逐项映射且各自进入摘要与重开合同() {
    let baseline = assembled(&definition()).state_digest();
    macro_rules! check {
        ($field:ident, $value:expr) => {{
            let mut content = definition();
            content.rules.$field = $value;
            let mut expected = SliceRules::development();
            expected.$field = $value;
            let mut state = assembled(&content);
            assert_eq!(
                state.board().unwrap().rules(),
                &expected,
                stringify!($field)
            );
            let original = state.state_digest();
            assert_ne!(original, baseline, stringify!($field));
            state.step(&plant(PlantKind::Sunflower, Cell { row: 0, column: 0 }));
            state.step(&[Command::Pause]);
            state.step(&[Command::Restart]);
            assert_eq!(state.state_digest(), original, stringify!($field));
        }};
    }
    check!(plant_health, 311);
    check!(zombie_health, 212);
    check!(pea_damage, 23);
    check!(bite_damage, 24);
    check!(zombie_speed, 25);
    check!(pea_speed, 606);
    check!(mower_speed, 1207);
    check!(plant_half_width, 2408);
    check!(zombie_half_width, 2009);
    check!(pea_radius, 710);
    check!(mower_half_width, 3011);
    check!(sun_radius, 3012);
    check!(mower_trigger_x, 6013);
    check!(defeat_x, -14);
    check!(sunflower_cost, 65);
    check!(peashooter_cost, 116);
    check!(card_cooldown, 197);
    check!(shoot_interval, 108);
    check!(bite_interval, 49);
    check!(sunflower_interval, 620);
    check!(sky_sun_interval, 321);
    check!(sun_lifetime, 922);
    check!(sun_amount, 48);
}

fn quiet_definition() -> DayLevelDefinition {
    let mut content = definition();
    content.rules.wave = vec![DaySpawnDefinition {
        at: 100_000,
        row: 0,
        x: 79_000,
    }];
    content
}

#[test]
fn 初始关卡阳光种子频率分别影响合同且种子实际控制阳光位置() {
    let original = assembled(&definition()).state_digest();
    for change in [
        (|d: &mut DayLevelDefinition| d.level = LevelId::new("content-day").unwrap())
            as fn(&mut DayLevelDefinition),
        |d| d.initial_sun = 73,
        |d| d.seed = 123,
        |d| d.ticks_per_second = 30,
    ] {
        let mut content = definition();
        change(&mut content);
        let mut state = assembled(&content);
        let initial = state.state_digest();
        assert_ne!(initial, original);
        assert_eq!(state.level(), &content.level);
        assert_eq!(state.sun(), content.initial_sun);
        assert_eq!(state.clock().ticks_per_second(), content.ticks_per_second);
        state.step(&plant(PlantKind::Sunflower, Cell { row: 0, column: 0 }));
        state.step(&[Command::Restart]);
        assert_eq!(state.state_digest(), initial);
    }
    let mut content = quiet_definition();
    content.rules.sky_sun_interval = 1;
    let mut first = assembled(&content);
    content.seed = 123;
    let mut second = assembled(&content);
    first.step(&[]);
    second.step(&[]);
    assert_ne!(
        first.board().unwrap().suns()[0].position,
        second.board().unwrap().suns()[0].position
    );
    assert_ne!(
        first.state_digest().rng_state(),
        second.state_digest().rng_state()
    );
    let events = second.step(&[Command::Restart]);
    assert_eq!(events.as_slice(), &[Event::Restarted]);
    first.step(&[Command::Restart]);
    let first_events = first.step(&[]);
    assert_eq!(
        second.step(&[]).as_slice().len(),
        first_events.as_slice().len()
    );
    assert_ne!(
        first.board().unwrap().suns()[0].position,
        second.board().unwrap().suns()[0].position
    );
}

#[test]
fn 内容费用生命冷却与向日葵间隔实际生效() {
    let mut content = quiet_definition();
    content.initial_sun = 100;
    content.rules.plant_health = 91;
    content.rules.sunflower_cost = 13;
    content.rules.peashooter_cost = 17;
    content.rules.card_cooldown = 3;
    content.rules.sunflower_interval = 2;
    let mut state = assembled(&content);
    state.step(&plant(PlantKind::Sunflower, Cell { row: 0, column: 0 }));
    assert_eq!(state.sun(), 87);
    assert_eq!(state.board().unwrap().plants()[0].health, 91);
    assert!(state.board().unwrap().suns().is_empty());
    state.step(&[]);
    assert_eq!(
        state.board().unwrap().suns()[0].position,
        Cell { row: 0, column: 0 }.center()
    );
    let rejected = state.step(&[Command::Plant {
        cell: Cell { row: 0, column: 1 },
    }]);
    assert!(rejected.as_slice().contains(&Event::CommandRejected {
        reason: Rejection::CoolingDown
    }));
    state.step(&[Command::Plant {
        cell: Cell { row: 0, column: 1 },
    }]);
    assert_eq!(state.sun(), 74);
    assert_eq!(state.board().unwrap().plants().len(), 2);
    state.step(&plant(PlantKind::Peashooter, Cell { row: 1, column: 0 }));
    assert_eq!(state.sun(), 57);
    assert_eq!(state.board().unwrap().plants()[2].health, 91);
}

#[test]
fn 阳光间隔寿命数额与命中半径实际生效() {
    let mut content = quiet_definition();
    content.initial_sun = 3;
    content.rules.sky_sun_interval = 2;
    content.rules.sun_lifetime = 2;
    content.rules.sun_amount = 9;
    content.rules.sun_radius = 5;
    let mut state = assembled(&content);
    state.step(&[]);
    assert!(state.board().unwrap().suns().is_empty());
    state.step(&[]);
    let sun = state.board().unwrap().suns()[0].clone();
    assert_eq!(sun.amount, 9);
    let position = sun.position;
    assert_eq!(
        state.board().unwrap().sun_at(Point {
            x: position.x + 3,
            y: position.y + 4
        }),
        Some(sun.id)
    );
    assert_eq!(
        state.board().unwrap().sun_at(Point {
            x: position.x + 4,
            y: position.y + 4
        }),
        None
    );
    state.step(&[Command::CollectSun { id: sun.id }]);
    assert_eq!(state.sun(), 12);
    state.step(&[]);
    let id = state.board().unwrap().suns()[0].id;
    state.step(&[]);
    assert_eq!(state.board().unwrap().suns().len(), 1);
    let events = state.step(&[]);
    assert!(events.as_slice().contains(&Event::SunExpired { id }));
    assert_eq!(state.board().unwrap().suns().len(), 1);
}

#[test]
fn 豌豆伤害射击间隔速度与碰撞半径实际生效() {
    let mut content = quiet_definition();
    content.rules.shoot_interval = 2;
    content.rules.pea_speed = 100;
    content.rules.pea_damage = 7;
    content.rules.zombie_health = 53;
    content.rules.zombie_speed = 1;
    content.rules.zombie_half_width = 20;
    content.rules.pea_radius = 30;
    content.rules.plant_half_width = 1;
    content.rules.wave[0] = DaySpawnDefinition {
        at: 1,
        row: 0,
        x: 12_149,
    };
    let mut state = assembled(&content);
    let first = state.step(&plant(PlantKind::Peashooter, Cell { row: 0, column: 0 }));
    assert!(
        !first
            .as_slice()
            .iter()
            .any(|e| matches!(e, Event::ProjectileFired { .. }))
    );
    assert_eq!(state.board().unwrap().zombies()[0].health, 53);
    assert_eq!(state.board().unwrap().zombies()[0].x, 12_148);
    let second = state.step(&[]);
    assert!(
        second
            .as_slice()
            .contains(&Event::ProjectileHit { id: 3, zombie: 2 })
    );
    assert_eq!(state.board().unwrap().zombies()[0].health, 46);
    assert!(state.board().unwrap().peas().is_empty());
    assert!(
        !state
            .step(&[])
            .as_slice()
            .iter()
            .any(|e| matches!(e, Event::ProjectileFired { .. }))
    );
    assert!(
        state
            .step(&[])
            .as_slice()
            .iter()
            .any(|e| matches!(e, Event::ProjectileFired { .. }))
    );
    assert_eq!(state.board().unwrap().zombies()[0].health, 39);

    content.rules.wave[0].x = 20_000;
    let mut distant = assembled(&content);
    distant.step(&plant(PlantKind::Peashooter, Cell { row: 0, column: 0 }));
    distant.step(&[]);
    assert_eq!(distant.board().unwrap().peas()[0].x, 12_100);
    distant.step(&[]);
    assert_eq!(distant.board().unwrap().peas()[0].x, 12_200);
}

#[test]
fn 啃食伤害间隔与植物僵尸半宽实际生效() {
    let mut content = quiet_definition();
    content.rules.plant_health = 19;
    content.rules.bite_damage = 7;
    content.rules.bite_interval = 2;
    content.rules.zombie_speed = 11;
    content.rules.plant_half_width = 123;
    content.rules.zombie_half_width = 456;
    content.rules.wave[0] = DaySpawnDefinition {
        at: 1,
        row: 0,
        x: 12_590,
    };
    let mut state = assembled(&content);
    state.step(&plant(PlantKind::Sunflower, Cell { row: 0, column: 0 }));
    assert_eq!(state.board().unwrap().plants()[0].health, 12);
    state.step(&[]);
    assert_eq!(state.board().unwrap().plants()[0].health, 12);
    state.step(&[]);
    assert_eq!(state.board().unwrap().plants()[0].health, 5);
    state.step(&[]);
    let events = state.step(&[]);
    assert!(events.as_slice().contains(&Event::PlantRemoved { id: 1 }));
    assert!(state.board().unwrap().plants().is_empty());

    content.rules.wave[0].x = 12_591;
    let mut outside = assembled(&content);
    outside.step(&plant(PlantKind::Sunflower, Cell { row: 0, column: 0 }));
    assert_eq!(outside.board().unwrap().plants()[0].health, 19);
    assert_eq!(outside.board().unwrap().zombies()[0].x, 12_580);
}

#[test]
fn 同tick波次不排序不去重且实体事件按输入顺序生成() {
    let mut content = quiet_definition();
    content.rules.zombie_speed = 7;
    content.rules.wave = vec![
        DaySpawnDefinition {
            at: 2,
            row: 4,
            x: 30_000,
        },
        DaySpawnDefinition {
            at: 2,
            row: 0,
            x: 40_000,
        },
        DaySpawnDefinition {
            at: 2,
            row: 4,
            x: 30_000,
        },
    ];
    let mut state = assembled(&content);
    state.step(&[]);
    assert!(state.board().unwrap().zombies().is_empty());
    let events = state.step(&[]);
    assert_eq!(
        events.as_slice(),
        &[
            Event::TickStarted { tick: 2 },
            Event::ZombieSpawned { id: 1, row: 4 },
            Event::ZombieSpawned { id: 2, row: 0 },
            Event::ZombieSpawned { id: 3, row: 4 },
            Event::TickCompleted { tick: 2 },
        ]
    );
    let actual: Vec<_> = state
        .board()
        .unwrap()
        .zombies()
        .iter()
        .map(|z| (z.id, z.row, z.x))
        .collect();
    assert_eq!(actual, [(1, 4, 29_993), (2, 0, 39_993), (3, 4, 29_993)]);
    assert_eq!(state.board().unwrap().spawned(), 3);
}

#[test]
fn 割草机触发位置速度半宽与失败线实际生效() {
    let mut content = quiet_definition();
    content.rules.mower_trigger_x = 5000;
    content.rules.defeat_x = -100;
    content.rules.zombie_speed = 10;
    content.rules.mower_speed = 100;
    content.rules.mower_half_width = 20;
    content.rules.zombie_half_width = 30;
    content.rules.wave = vec![
        DaySpawnDefinition {
            at: 1,
            row: 0,
            x: 5011,
        },
        DaySpawnDefinition {
            at: 2,
            row: 0,
            x: 5260,
        },
        DaySpawnDefinition {
            at: 1000,
            row: 1,
            x: 79_000,
        },
    ];
    let mut state = assembled(&content);
    assert_eq!(state.board().unwrap().mowers()[0].x, 5000);
    assert!(
        !state
            .step(&[])
            .as_slice()
            .contains(&Event::MowerStarted { row: 0 })
    );
    let events = state.step(&[]);
    assert!(events.as_slice().contains(&Event::MowerStarted { row: 0 }));
    assert_eq!(state.board().unwrap().mowers()[0].x, 5100);
    assert_eq!(state.board().unwrap().zombies()[0].x, 5250);
    let events = state.step(&[]);
    assert!(events.as_slice().contains(&Event::ZombieDied { id: 2 }));
    assert_eq!(state.board().unwrap().mowers()[0].x, 5200);

    content.rules.zombie_speed = 5200;
    content.rules.mower_speed = 100_000;
    content.rules.wave = vec![
        DaySpawnDefinition {
            at: 1,
            row: 0,
            x: 5000,
        },
        DaySpawnDefinition {
            at: 2,
            row: 0,
            x: 5101,
        },
    ];
    let mut defeated = assembled(&content);
    defeated.step(&[]);
    assert_eq!(defeated.board().unwrap().status(), GameStatus::Running);
    assert!(!defeated.step(&[]).as_slice().contains(&Event::Defeat));
    assert_eq!(defeated.board().unwrap().zombies()[0].x, -99);
    assert!(defeated.step(&[]).as_slice().contains(&Event::Defeat));
    assert_eq!(defeated.board().unwrap().status(), GameStatus::Lost);
}

fn rejects_rules(content: &DayLevelDefinition) {
    assert!(matches!(
        GameState::from_day_definition(content),
        Err(InvalidDayLevel::InvalidRules(_))
    ));
}

#[test]
fn 各类数值与波次的非法边界均透传既有规则拒绝() {
    macro_rules! reject {
        ($field:ident, $values:expr) => {
            for value in $values {
                let mut content = definition();
                content.rules.$field = value;
                rejects_rules(&content);
            }
        };
    }
    reject!(plant_health, [0]);
    reject!(zombie_health, [0]);
    reject!(pea_damage, [0]);
    reject!(bite_damage, [0]);
    reject!(sun_amount, [0]);
    reject!(card_cooldown, [0]);
    reject!(shoot_interval, [0]);
    reject!(bite_interval, [0]);
    reject!(sunflower_interval, [0]);
    reject!(sky_sun_interval, [0]);
    reject!(sun_lifetime, [0]);
    reject!(zombie_speed, [i32::MIN, -1, 0, 100_001, i32::MAX]);
    reject!(pea_speed, [-1, 0, 100_001]);
    reject!(mower_speed, [-1, 0, 100_001]);
    reject!(plant_half_width, [-1, 0, 100_001]);
    reject!(zombie_half_width, [-1, 0, 100_001]);
    reject!(pea_radius, [-1, 0, 100_001]);
    reject!(mower_half_width, [-1, 0, 100_001]);
    reject!(sun_radius, [-1, 0, 100_001]);
    reject!(defeat_x, [-100_001, i32::MIN, 6000]);
    reject!(mower_trigger_x, [0, BoardGeometry::LEFT, i32::MAX]);
    let invalid_waves = [
        vec![],
        vec![
            DaySpawnDefinition {
                at: 1,
                row: 0,
                x: 79_000
            };
            10_001
        ],
        vec![DaySpawnDefinition {
            at: 0,
            row: 0,
            x: 79_000,
        }],
        vec![DaySpawnDefinition {
            at: 1,
            row: 5,
            x: 79_000,
        }],
        vec![DaySpawnDefinition {
            at: 1,
            row: u8::MAX,
            x: 79_000,
        }],
        vec![DaySpawnDefinition {
            at: 1,
            row: 0,
            x: 5999,
        }],
        vec![DaySpawnDefinition {
            at: 1,
            row: 0,
            x: BoardGeometry::RIGHT + 1,
        }],
        vec![
            DaySpawnDefinition {
                at: 2,
                row: 0,
                x: 79_000,
            },
            DaySpawnDefinition {
                at: 1,
                row: 0,
                x: 79_000,
            },
        ],
    ];
    for wave in invalid_waves {
        let mut content = definition();
        content.rules.wave = wave;
        rejects_rules(&content);
    }
}

#[test]
fn 装配不擅自缩窄既有规则合法边界() {
    let mut content = definition();
    let rules: &mut DayRulesDefinition = &mut content.rules;
    rules.plant_health = u32::MAX;
    rules.zombie_health = u32::MAX;
    rules.pea_damage = u32::MAX;
    rules.bite_damage = u32::MAX;
    rules.sun_amount = u32::MAX;
    rules.sunflower_cost = 0;
    rules.peashooter_cost = u32::MAX;
    rules.card_cooldown = u64::MAX;
    rules.shoot_interval = u64::MAX;
    rules.bite_interval = u64::MAX;
    rules.sunflower_interval = u64::MAX;
    rules.sky_sun_interval = u64::MAX;
    rules.sun_lifetime = u64::MAX;
    rules.zombie_speed = 1;
    rules.pea_speed = 100_000;
    rules.mower_speed = 1;
    rules.plant_half_width = 100_000;
    rules.zombie_half_width = 1;
    rules.pea_radius = 100_000;
    rules.mower_half_width = 1;
    rules.sun_radius = 100_000;
    rules.defeat_x = -100_000;
    rules.mower_trigger_x = BoardGeometry::LEFT - 1;
    rules.wave = vec![
        DaySpawnDefinition {
            at: u64::MAX,
            row: 4,
            x: BoardGeometry::RIGHT
        };
        10_000
    ];
    assert!(GameState::from_day_definition(&content).is_ok());
    content.rules.wave = vec![DaySpawnDefinition {
        at: 1,
        row: 0,
        x: 6000,
    }];
    content.rules.mower_trigger_x = 6000;
    assert!(GameState::from_day_definition(&content).is_ok());
}
