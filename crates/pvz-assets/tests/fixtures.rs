use pvz_assets::{AssetError, AssetRoot, Diagnostic, EventKind, parse_particle, parse_reanim};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::fs;
use tempfile::TempDir;

fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn repository() -> TempDir {
    let repository = tempfile::tempdir().unwrap();
    fs::create_dir_all(repository.path().join("assets/original")).unwrap();
    repository
}

type Record<'a> = (&'a str, Option<&'a [u8]>, &'a [u8]);

fn write_manifest(repository: &TempDir, records: &[Record<'_>]) {
    let mut files = records
        .iter()
        .map(|(path, existing, expected)| {
            if let Some(bytes) = existing {
                let file = repository.path().join("assets/original").join(path);
                if let Some(parent) = file.parent() {
                    fs::create_dir_all(parent).unwrap();
                }
                fs::write(file, bytes).unwrap();
            }
            json!({
                "path": path,
                "kind": "text",
                "source_size": expected.len(),
                "source_sha256": hash(expected),
                "size": expected.len(),
                "sha256": hash(expected),
                "source_encoding": "UTF-8"
            })
        })
        .collect::<Vec<_>>();
    // 原创的九字节包夹具只用于来源合同，不宣称是可解包的原版 PAK。
    let package_bytes = b"123456789";
    fs::write(
        repository.path().join("assets/original/main.pak"),
        package_bytes,
    )
    .unwrap();
    files.push(json!({"path":"main.pak","kind":"package","size":9,"source_size":9,"sha256":hash(package_bytes),"source_sha256":hash(package_bytes)}));
    let manifest = json!({
        "schema_version": 1,
        "baseline_id":"original-test",
        "root": "assets/original",
        "text_normalization":"原创文本采用 UTF-8 和 LF",
        "source_archive":{"name":"main.pak","size":9,"sha256":hash(package_bytes)},
        "package":{"path":"main.pak","size":9,"sha256":hash(package_bytes),"index_bytes":8,"entry_count":records.len(),"extraction_check":"原创夹具"},
        "files": files
    });
    fs::write(
        repository.path().join("assets/manifest.json"),
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
}

#[test]
fn 有效原创清单可以审计通过() {
    let repository = repository();
    let content = b"alpha\n";
    write_manifest(&repository, &[("a.txt", Some(content), content)]);

    let index = AssetRoot::open(repository.path()).unwrap();
    let report = index.audit();

    assert_eq!(index.read("a.txt").unwrap(), content);
    assert!(report.is_clean());
    assert!(report.diagnostics().is_empty());
}

#[test]
fn 缺失和哈希错误按路径稳定排序() {
    let repository = repository();
    let expected_a = b"expected\n";
    let actual_a = b"mismatch\n";
    write_manifest(
        &repository,
        &[
            ("z.txt", None, b"missing\n"),
            ("a.txt", Some(actual_a), expected_a),
        ],
    );

    let report = AssetRoot::open(repository.path()).unwrap().audit();

    assert_eq!(
        report.diagnostics(),
        &[
            Diagnostic::HashMismatch {
                path: "a.txt".to_owned(),
            },
            Diagnostic::Missing {
                path: "z.txt".to_owned(),
            },
        ]
    );
    assert_eq!(report.diagnostics()[0].kind(), EventKind::Error);
}

#[test]
fn 清单路径大小写冲突会在打开时拒绝() {
    let repository = repository();
    let content = b"x\n";
    write_manifest(
        &repository,
        &[
            ("A.txt", Some(content), content),
            ("a.txt", Some(content), content),
        ],
    );

    let error = AssetRoot::open(repository.path()).unwrap_err();

    assert!(matches!(error, AssetError::PathCollision { .. }));
}

#[test]
fn 缺失或不支持的清单版本会被拒绝() {
    let repository = repository();
    assert!(matches!(
        AssetRoot::open(repository.path()),
        Err(AssetError::Io { .. })
    ));

    let content = b"x\n";
    write_manifest(&repository, &[("a.txt", Some(content), content)]);
    let manifest_path = repository.path().join("assets/manifest.json");
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
    manifest["schema_version"] = json!(2);
    fs::write(&manifest_path, serde_json::to_vec(&manifest).unwrap()).unwrap();
    assert!(matches!(
        AssetRoot::open(repository.path()),
        Err(AssetError::UnsupportedSchema { version: 2 })
    ));
}

#[test]
fn reanim和粒子原创夹具可以解析() {
    let reanim = "<root><fps>12</fps><track><name>idle</name><t><f>0</f></t><t></t></track></root>";
    let animation = parse_reanim(reanim).unwrap();
    assert_eq!(animation.fps(), 12.0);
    assert_eq!(animation.tracks()[0].name(), "idle");
    assert_eq!(animation.tracks()[0].frame_count(), 2);

    let particle = parse_particle(
        "<Emitter><Name>原创</Name><EmitterType>Box</EmitterType><Image>IMAGE_TEST</Image><SystemDuration>100</SystemDuration><Field><FieldType>Position</FieldType></Field></Emitter>",
    )
    .unwrap();
    assert_eq!(particle.name(), "原创");
    assert_eq!(particle.image(), "IMAGE_TEST");
    assert_eq!(particle.emitter_type(), "Box");
    assert_eq!(particle.system_duration(), Some(100.0));
    assert_eq!(particle.fields(), &["Position".to_owned()]);
}

#[test]
fn 畸形_xml会被拒绝() {
    assert!(matches!(
        parse_reanim("<root>"),
        Err(AssetError::Xml { .. })
    ));
    assert!(matches!(
        parse_particle("<Emitter>"),
        Err(AssetError::Xml { .. })
    ));
    assert!(parse_reanim("<root><track><name>idle</name></track></root>").is_err());
    assert!(parse_particle("<Emitter><Field><X>缺字段</X></Field></Emitter>").is_err());
}

#[test]
fn 版本化资源清单可以只读审计() {
    let repository = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let index = AssetRoot::open(&repository).unwrap();
    let report = index.audit();

    assert_eq!(index.len(), 3_229);
    assert!(
        report.is_clean(),
        "资源审计诊断：{:?}",
        report.diagnostics()
    );
}
