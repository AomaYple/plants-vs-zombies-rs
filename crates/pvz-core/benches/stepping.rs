use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use std::hint::black_box;

#[path = "support/day_scenario.rs"]
mod day_scenario;
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
            "开发规则{}-{label}-1000次step-实际推进1000tick-每tick{}命令-seed7",
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
    let mut checked = day_scenario::prepared_day();
    let advanced = day_scenario::measured_steps(&mut checked);
    assert_eq!(advanced, 1_000, "基准实际推进量必须与回归合同一致");
    c.bench_function(
        &format!("开发规则2.0-5排-预热45植物(5日40豌)-波次总量100-1000次step-实际推进{advanced}tick-seed7"),
        |bencher| {
            bencher.iter_batched(
                day_scenario::prepared_day,
                |mut state| {
                    black_box(day_scenario::measured_steps(&mut state));
                    black_box(state.state_digest());
                },
                BatchSize::SmallInput,
            );
        },
    );
}

criterion_group!(benches, 模拟推进);
criterion_main!(benches);
