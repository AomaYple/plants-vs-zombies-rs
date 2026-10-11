use pvz_app::{
    input::InputEvent,
    resources::{SoundCue, selected_pose},
    runtime::{Runtime, sound_cue},
    viewport::LogicalPoint,
};
use pvz_core::{Cell, Command, Event, GameStatus, PlantKind};

fn click(runtime: &mut Runtime, x: f32, y: f32) {
    runtime.input(InputEvent::PointerDown {
        id: 9,
        point: LogicalPoint { x, y },
    });
}

#[test]
fn 无资源平台逻辑输入选卡种植铲除与阳光没有作弊入口() {
    let mut runtime = Runtime::day();
    click(&mut runtime, 180.0, 25.0);
    click(&mut runtime, 120.0, 148.0);
    runtime.advance(1.0 / 60.0);
    let board = runtime.game.board().unwrap();
    assert_eq!(board.plants()[0].cell, Cell { row: 0, column: 0 });
    assert_eq!(board.plants()[0].kind, PlantKind::Sunflower);
    assert_eq!(runtime.game.sun(), 100);
    let events = runtime.take_events();
    assert_eq!(
        events.iter().filter_map(sound_cue).collect::<Vec<_>>(),
        [SoundCue::Plant]
    );
    assert!(runtime.take_events().is_empty());
    click(&mut runtime, 480.0, 25.0);
    click(&mut runtime, 120.0, 148.0);
    runtime.advance(1.0 / 60.0);
    assert!(runtime.game.board().unwrap().plants().is_empty());
    assert_eq!(runtime.game.sun(), 100);
    click(&mut runtime, 20.0, 300.0);
    runtime.advance(1.0 / 60.0);
    assert_eq!(runtime.game.sun(), 100);
}

#[test]
fn 切片暂停仍接受恢复重开且终局停止动画与模拟() {
    let mut runtime = Runtime::day();
    click(&mut runtime, 600.0, 25.0);
    runtime.advance(1.0 / 60.0);
    assert!(runtime.paused);
    let digest = runtime.game.state_digest();
    runtime.advance(10.0);
    assert_eq!(runtime.game.state_digest(), digest);
    assert_eq!(runtime.animation_seconds, 0.0);
    click(&mut runtime, 600.0, 25.0);
    runtime.advance(1.0 / 60.0);
    assert!(!runtime.paused);
    assert_eq!(runtime.game.tick(), 1);
    runtime.command(Command::Pause);
    runtime.advance(1.0 / 60.0);
    click(&mut runtime, 730.0, 25.0);
    runtime.advance(1.0 / 60.0);
    assert_eq!(runtime.game.tick(), 0);
    assert_eq!(runtime.game.sun(), 150);
    assert_eq!(runtime.game.board().unwrap().status(), GameStatus::Running);
    assert!(!runtime.paused);
    click(&mut runtime, 600.0, 25.0);
    click(&mut runtime, 600.0, 25.0);
    runtime.advance(1.0 / 60.0);
    assert!(!runtime.paused, "同批两次按钮点击依次暂停和恢复");
}

#[test]
fn 原创构图选择保留源层序并拒绝缺失轨道或空帧() {
    let clip = pvz_assets::parse_reanim("<fps>12</fps><track><name>底层</name><t><i>底图</i></t></track><track><name>附件</name><t><i>附件图</i></t></track><track><name>上层</name><t><i>上图</i></t></track>").unwrap();
    let animation = pvz_app::animation::PreparedAnimation::new(&clip);
    let pose = selected_pose(&animation, 0, &["上层", "底层"]).unwrap();
    assert_eq!(
        pose.iter()
            .map(|f| f.i.as_deref().unwrap())
            .collect::<Vec<_>>(),
        ["底图", "上图"]
    );
    assert!(selected_pose(&animation, 0, &["缺失"]).is_err());
    assert!(selected_pose(&animation, 1, &[]).is_err());
}

#[test]
fn 命中收集和割草机事件只映射音效而伤害不重算() {
    assert_eq!(
        sound_cue(&Event::ProjectileHit { id: 1, zombie: 2 }),
        Some(SoundCue::Hit)
    );
    assert_eq!(
        sound_cue(&Event::SunCollected {
            id: 3,
            amount: 25,
            total: 50
        }),
        Some(SoundCue::Collect)
    );
    assert_eq!(
        sound_cue(&Event::MowerStarted { row: 1 }),
        Some(SoundCue::Mower)
    );
    assert_eq!(
        sound_cue(&Event::Damaged {
            target: 2,
            remaining: 0
        }),
        None
    );
}

