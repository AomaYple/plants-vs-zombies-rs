use criterion::{BatchSize, Criterion, black_box, criterion_group, criterion_main};
use pvz_core::{Cell, Command, GameState, InitialState, PlantKind, SliceRules, Spawn};
use pvz_data::{LevelId, RulesVersion};

// 场景为原创开发输入，无原版资源；创建初始状态放在计时区间之外。
fn 模拟推进(c: &mut Criterion) {
    let commands = [
        Command::SpendSun { amount: 10 },
        Command::AddSun { amount: 10 },
    ];
    for (level, label, input) in [
        ("benchmark-empty", "空棋盘", &[][..]),
        ("benchmark-commands", "阳光命令", &commands[..]),
    ] {
        let name = format!(
            "开发规则{}-{label}-1000tick-每tick{}命令-seed7",
            RulesVersion::new(1, 0).expect("空棋盘版本有效"),
            input.len()
        );
        c.bench_function(&name, |bencher| {
            bencher.iter_batched(
                || {
                    GameState::new(
                        7,
                        InitialState::new(LevelId::new(level).expect("固定原创夹具必须有效"), 0),
                    )
                },
                |mut state| {
                    for _ in 0..1_000 {
                        black_box(state.step(black_box(input)));
                    }
                    black_box(state.state_digest());
                },
                BatchSize::SmallInput,
            );
        });
    }
    c.bench_function(
        "开发规则2.0-5排45植物100僵尸-1000tick-seed7",
        |bencher| {
            bencher.iter_batched(
                || {
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
                            state.step(&[
                                Command::SelectCard {
                                    kind: if column == 0 {
                                        PlantKind::Sunflower
                                    } else {
                                        PlantKind::Peashooter
                                    },
                                },
                                Command::Plant {
                                    cell: Cell { row, column },
                                },
                            ]);
                        }
                    }
                    state
                },
                |mut state| {
                    for _ in 0..1_000 {
                        black_box(state.step(black_box(&[])));
                    }
                    black_box(state.state_digest());
                },
                BatchSize::SmallInput,
            );
        },
    );
}

criterion_group!(benches, 模拟推进);
criterion_main!(benches);
