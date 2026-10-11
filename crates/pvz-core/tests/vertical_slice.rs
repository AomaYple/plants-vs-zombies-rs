use pvz_core::{
    BoardGeometry, Cell, Event, GameStatus, PlantKind, Point, Rejection, SliceRules, Spawn,
};
use pvz_core::{Command, GameState, InitialState};
use pvz_data::{LevelId, RulesVersion};

fn game(rules: SliceRules, sun: u32) -> GameState {
    GameState::day(
        7,
        InitialState::new(LevelId::new("day-test").unwrap(), sun),
        rules,
    )
    .unwrap()
}
fn quiet() -> SliceRules {
    SliceRules {
        wave: vec![Spawn {
            at: 100_000,
            row: 0,
            x: 79_000,
        }],
        ..SliceRules::development()
    }
}
fn plant(cell: Cell, kind: PlantKind) -> [Command; 2] {
    [Command::SelectCard { kind }, Command::Plant { cell }]
}

#[test]
fn 五乘九种植边界重复余额和冷却拒绝顺序明确() {
    let mut rules = quiet();
    rules.card_cooldown = 1;
    let mut state = game(rules, 10_000);
    for row in 0..5 {
        for column in 0..9 {
            state.step(&plant(Cell { row, column }, PlantKind::Sunflower));
        }
    }
    assert_eq!(state.board().unwrap().plants().len(), 45);
    assert_eq!(state.sun(), 7_750);
    let events = state.step(&[
        Command::Plant {
            cell: Cell { row: 0, column: 0 },
        },
        Command::Plant {
            cell: Cell { row: 5, column: 0 },
        },
    ]);
    assert_eq!(
        &events.as_slice()[1..3],
        &[
            Event::CommandRejected {
                reason: Rejection::Occupied
            },
            Event::CommandRejected {
                reason: Rejection::InvalidCell
            }
        ]
    );
    let mut state = game(quiet(), 50);
    let events = state.step(&plant(Cell { row: 0, column: 0 }, PlantKind::Peashooter));
    assert!(events.as_slice().contains(&Event::CommandRejected {
        reason: Rejection::NotEnoughSun
    }));
    state.step(&plant(Cell { row: 0, column: 0 }, PlantKind::Sunflower));
    assert_eq!(state.sun(), 0);
    assert!(
        state
            .step(&[Command::Plant {
                cell: Cell { row: 0, column: 1 }
            }])
            .as_slice()
            .contains(&Event::CommandRejected {
                reason: Rejection::CoolingDown
            })
    );
}

#[test]
fn 扣费种植事件顺序独立核对并且铲除不返还() {
    let mut state = game(quiet(), 100);
    let cell = Cell { row: 2, column: 0 };
    assert_eq!(
        state.step(&plant(cell, PlantKind::Peashooter)).as_slice(),
        &[
            Event::TickStarted { tick: 1 },
            Event::CardSelected {
                kind: PlantKind::Peashooter
            },
            Event::SunSpent {
                amount: 100,
                total: 0
            },
            Event::Planted {
                id: 1,
                kind: PlantKind::Peashooter,
                cell
            },
            Event::TickCompleted { tick: 1 }
        ]
    );
    state.step(&[Command::Shovel { cell }]);
    assert!(state.board().unwrap().plants().is_empty());
    assert_eq!(state.sun(), 0);
    assert!(
        state
            .step(&[Command::Shovel { cell }])
            .as_slice()
            .contains(&Event::CommandRejected {
                reason: Rejection::MissingPlant
            })
    );
}

#[test]
fn 阳光生成同tick重复收集过期与溢出保持实体() {
    let mut rules = quiet();
    rules.sky_sun_interval = 2;
    rules.sun_lifetime = 2;
    let mut state = game(rules.clone(), 0);
    state.step(&[]);
    state.step(&[]);
    let id = state.board().unwrap().suns()[0].id;
    let position = state.board().unwrap().suns()[0].position;
    assert_eq!(state.board().unwrap().sun_at(position), Some(id));
    assert_eq!(
        state.board().unwrap().sun_at(Point {
            x: i32::MIN,
            y: i32::MAX
        }),
        None
    );
    let events = state.step(&[Command::CollectSun { id }, Command::CollectSun { id }]);
    assert_eq!(
        &events.as_slice()[1..3],
        &[
            Event::SunCollected {
                id,
                amount: 25,
                total: 25
            },
            Event::CommandRejected {
                reason: Rejection::MissingSun
            }
        ]
    );
    let mut overflow = game(rules, u32::MAX);
    overflow.step(&[]);
    overflow.step(&[]);
    let id = overflow.board().unwrap().suns()[0].id;
    assert!(
        overflow
            .step(&[Command::CollectSun { id }])
            .as_slice()
            .contains(&Event::CommandRejected {
                reason: Rejection::BalanceOverflow
            })
    );
    assert_eq!(overflow.board().unwrap().suns().len(), 1);
    assert!(
        overflow
            .step(&[])
            .as_slice()
            .contains(&Event::SunExpired { id })
    );
}

