use pvz_app::{content::DayContentError, runtime::Runtime};
use pvz_core::{Cell, Command, Event, GameState, GameStatus, InitialState, PlantKind, SliceRules};
use pvz_data::LevelId;
use std::error::Error;

const DAY_JSON: &str = include_str!("../../../content/levels/development-day.json");

fn content_value() -> serde_json::Value {
    serde_json::from_str(DAY_JSON).unwrap()
}

fn with_frequency(frequency: u32) -> Runtime {
    let mut value = content_value();
    value["ticks_per_second"] = frequency.into();
    Runtime::from_day_json(&value.to_string()).unwrap()
}

#[test]
fn 默认内容入口与旧开发工厂拥有相同初始合同而探针保持独立() {
    let expected = GameState::day(
        7,
        InitialState::new(LevelId::new("development-day-slice").unwrap(), 150),
        SliceRules::development(),
    )
    .unwrap();
    assert_eq!(Runtime::day().game.state_digest(), expected.state_digest());
    assert_eq!(
        Runtime::from_day_json(DAY_JSON)
            .unwrap()
            .game
            .state_digest(),
        expected.state_digest()
    );
    let probe = Runtime::default();
    assert!(probe.game.board().is_none());
    assert_eq!(probe.game.sun(), 50);
}

#[test]
fn 内容变更驱动实际种植成本与初始状态而没有回退() {
    let mut value = content_value();
    value["seed"] = 42.into();
    value["initial_sun"] = 91.into();
    value["level"] = "original-app-test".into();
    value["rules"]["sunflower_cost"] = 17.into();
    let mut runtime = Runtime::from_day_json(&value.to_string()).unwrap();
    assert_eq!(runtime.game.sun(), 91);
    assert_eq!(runtime.game.level().as_str(), "original-app-test");
    runtime.command(Command::SelectCard {
        kind: PlantKind::Sunflower,
    });
    runtime.command(Command::Plant {
        cell: Cell { row: 0, column: 0 },
    });
    runtime.advance(1.0 / 60.0);
    assert_eq!(runtime.game.sun(), 74);
    assert_eq!(runtime.game.board().unwrap().plants().len(), 1);
    assert_ne!(
        runtime.game.state_digest(),
        Runtime::day().game.state_digest()
    );
}

#[test]
fn 损坏未知遗漏重复字段与未知版本明确解析失败并保留原因() {
    let mut cases = vec![
        "{".to_owned(),
        DAY_JSON.replace("\"seed\": 7", "\"seed\": 7, \"seed\": 8"),
    ];
    for (field, replacement) in [
        ("schema_version", serde_json::json!(99)),
        ("basis", serde_json::json!("measured")),
    ] {
        let mut value = content_value();
        value[field] = replacement;
        cases.push(value.to_string());
    }
    let mut unknown = content_value();
    unknown["unknown"] = true.into();
    cases.push(unknown.to_string());
    let mut missing = content_value();
    missing.as_object_mut().unwrap().remove("seed");
    cases.push(missing.to_string());
    for source in cases {
        let error = Runtime::from_day_json(&source).err().expect("输入必须拒绝");
        assert!(matches!(error, DayContentError::Parse(_)));
        assert!(error.to_string().contains("白天内容"));
        assert!(error.source().is_some());
    }
}

#[test]
fn 非法规则和频率没有默认回退且中文诊断保留核心原因() {
    let mut unsupported = content_value();
    unsupported["rules_version"] = "9.0".into();
    let error = Runtime::from_day_json(&unsupported.to_string())
        .err()
        .expect("未知规则版本必须拒绝");
    assert!(matches!(error, DayContentError::InvalidLevel(_)));
    assert!(error.source().is_some());
    for (field, number) in [
        ("ticks_per_second", 0),
        ("ticks_per_second", 1001),
        ("pea_damage", 0),
    ] {
        let mut value = content_value();
        if field == "pea_damage" {
            value["rules"][field] = number.into();
        } else {
            value[field] = number.into();
        }
        let error = Runtime::from_day_json(&value.to_string())
            .err()
            .expect("无效参数必须拒绝");
        assert!(matches!(error, DayContentError::InvalidLevel(_)));
        assert!(error.to_string().contains("白天内容"));
        assert!(error.source().is_some());
    }
}

