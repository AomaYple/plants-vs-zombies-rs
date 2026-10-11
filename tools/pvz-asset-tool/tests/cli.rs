use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    process::{Command, Output},
};
use tempfile::TempDir;

const REANIM: &[u8] = b"<fps>12.5</fps><track><name>z</name><t><x>1</x></t><t/></track><track><name>a</name><t><i/></t></track>\n";

fn run(repository: &TempDir, arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_pvz-asset-tool"))
        .current_dir(repository.path())
        .args(arguments)
        .output()
        .unwrap()
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).unwrap()
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).unwrap()
}

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn fixture() -> TempDir {
    let repository = tempfile::tempdir().unwrap();
    fs::create_dir_all(repository.path().join("assets/original/reanim")).unwrap();
    let files = [
        ("z.txt", b"z\n".as_slice()),
        ("reanim/PeaShooter.reanim", REANIM),
        ("a.txt", b"a\n".as_slice()),
        ("main.pak", b"123456789".as_slice()),
    ];
    let records = files.iter().map(|(path, bytes)| {
        fs::write(repository.path().join("assets/original").join(path), bytes).unwrap();
        let mut record = json!({"path":path,"kind":if *path == "main.pak" { "package" } else { "text" },"size":bytes.len(),"source_size":bytes.len(),"sha256":hash(bytes),"source_sha256":hash(bytes)});
        if *path != "main.pak" { record["source_encoding"] = json!("UTF-8"); }
        record
    }).collect::<Vec<_>>();
    // 原创九字节夹具仅满足清单元数据合同，不代表可解包的 PAK。
    let manifest = json!({"schema_version":1,"baseline_id":"cli-fixture","root":"assets/original","text_normalization":"原创 UTF-8/LF 夹具","source_archive":{"name":"main.pak","size":9,"sha256":hash(b"123456789")},"package":{"path":"main.pak","entry_count":3,"index_bytes":8,"size":9,"sha256":hash(b"123456789"),"extraction_check":"原创夹具"},"files":records});
    fs::write(
        repository.path().join("assets/manifest.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    repository
}

fn edit_manifest(repository: &TempDir, edit: impl FnOnce(&mut Value)) {
    let path = repository.path().join("assets/manifest.json");
    let mut manifest = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    edit(&mut manifest);
    fs::write(path, serde_json::to_vec(&manifest).unwrap()).unwrap();
}

#[test]
fn 帮助无需资源并正常退出() {
    let repository = tempfile::tempdir().unwrap();
    for arguments in [
        &["--help"][..],
        &["audit", "--help"],
        &["inspect-reanim", "--help"],
    ] {
        let output = run(&repository, arguments);
        assert_eq!(output.status.code(), Some(0));
        assert!(output.stderr.is_empty());
        assert!(stdout(&output).contains("资源"));
    }
    let version = run(&repository, &["--version"]);
    assert_eq!(version.status.code(), Some(0));
    assert!(stdout(&version).starts_with("pvz-asset-tool "));
}

#[test]
fn 成功审计输出稳定的显式_json_结构() {
    let repository = fixture();
    let output = run(&repository, &["audit", "--format", "json"]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert!(output.stderr.is_empty());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        report,
        json!({"schema_version":1,"command":"audit","status":"ok","baseline_id":"cli-fixture","summary":{"record_count":4,"unpacked_file_count":3,"package_count":1,"diagnostic_count":0},"diagnostics":[],"error":null})
    );
    assert!(
        stdout(&output).find("schema_version").unwrap() < stdout(&output).find("command").unwrap()
    );
    assert_eq!(
        output.stdout,
        run(&repository, &["audit", "--format", "json"]).stdout
    );
}

#[test]
fn 文本审计默认输出中文且不修改输入() {
    let repository = fixture();
    let paths = [
        "assets/manifest.json",
        "assets/original/main.pak",
        "assets/original/a.txt",
        "assets/original/z.txt",
        "assets/original/reanim/PeaShooter.reanim",
    ];
    let before = paths.map(|path| fs::read(repository.path().join(path)).unwrap());
    let output = run(&repository, &["audit"]);
    assert_eq!(output.status.code(), Some(0));
    assert!(stdout(&output).contains("资源审计通过"));
    assert!(stdout(&output).contains("4 条记录"));
    assert_eq!(
        before,
        paths.map(|path| fs::read(repository.path().join(path)).unwrap())
    );
}

#[test]
fn 缺失清单在两种格式中显式失败() {
    let repository = tempfile::tempdir().unwrap();
    let text = run(&repository, &["audit", "--format", "text"]);
    assert_eq!(text.status.code(), Some(1));
    assert!(text.stdout.is_empty());
    assert!(stderr(&text).contains("manifest_read"));
    let json = run(&repository, &["audit", "--format", "json"]);
    assert_eq!(json.status.code(), Some(1));
    let report: Value = serde_json::from_slice(&json.stdout).unwrap();
    assert_eq!(report["error"]["code"], "manifest_read");
    assert_eq!(report["summary"], Value::Null);
    assert!(json.stderr.is_empty());
}

#[test]
fn 无效清单在两种格式中显式失败且不泄露宿主路径() {
    let repository = fixture();
    fs::write(
        repository.path().join("assets/manifest.json"),
        br#"{"root":"C:\\Users\\PRIVATE\\input"}"#,
    )
    .unwrap();
    for format in ["text", "json"] {
        let output = run(&repository, &["audit", "--format", format]);
        assert_eq!(output.status.code(), Some(1));
        let combined = format!("{}{}", stdout(&output), stderr(&output));
        assert!(combined.contains("manifest_invalid"));
        assert!(!combined.contains("PRIVATE"));
        assert!(!combined.contains(&repository.path().to_string_lossy().to_string()));
    }
}

#[test]
fn 清单中的无效绝对路径和来源字段不回显() {
    for field in [
        "root",
        "file_path",
        "kind",
        "package_path",
        "source_name",
        "baseline_id",
    ] {
        let repository = fixture();
        edit_manifest(&repository, |manifest| match field {
            "root" => manifest["root"] = json!("C:\\PRIVATE\\assets"),
            "file_path" => manifest["files"][0]["path"] = json!("/PRIVATE/a.txt"),
            "kind" => manifest["files"][0]["kind"] = json!("C:\\PRIVATE\\type"),
            "package_path" => manifest["package"]["path"] = json!("/PRIVATE/main.pak"),
            "source_name" => manifest["source_archive"]["name"] = json!("/PRIVATE/main.pak"),
            "baseline_id" => manifest["baseline_id"] = json!("/PRIVATE/baseline"),
            _ => unreachable!(),
        });
        let output = run(&repository, &["audit", "--format", "json"]);
        assert_eq!(output.status.code(), Some(1), "字段：{field}");
        assert!(!stdout(&output).contains("PRIVATE"), "字段：{field}");
        assert_eq!(
            serde_json::from_slice::<Value>(&output.stdout).unwrap()["error"]["code"],
            "manifest_invalid"
        );
    }
}

#[test]
fn 哈希缺失大小文本和未登记诊断按路径稳定输出() {
    let repository = fixture();
    fs::write(repository.path().join("assets/original/z.txt"), b"x\n").unwrap();
    fs::remove_file(repository.path().join("assets/original/a.txt")).unwrap();
    fs::write(
        repository
            .path()
            .join("assets/original/reanim/PeaShooter.reanim"),
        b"bad\r\n",
    )
    .unwrap();
    fs::write(repository.path().join("assets/original/b.txt"), b"b\n").unwrap();
    let output = run(&repository, &["audit", "--format", "json"]);
    assert_eq!(output.status.code(), Some(1));
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    let diagnostics = report["diagnostics"].as_array().unwrap();
    let codes = diagnostics
        .iter()
        .map(|diagnostic| diagnostic["code"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        codes,
        [
            "missing",
            "unexpected",
            "size_mismatch",
            "invalid_text",
            "hash_mismatch"
        ]
    );
    let paths = diagnostics
        .iter()
        .map(|diagnostic| diagnostic["path"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        paths,
        [
            "a.txt",
            "b.txt",
            "reanim/PeaShooter.reanim",
            "reanim/PeaShooter.reanim",
            "z.txt"
        ]
    );
    assert_eq!(diagnostics[2]["expected_size"], REANIM.len());
    assert_eq!(diagnostics[2]["actual_size"], 5);
    assert_eq!(report["status"], "error");
    assert!(report["error"].is_null());
    assert_eq!(
        output.stdout,
        run(&repository, &["audit", "--format", "json"]).stdout
    );
    let text = run(&repository, &["audit", "--format", "text"]);
    assert_eq!(text.status.code(), Some(1));
    assert!(stdout(&text).contains("hash_mismatch"));
}

#[test]
fn 参数错误不回显绝对路径并返回二() {
    let repository = fixture();
    for arguments in [
        &[][..],
        &["audit", "--format", "C:\\PRIVATE\\json"][..],
        &["audit", "--/PRIVATE/unknown"],
        &["/PRIVATE/command"],
        &["inspect-reanim", "/PRIVATE/PeaShooter"],
        &["inspect-reanim", "C:\\PRIVATE\\PeaShooter"],
        &["inspect-reanim", "../PeaShooter"],
        &["inspect-reanim", "reanim/PeaShooter"],
        &["inspect-reanim", "--/PRIVATE/unknown"],
        &["inspect-reanim"],
        &["audit", "extra"],
    ] {
        let output = run(&repository, arguments);
        assert_eq!(output.status.code(), Some(2), "参数：{arguments:?}");
        assert!(output.stdout.is_empty());
        assert!(stderr(&output).contains("参数"));
        assert!(!stderr(&output).contains("PRIVATE"));
        assert!(!stderr(&output).contains("../PeaShooter"));
    }
}

#[test]
fn 文件被目录替代时报告稳定_io_诊断() {
    let repository = fixture();
    let path = repository.path().join("assets/original/a.txt");
    fs::remove_file(&path).unwrap();
    fs::create_dir(&path).unwrap();
    let output = run(&repository, &["audit", "--format", "json"]);
    assert_eq!(output.status.code(), Some(1));
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["diagnostics"][0]["code"], "io");
    assert_eq!(report["diagnostics"][0]["path"], "a.txt");
    assert_eq!(report["summary"]["diagnostic_count"], 1);
}

#[test]
fn inspect在清单或登记文件缺失时失败() {
    let repository = tempfile::tempdir().unwrap();
    let output = run(&repository, &["inspect-reanim", "PeaShooter"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("manifest_read"));
    let repository = fixture();
    fs::remove_file(
        repository
            .path()
            .join("assets/original/reanim/PeaShooter.reanim"),
    )
    .unwrap();
    let output = run(&repository, &["inspect-reanim", "PeaShooter"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("asset_read"));
    assert!(output.stdout.is_empty());
}

#[test]
fn inspect读取登记的_reanim_并保留轨道顺序() {
    let repository = fixture();
    let output = run(&repository, &["inspect-reanim", "PeaShooter"]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    let text = stdout(&output);
    assert!(text.contains("reanim/PeaShooter.reanim"));
    assert!(text.contains("12.5"));
    assert!(text.contains("z：2 帧"));
    assert!(text.contains("a：1 帧"));
    assert!(text.find("z：2 帧").unwrap() < text.find("a：1 帧").unwrap());
}

#[test]
fn inspect拒绝未登记或指纹错误的输入() {
    let repository = fixture();
    let missing = run(&repository, &["inspect-reanim", "Unknown"]);
    assert_eq!(missing.status.code(), Some(1));
    assert!(stderr(&missing).contains("asset_unregistered"));
    fs::write(
        repository
            .path()
            .join("assets/original/reanim/PeaShooter.reanim"),
        b"<fps>12</fps>\n",
    )
    .unwrap();
    let invalid = run(&repository, &["inspect-reanim", "PeaShooter"]);
    assert_eq!(invalid.status.code(), Some(1));
    assert!(stderr(&invalid).contains("asset_invalid"));
    assert!(invalid.stdout.is_empty());
}

#[test]
fn inspect拒绝合法指纹但无效的模型() {
    let repository = fixture();
    let bytes = b"<PRIVATE:C><fps>12</fps></PRIVATE:C>\n";
    fs::write(
        repository
            .path()
            .join("assets/original/reanim/PeaShooter.reanim"),
        bytes,
    )
    .unwrap();
    edit_manifest(&repository, |manifest| {
        let record = &mut manifest["files"][1];
        record["size"] = json!(bytes.len());
        record["source_size"] = json!(bytes.len());
        record["sha256"] = json!(hash(bytes));
        record["source_sha256"] = json!(hash(bytes));
    });
    let output = run(&repository, &["inspect-reanim", "PeaShooter"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("reanim_invalid"));
    assert!(!stderr(&output).contains("PRIVATE"));
}