#[test]
fn 豌豆扫掠同排最近目标跨排隔离伤害死亡胜利有序() {
    let mut rules = quiet();
    rules.shoot_interval = 1;
    rules.pea_speed = 30_000;
    rules.zombie_health = 20;
    rules.wave = vec![
        Spawn {
            at: 1,
            row: 0,
            x: 40_000,
        },
        Spawn {
            at: 1,
            row: 1,
            x: 20_000,
        },
        Spawn {
            at: 1,
            row: 0,
            x: 20_000,
        },
    ];
    let mut state = game(rules, 100);
    let events = state.step(&plant(Cell { row: 0, column: 0 }, PlantKind::Peashooter));
    let hit = events
        .as_slice()
        .iter()
        .position(|e| matches!(e, Event::ProjectileHit { .. }))
        .unwrap();
    assert_eq!(
        &events.as_slice()[hit..hit + 3],
        &[
            Event::ProjectileHit { id: 5, zombie: 4 },
            Event::Damaged {
                target: 4,
                remaining: 0
            },
            Event::ZombieDied { id: 4 }
        ]
    );
    assert_eq!(state.board().unwrap().zombies()[0].row, 0);
    assert_eq!(state.board().unwrap().zombies()[0].health, 20);
    assert_eq!(state.board().unwrap().zombies()[1].row, 1);
    assert_eq!(state.board().unwrap().zombies()[1].health, 20);
}

#[test]
fn 向日葵按规则生成而啃食植物死亡后释放格子() {
    let mut rules = quiet();
    rules.sunflower_interval = 1;
    rules.plant_health = 20;
    rules.wave = vec![Spawn {
        at: 1,
        row: 0,
        x: 16_000,
    }];
    let mut state = game(rules, 50);
    let events = state.step(&plant(Cell { row: 0, column: 0 }, PlantKind::Sunflower));
    assert!(
        events
            .as_slice()
            .iter()
            .any(|e| matches!(e, Event::SunSpawned { .. }))
    );
    assert!(events.as_slice().contains(&Event::Damaged {
        target: 1,
        remaining: 0
    }));
    assert!(events.as_slice().contains(&Event::PlantRemoved { id: 1 }));
    assert!(state.board().unwrap().plants().is_empty());
}

#[test]
fn 割草机一次启动固定波次完成胜利与后续突破失败() {
    let mut rules = quiet();
    rules.zombie_speed = 10_000;
    rules.mower_speed = 100_000;
    rules.wave = vec![Spawn {
        at: 1,
        row: 0,
        x: 6_000,
    }];
    let mut state = game(rules.clone(), 0);
    let events = state.step(&[]);
    assert!(events.as_slice().contains(&Event::MowerStarted { row: 0 }));
    assert!(events.as_slice().contains(&Event::Victory));
    assert_eq!(state.board().unwrap().status(), GameStatus::Won);
    let before = state.state_digest();
    let events = state.step(&[Command::Pause, Command::CollectSun { id: 1 }]);
    assert_eq!(state.state_digest(), before);
    assert_eq!(
        events.as_slice(),
        &[
            Event::CommandRejected {
                reason: Rejection::Ended
            },
            Event::CommandRejected {
                reason: Rejection::Ended
            }
        ]
    );
    rules.wave.push(Spawn {
        at: 2,
        row: 0,
        x: 6_000,
    });
    let mut state = game(rules, 0);
    let mut all = state.step(&[]).as_slice().to_vec();
    all.extend_from_slice(state.step(&[]).as_slice());
    assert_eq!(
        all.iter()
            .filter(|e| matches!(e, Event::MowerStarted { .. }))
            .count(),
        1
    );
    assert!(all.contains(&Event::Defeat));
    assert_eq!(state.board().unwrap().status(), GameStatus::Lost);
}

