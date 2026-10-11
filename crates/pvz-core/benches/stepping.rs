use criterion::{Criterion, black_box, criterion_group, criterion_main};
use pvz_core::{GameState, InitialState};
use pvz_data::LevelId;

fn 空棋盘推进(c: &mut Criterion) {
    c.bench_function("空棋盘推进-1000 tick", |bencher| {
        bencher.iter(|| {
            let level = LevelId::new("day-1").expect("固定夹具必须有效");
            let initial = InitialState::new(level, 50);
            let mut state = GameState::new(7, initial);
            for _ in 0..1_000 {
                black_box(state.step(&[]));
            }
            black_box(state.state_digest());
        });
    });
}

criterion_group!(benches, 空棋盘推进);
criterion_main!(benches);
