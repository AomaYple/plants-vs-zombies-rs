use criterion::{BatchSize, Criterion, black_box, criterion_group, criterion_main};
use pvz_core::{Command, GameState, InitialState};
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
            RulesVersion::current(),
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
}

criterion_group!(benches, 模拟推进);
criterion_main!(benches);