#[test]
fn 暂停恢复同批输入重开前后命令及初始合同恢复() {
    let mut state = game(quiet(), 100);
    state.step(&[Command::Pause]);
    let frozen = state.state_digest();
    for _ in 0..20 {
        state.step(&[]);
    }
    assert_eq!(state.state_digest(), frozen);
    assert_eq!(state.tick(), 0);
    let cell = Cell { row: 0, column: 0 };
    let events = state.step(&[
        Command::Plant { cell },
        Command::Resume,
        Command::SelectCard {
            kind: PlantKind::Peashooter,
        },
        Command::Plant { cell },
    ]);
    assert_eq!(
        events.as_slice()[1],
        Event::CommandRejected {
            reason: Rejection::Paused
        }
    );
    assert_eq!(state.sun(), 0);
    assert_eq!(state.tick(), 1);
    state.step(&[Command::Pause, Command::Resume]);
    assert_eq!(state.tick(), 2);
    let original = game(quiet(), 100).state_digest();
    state.step(&[
        Command::SelectCard {
            kind: PlantKind::Sunflower,
        },
        Command::Restart,
    ]);
    assert_eq!(state.state_digest(), original);
    state.step(&[
        Command::Restart,
        Command::SelectCard {
            kind: PlantKind::Peashooter,
        },
        Command::Plant { cell },
    ]);
    assert_eq!(state.tick(), 0);
    assert_eq!(state.sun(), 0);
    assert_eq!(state.board().unwrap().plants().len(), 1);
}

#[test]
fn 无资源胜负回放逐步一致摘要包含隐藏状态与规则() {
    for (wave, terminal) in [
        (
            vec![Spawn {
                at: 1,
                row: 0,
                x: 6_000,
            }],
            GameStatus::Won,
        ),
        (
            vec![
                Spawn {
                    at: 1,
                    row: 0,
                    x: 6_000,
                },
                Spawn {
                    at: 2,
                    row: 0,
                    x: 6_000,
                },
            ],
            GameStatus::Lost,
        ),
    ] {
        let mut rules = quiet();
        rules.wave = wave;
        rules.zombie_speed = 10_000;
        rules.mower_speed = 100_000;
        let mut left = game(rules.clone(), 100);
        let mut right = game(rules, 100);
        for _ in 0..10 {
            assert_eq!(left.step(&[]), right.step(&[]));
            assert_eq!(left.state_digest(), right.state_digest());
        }
        assert_eq!(left.board().unwrap().status(), terminal);
    }
    let mut rules = quiet();
    rules.bite_interval += 1;
    assert_ne!(
        game(rules, 100).state_digest(),
        game(quiet(), 100).state_digest()
    );
    let mut selected = game(quiet(), 100);
    selected.step(&[
        Command::SelectCard {
            kind: PlantKind::Sunflower,
        },
        Command::Pause,
    ]);
    assert_ne!(selected.state_digest(), game(quiet(), 100).state_digest());
}

#[test]
fn 棋盘坐标边界与非法规则显式拒绝() {
    assert_eq!(
        BoardGeometry::cell_at(Point { x: 8000, y: 10000 }),
        Some(Cell { row: 0, column: 0 })
    );
    assert_eq!(
        BoardGeometry::cell_at(Point { x: 79999, y: 57999 }),
        Some(Cell { row: 4, column: 8 })
    );
    assert_eq!(BoardGeometry::cell_at(Point { x: 80000, y: 58000 }), None);
    let mut rules = quiet();
    rules.pea_speed = 0;
    assert!(rules.validate().is_err());
    rules = quiet();
    rules.wave[0].row = 5;
    assert!(rules.validate().is_err());
    rules = quiet();
    rules.wave.clear();
    assert!(rules.validate().is_err());
}

#[test]
fn 摘要区分当前余额相同但重开结果不同的初始合同() {
    let initial = |sun| InitialState::new(LevelId::new("day-contract").unwrap(), sun);
    let mut left = GameState::new(7, initial(50));
    let mut right = GameState::new(7, initial(100));
    left.step(&[]);
    right.step(&[Command::SpendSun { amount: 50 }]);
    assert_eq!(left.sun(), right.sun());
    assert_ne!(left.state_digest(), right.state_digest());
}

#[test]
fn 切片开发规则主版本升级但空棋盘保持原合同() {
    assert_eq!(RulesVersion::current().to_string(), "2.0");
    let game = GameState::new(7, InitialState::new(LevelId::new("empty").unwrap(), 50));
    assert_eq!(game.state_digest().rules_version().to_string(), "1.0");
}