#[test]
fn 动画帧时钟独立于模拟tick且长帧重开不承接旧局余量() {
    let mut runtime = Runtime::day();
    runtime.advance(0.005);
    assert_eq!(runtime.game.tick(), 0);
    assert_eq!(runtime.animation_seconds, 0.005);
    runtime.command(Command::Restart);
    runtime.advance(0.25);
    assert_eq!(runtime.game.tick(), 0);
    assert_eq!(runtime.animation_seconds, 0.0);
}

#[test]
fn 同批重开后铲除选择保留到下一批() {
    let mut runtime = Runtime::day();
    click(&mut runtime, 480.0, 25.0);
    runtime.command(Command::Restart);
    click(&mut runtime, 480.0, 25.0);
    runtime.advance(1.0 / 60.0);
    assert!(runtime.shovel);
    click(&mut runtime, 120.0, 148.0);
    runtime.advance(1.0 / 60.0);
    assert!(runtime.take_events().contains(&Event::CommandRejected {
        reason: pvz_core::Rejection::MissingPlant
    }));
}

#[test]
fn 铲除后重开清空选择且下一批选卡种植正常() {
    let mut runtime = Runtime::day();
    click(&mut runtime, 480.0, 25.0);
    click(&mut runtime, 730.0, 25.0);
    runtime.advance(1.0 / 60.0);
    assert!(!runtime.shovel);
    click(&mut runtime, 180.0, 25.0);
    click(&mut runtime, 120.0, 148.0);
    runtime.advance(1.0 / 60.0);
    assert_eq!(runtime.game.board().unwrap().plants().len(), 1);
    assert_eq!(runtime.game.sun(), 100);
}

#[test]
fn 已暂停时重开后暂停切换基于新局意图() {
    let mut runtime = Runtime::day();
    runtime.input(InputEvent::Pause);
    runtime.advance(1.0 / 60.0);
    runtime.input(InputEvent::Restart);
    runtime.input(InputEvent::TogglePause);
    runtime.advance(1.0 / 60.0);
    assert!(runtime.paused);
    assert_eq!(runtime.game.tick(), 0);
    assert!(matches!(
        runtime.take_events().as_slice(),
        [Event::Paused, Event::Restarted, Event::Paused]
    ));
}

fn sky_sun_fixture() -> (Runtime, LogicalPoint) {
    let mut runtime = Runtime::day();
    for _ in 0..20 {
        runtime.advance(0.25);
    }
    let sun = &runtime.game.board().unwrap().suns()[0];
    let point = LogicalPoint {
        x: sun.position.x as f32 / pvz_core::UNITS_PER_PIXEL as f32,
        y: sun.position.y as f32 / pvz_core::UNITS_PER_PIXEL as f32,
    };
    runtime.take_events();
    (runtime, point)
}

#[test]
fn 同批重开后旧阳光命中不能覆盖新局种植与铲除() {
    for shovel in [false, true] {
        let (mut runtime, point) = sky_sun_fixture();
        runtime.input(InputEvent::Restart);
        click(&mut runtime, if shovel { 480.0 } else { 180.0 }, 25.0);
        click(&mut runtime, point.x, point.y);
        runtime.advance(1.0 / 60.0);
        let events = runtime.take_events();
        if shovel {
            assert!(events.contains(&Event::CommandRejected {
                reason: pvz_core::Rejection::MissingPlant
            }));
            assert!(runtime.shovel);
            assert_eq!(runtime.game.sun(), 150);
        } else {
            assert_eq!(runtime.game.board().unwrap().plants().len(), 1);
            assert_eq!(runtime.game.sun(), 100);
            assert!(events.iter().any(|event| matches!(
                event,
                Event::Planted {
                    kind: PlantKind::Sunflower,
                    ..
                }
            )));
        }
        assert!(!events.contains(&Event::CommandRejected {
            reason: pvz_core::Rejection::MissingSun
        }));
    }
}

#[test]
fn 同批重开前点击阳光仍先收集后重开() {
    let (mut runtime, point) = sky_sun_fixture();
    click(&mut runtime, point.x, point.y);
    runtime.input(InputEvent::Restart);
    runtime.advance(1.0 / 60.0);
    assert!(matches!(
        runtime.take_events().as_slice(),
        [
            Event::SunCollected {
                amount: 25,
                total: 175,
                ..
            },
            Event::Restarted
        ]
    ));
    assert_eq!(runtime.game.sun(), 150);
    assert!(runtime.game.board().unwrap().suns().is_empty());
}
