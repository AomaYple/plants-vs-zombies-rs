use pvz_data::{
    AssetId, LevelId, PlantId, ReferenceBaselineId, RulesVersion, SchemaVersion, ZombieId,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct OriginalFixture {
    asset: AssetId,
    plant: PlantId,
    zombie: ZombieId,
    level: LevelId,
    schema_version: SchemaVersion,
    rules_version: RulesVersion,
    reference_baseline: ReferenceBaselineId,
}

#[test]
fn 原创夹具可以稳定序列化和反序列化() {
    let fixture = OriginalFixture {
        asset: AssetId::new("images/Plants/PeaShooter.png").unwrap(),
        plant: PlantId::new("peashooter").unwrap(),
        zombie: ZombieId::new("zombie").unwrap(),
        level: LevelId::new("day-1").unwrap(),
        schema_version: SchemaVersion::current(),
        rules_version: RulesVersion::new(1, 0).unwrap(),
        reference_baseline: ReferenceBaselineId::new("goty-zh-a7ab7df8-7f1e824e").unwrap(),
    };

    let encoded = serde_json::to_string(&fixture).unwrap();
    assert_eq!(
        encoded,
        r#"{"asset":"images/Plants/PeaShooter.png","plant":"peashooter","zombie":"zombie","level":"day-1","schema_version":1,"rules_version":"1.0","reference_baseline":"goty-zh-a7ab7df8-7f1e824e"}"#
    );
    assert_eq!(
        serde_json::from_str::<OriginalFixture>(&encoded).unwrap(),
        fixture
    );
}

#[test]
fn 不支持的_schema_版本会被拒绝() {
    assert!(SchemaVersion::new(2).is_err());
    assert!(serde_json::from_str::<SchemaVersion>("2").is_err());
}

#[test]
fn 强类型_id使用稳定字符串格式() {
    assert_eq!(
        serde_json::to_string(&AssetId::new("a/b.png").unwrap()).unwrap(),
        "\"a/b.png\""
    );
    assert_eq!(
        serde_json::to_string(&PlantId::new("sunflower").unwrap()).unwrap(),
        "\"sunflower\""
    );
    assert_eq!(
        serde_json::to_string(&ZombieId::new("zombie").unwrap()).unwrap(),
        "\"zombie\""
    );
    assert_eq!(
        serde_json::to_string(&LevelId::new("day-1").unwrap()).unwrap(),
        "\"day-1\""
    );
}

#[test]
fn 版本类型拒绝无效值和格式() {
    assert!(RulesVersion::new(0, 1).is_err());
    assert!(serde_json::from_str::<RulesVersion>(r#""1""#).is_err());
    assert!(serde_json::from_str::<RulesVersion>(r#""1.0.0""#).is_err());
    assert!(serde_json::from_str::<RulesVersion>(r#""0.1""#).is_err());
}

#[test]
fn 标识符拒绝空值和不安全路径() {
    assert!(AssetId::new("").is_err());
    assert!(AssetId::new("../outside.png").is_err());
    assert!(AssetId::new("C:/outside.png").is_err());
    assert!(PlantId::new("pea shooter").is_err());
}
