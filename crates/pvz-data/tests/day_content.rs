use pvz_data::{
    ContentBasis, DayLevelDefinition, DayRulesDefinition, DaySpawnDefinition, LevelId,
    RulesVersion, SchemaVersion,
};
use serde_json::{Value, json};

const DEVELOPMENT_DAY: &str = include_str!("../../../content/levels/development-day.json");

fn parse(value: &Value) -> Result<DayLevelDefinition, serde_json::Error> {
    serde_json::from_str(&serde_json::to_string(value).unwrap())
}

fn source() -> Value {
    serde_json::from_str(DEVELOPMENT_DAY).unwrap()
}

#[test]
fn 原创关卡逐字段锁定默认值和有序波次并稳定往返() {
    let definition: DayLevelDefinition = serde_json::from_str(DEVELOPMENT_DAY).unwrap();
    assert_eq!(definition.level.as_str(), "development-day-slice");
    assert_eq!(definition.rules.wave[0].at, 600);
    assert_eq!(definition.rules.wave[1].row, 0);
    assert_eq!(
        definition,
        DayLevelDefinition {
            schema_version: SchemaVersion::current(),
            rules_version: RulesVersion::new(2, 0).unwrap(),
            basis: ContentBasis::Development,
            level: LevelId::new("development-day-slice").unwrap(),
            seed: 7,
            initial_sun: 150,
            ticks_per_second: 60,
            rules: DayRulesDefinition {
                plant_health: 300,
                zombie_health: 200,
                pea_damage: 20,
                bite_damage: 20,
                zombie_speed: 20,
                pea_speed: 600,
                mower_speed: 1_200,
                plant_half_width: 2_400,
                zombie_half_width: 2_000,
                pea_radius: 700,
                mower_half_width: 3_000,
                sun_radius: 3_000,
                mower_trigger_x: 6_000,
                defeat_x: 0,
                sunflower_cost: 50,
                peashooter_cost: 100,
                card_cooldown: 180,
                shoot_interval: 90,
                bite_interval: 30,
                sunflower_interval: 600,
                sky_sun_interval: 300,
                sun_lifetime: 900,
                sun_amount: 25,
                wave: vec![
                    DaySpawnDefinition {
                        at: 600,
                        row: 2,
                        x: 79_000
                    },
                    DaySpawnDefinition {
                        at: 960,
                        row: 0,
                        x: 79_000
                    },
                    DaySpawnDefinition {
                        at: 1_320,
                        row: 4,
                        x: 79_000
                    },
                    DaySpawnDefinition {
                        at: 1_680,
                        row: 1,
                        x: 79_000
                    },
                    DaySpawnDefinition {
                        at: 2_040,
                        row: 3,
                        x: 79_000
                    },
                    DaySpawnDefinition {
                        at: 2_400,
                        row: 2,
                        x: 79_000
                    },
                    DaySpawnDefinition {
                        at: 2_760,
                        row: 0,
                        x: 79_000
                    },
                    DaySpawnDefinition {
                        at: 3_120,
                        row: 4,
                        x: 79_000
                    },
                    DaySpawnDefinition {
                        at: 3_480,
                        row: 1,
                        x: 79_000
                    },
                    DaySpawnDefinition {
                        at: 3_840,
                        row: 3,
                        x: 79_000
                    },
                ],
            },
        }
    );
    let encoded = serde_json::to_string(&definition).unwrap();
    assert_eq!(
        serde_json::from_str::<DayLevelDefinition>(&encoded).unwrap(),
        definition
    );
    assert_eq!(
        serde_json::to_string(&serde_json::from_str::<DayLevelDefinition>(&encoded).unwrap())
            .unwrap(),
        encoded
    );
}

#[test]
fn 所有对象层级拒绝未知遗漏重复和空字段() {
    let baseline = source();
    for path in ["", "/rules", "/rules/wave/0"] {
        let fields = baseline.pointer(path).unwrap().as_object().unwrap();
        let mut unknown = baseline.clone();
        unknown
            .pointer_mut(path)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("unknown".into(), json!(1));
        assert!(parse(&unknown).is_err(), "未知字段未拒绝：{path}");
        for (field, value) in fields {
            let mut missing = baseline.clone();
            missing
                .pointer_mut(path)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .remove(field);
            assert!(parse(&missing).is_err(), "遗漏字段未拒绝：{path}/{field}");
            let mut null = baseline.clone();
            null.pointer_mut(path).unwrap()[field] = Value::Null;
            assert!(parse(&null).is_err(), "空字段未拒绝：{path}/{field}");

            // 直接保留重复键的 JSON 文本，避免 Value 的对象映射吞掉重复字段。
            let original = serde_json::to_string(fields).unwrap();
            let duplicated = format!(
                "{{{}:{},{}",
                serde_json::to_string(field).unwrap(),
                value,
                &original[1..]
            );
            let encoded =
                serde_json::to_string(&baseline)
                    .unwrap()
                    .replacen(&original, &duplicated, 1);
            assert!(
                serde_json::from_str::<DayLevelDefinition>(&encoded).is_err(),
                "重复字段未拒绝：{path}/{field}"
            );
        }
    }
}