#[test]
fn 一兆字节按字节判断边界且超大输入先于解析拒绝() {
    let mut boundary = DAY_JSON.to_owned();
    boundary.extend(std::iter::repeat_n(' ', 1_048_576 - boundary.len()));
    assert!(Runtime::from_day_json(&boundary).is_ok());
    boundary.push(' ');
    assert!(matches!(
        Runtime::from_day_json(&boundary),
        Err(DayContentError::TooLarge { bytes: 1_048_577 })
    ));
    let error = Runtime::from_day_json(&boundary).err().unwrap();
    assert!(error.to_string().contains("白天内容") && error.to_string().contains("1048576"));
    assert!(error.source().is_none());
    let malformed = "坏".repeat(349_526);
    assert!(matches!(
        Runtime::from_day_json(&malformed),
        Err(DayContentError::TooLarge { bytes: 1_048_578 })
    ));
}

#[test]
fn 实际30与120赫兹时钟消费四分之一秒且保留帧上限和余量() {
    for (frequency, first, after_remainder, capped) in [(30, 7, 8, 15), (120, 30, 32, 62)] {
        let mut runtime = with_frequency(frequency);
        runtime.advance(0.25);
        assert_eq!(runtime.game.tick(), first);
        runtime.advance(1.0 / 60.0);
        assert_eq!(runtime.game.tick(), after_remainder);
        runtime.advance(20.0);
        assert_eq!(runtime.game.tick(), capped);
    }
}

#[test]
fn 默认60赫兹细帧保留余量且有序命令只在首个实际tick消费() {
    let mut runtime = Runtime::day();
    runtime.command(Command::SelectCard {
        kind: PlantKind::Sunflower,
    });
    runtime.command(Command::Plant {
        cell: Cell { row: 0, column: 0 },
    });
    for expected_tick in [0, 0, 0, 1, 1, 1, 2] {
        runtime.advance(0.005);
        assert_eq!(runtime.game.tick(), expected_tick);
        assert_eq!(
            runtime.game.board().unwrap().plants().len(),
            usize::from(expected_tick != 0)
        );
    }
    assert_eq!(runtime.game.sun(), 100);
    let events = runtime.take_events();
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(event, Event::Planted { .. }))
            .count(),
        1
    );
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(event, Event::TickStarted { .. }))
            .count(),
        2
    );
    assert!((runtime.animation_seconds - 0.035).abs() < 1e-12);
}

#[test]
fn 非默认时钟暂停冻结恢复不补停顿重开恢复完整初始合同和动画() {
    for frequency in [30, 120] {
        let mut runtime = with_frequency(frequency);
        let initial = runtime.game.state_digest();
        runtime.advance(0.25);
        runtime.command(Command::Pause);
        runtime.advance(1.0 / f64::from(frequency));
        let paused = runtime.game.state_digest();
        let animation = runtime.animation_seconds;
        runtime.advance(100.0);
        assert_eq!(runtime.game.state_digest(), paused);
        assert_eq!(runtime.animation_seconds, animation);
        runtime.command(Command::Resume);
        runtime.advance(100.0);
        assert_eq!(runtime.game.tick(), if frequency == 30 { 8 } else { 31 });
        runtime.command(Command::Restart);
        runtime.advance(0.25);
        assert_eq!(runtime.game.state_digest(), initial);
        assert_eq!(runtime.game.clock().ticks_per_second(), frequency);
        assert_eq!(runtime.animation_seconds, 0.0);
        runtime.advance(1.0 / f64::from(frequency));
        assert_eq!(runtime.game.tick(), 1);
        assert!(runtime.take_events().contains(&Event::Restarted));
    }
}

#[test]
fn 自定义时钟终局冻结且重开不承接终局耗时() {
    let mut value = content_value();
    value["ticks_per_second"] = 120.into();
    value["rules"]["wave"] = serde_json::json!([{ "at": 1, "row": 0, "x": 6000 }]);
    let mut runtime = Runtime::from_day_json(&value.to_string()).unwrap();
    runtime.advance(0.25);
    assert_eq!(runtime.game.board().unwrap().status(), GameStatus::Won);
    let terminal = runtime.game.state_digest();
    let animation = runtime.animation_seconds;
    runtime.advance(100.0);
    assert_eq!(runtime.game.state_digest(), terminal);
    assert_eq!(runtime.animation_seconds, animation);
    runtime.command(Command::Restart);
    runtime.advance(100.0);
    assert_eq!(runtime.game.tick(), 0);
    assert_eq!(runtime.game.clock().ticks_per_second(), 120);
    assert_eq!(runtime.animation_seconds, 0.0);
}