#[test]
fn 整数拒绝小数错误类型溢出和无符号负数() {
    let baseline = source();
    for path in ["", "/rules", "/rules/wave/0"] {
        for (field, original) in baseline.pointer(path).unwrap().as_object().unwrap() {
            if !original.is_number() {
                continue;
            }
            let pointer = format!("{path}/{field}");
            let signed = (path == "/rules"
                && matches!(
                    field.as_str(),
                    "zombie_speed"
                        | "pea_speed"
                        | "mower_speed"
                        | "plant_half_width"
                        | "zombie_half_width"
                        | "pea_radius"
                        | "mower_half_width"
                        | "sun_radius"
                        | "mower_trigger_x"
                        | "defeat_x"
                ))
                || (path == "/rules/wave/0" && field == "x");
            let maximum = if signed {
                "2147483648"
            } else if field == "schema_version" {
                "65536"
            } else if path == "/rules/wave/0" && field == "row" {
                "256"
            } else if field == "seed"
                || field == "at"
                || field.ends_with("interval")
                || matches!(field.as_str(), "card_cooldown" | "sun_lifetime")
            {
                "18446744073709551616"
            } else {
                "4294967296"
            };
            for invalid in [
                json!(1.5),
                json!("1"),
                json!(true),
                serde_json::from_str(maximum).unwrap(),
                json!(if signed { -2_147_483_649_i64 } else { -1 }),
            ] {
                let mut changed = baseline.clone();
                *changed.pointer_mut(&pointer).unwrap() = invalid.clone();
                assert!(
                    parse(&changed).is_err(),
                    "无效整数未拒绝：{pointer}={invalid}"
                );
            }
        }
    }
}

#[test]
fn 版本依据标识符与结构格式错误明确失败() {
    for (field, invalids) in [
        ("schema_version", vec![json!(0), json!(2), json!("1")]),
        (
            "rules_version",
            vec![
                json!("2"),
                json!("0.0"),
                json!("02.0"),
                json!("2.00"),
                json!("2.0.0"),
                json!("2.65536"),
                json!(2),
            ],
        ),
        (
            "basis",
            vec![json!("original"), json!("Development"), json!("")],
        ),
        (
            "level",
            vec![
                json!(""),
                json!(" day"),
                json!("day/night"),
                json!("白天"),
                json!(7),
            ],
        ),
        ("rules", vec![json!([]), json!("rules")]),
    ] {
        for invalid in invalids {
            let mut changed = source();
            changed[field] = invalid.clone();
            assert!(
                parse(&changed).is_err(),
                "无效字段未拒绝：{field}={invalid}"
            );
        }
    }
    for invalid in ["", "{", "[]", "null", "{}", "{} {}"] {
        assert!(serde_json::from_str::<DayLevelDefinition>(invalid).is_err());
    }
    for invalid in [json!({}), json!(1), json!([null]), json!([[]])] {
        let mut changed = source();
        changed["rules"]["wave"] = invalid;
        assert!(parse(&changed).is_err());
    }
}

#[test]
fn 格式层保留规则数值范围和波次顺序交由核心验证() {
    let mut changed = source();
    changed["rules_version"] = json!("3.1");
    changed["ticks_per_second"] = json!(0);
    changed["rules"]["sunflower_cost"] = json!(0);
    changed["rules"]["peashooter_cost"] = json!(u32::MAX);
    changed["rules"]["shoot_interval"] = json!(u64::MAX);
    changed["rules"]["defeat_x"] = json!(i32::MIN);
    changed["rules"]["wave"] = json!([
        {"at": 2, "row": 4, "x": 10},
        {"at": 2, "row": 1, "x": -20},
        {"at": 1, "row": 255, "x": 30}
    ]);
    let definition = parse(&changed).unwrap();
    assert_eq!(definition.rules_version, RulesVersion::new(3, 1).unwrap());
    assert_eq!(definition.rules.shoot_interval, u64::MAX);
    assert_eq!(
        definition
            .rules
            .wave
            .iter()
            .map(|spawn| (spawn.at, spawn.row, spawn.x))
            .collect::<Vec<_>>(),
        vec![(2, 4, 10), (2, 1, -20), (1, 255, 30)]
    );
    changed["rules"]["wave"] = json!([]);
    assert!(parse(&changed).unwrap().rules.wave.is_empty());
}
