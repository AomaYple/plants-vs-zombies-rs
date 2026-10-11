use pvz_assets::{
    AnimationFrame, AssetError, AssetKind, AssetRoot, Manifest, parse_particle, parse_reanim,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs;

#[test]
fn 稀疏帧区分缺省与显式空字符串并保留零值() {
    let clip = parse_reanim(
        "<fps>12</fps><track><name>原创</name><t/><t><i/><font></font><text/><x>0</x><f>0</f><a>0</a></t></track>",
    )
    .unwrap();
    assert_eq!(
        clip.tracks()[0].frames(),
        &[
            AnimationFrame::default(),
            AnimationFrame {
                i: Some(String::new()),
                font: Some(String::new()),
                text: Some(String::new()),
                x: Some(0.0),
                f: Some(0.0),
                a: Some(0.0),
                ..AnimationFrame::default()
            }
        ]
    );
}

#[test]
fn cdata成功路径保留字面分隔符而普通文本解码实体() {
    let clip = parse_reanim(
        "<fps><![CDATA[12.5]]></fps><track><name><![CDATA[原创<&]]></name><t><text>前&amp;<![CDATA[<&]]>后</text><i><![CDATA[IMAGE_<A>&]]></i></t></track>",
    )
    .unwrap();
    assert_eq!(clip.fps(), 12.5);
    assert_eq!(clip.tracks()[0].name(), "原创<&");
    assert_eq!(
        clip.tracks()[0].frames()[0].text.as_deref(),
        Some("前&<&后")
    );
    assert_eq!(
        clip.tracks()[0].frames()[0].i.as_deref(),
        Some("IMAGE_<A>&")
    );
    let particle = parse_particle(
        "<Emitter><Name><![CDATA[原创<&]]></Name><SystemDuration><![CDATA[100]]></SystemDuration></Emitter>",
    )
    .unwrap();
    assert_eq!(particle.name(), "原创<&");
    assert_eq!(particle.system_duration(), Some(100.0));
    assert_eq!(particle.emitters()[0].image(), None);
    assert_eq!(particle.emitters()[0].emitter_type(), None);
}

#[test]
fn 重复_field和_systemfield保持原文顺序及独立子树() {
    let particle = parse_particle(
        "<Emitter><Field><FieldType>Position</FieldType><X>1</X></Field><SystemField><Y>2</Y><FieldType>Acceleration</FieldType></SystemField><Field><FieldType>Friction</FieldType><Y>3</Y></Field><SystemField><FieldType>Velocity</FieldType><X>4</X></SystemField></Emitter>",
    )
    .unwrap();
    let emitter = &particle.emitters()[0];
    let properties = emitter
        .properties()
        .iter()
        .map(|property| {
            (
                property.name(),
                property
                    .children()
                    .iter()
                    .map(|child| (child.name(), child.text()))
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        properties,
        [
            ("Field", vec![("FieldType", "Position"), ("X", "1")]),
            (
                "SystemField",
                vec![("Y", "2"), ("FieldType", "Acceleration")]
            ),
            ("Field", vec![("FieldType", "Friction"), ("Y", "3")]),
            ("SystemField", vec![("FieldType", "Velocity"), ("X", "4")]),
        ]
    );
    assert_eq!(particle.fields(), ["Position", "Friction"]);
    assert_eq!(
        emitter.property("Field").unwrap().children()[0].text(),
        "Position"
    );
    assert_eq!(
        emitter.property("SystemField").unwrap().children()[0].text(),
        "2"
    );
    assert_eq!(emitter.name(), None);
}

#[test]
fn 粒子允许显示属性缺省但拒绝显式空标量() {
    let particle =
        parse_particle("<Emitter><SystemDuration>100</SystemDuration></Emitter>").unwrap();
    let emitter = &particle.emitters()[0];
    assert_eq!(emitter.name(), None);
    assert_eq!(emitter.image(), None);
    assert_eq!(emitter.emitter_type(), None);
    for empty in ["<Name/>", "<Image></Image>", "<EmitterType/>"] {
        assert!(
            parse_particle(&format!(
                "<Emitter>{empty}<SystemDuration>100</SystemDuration></Emitter>"
            ))
            .is_err()
        );
    }
}

#[test]
fn 截断和未知_xml不能伪装成成功() {
    for source in [
        "<root><fps>12</fps><track><name>idle</name><t/></track>",
        "<root><fps>12</fps><fps>30</fps><track><name>idle</name><t/></track></root>",
        "<root><fps>12</fps><track><name>idle</name><t><unknown>9</unknown></t></track></root>",
        "<root><fps>12</fps><track><name>idle</name><t><name>bad</name></t></track></root>",
    ] {
        assert!(parse_reanim(source).is_err(), "不应接受：{source}");
    }
    for source in [
        "<Emitter><Name>a</Name><Image>b</Image><EmitterType>Box</EmitterType>",
        "<Emitter><Name>a</Name><Name>b</Name><Image>b</Image><EmitterType>Box</EmitterType></Emitter>",
        "<Emitter><Name>a</Name><Image>b</Image><EmitterType>Box</EmitterType><Unknown>1</Unknown></Emitter>",
        "<Emitter><Name>a</Name><Image>b</Image><EmitterType>Box</EmitterType><Field><Image>bad</Image></Field></Emitter>",
    ] {
        assert!(parse_particle(source).is_err(), "不应接受：{source}");
    }
}

#[test]
fn 无来源合同的清单被拒绝() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir(root.path().join("assets")).unwrap();
    fs::write(
        root.path().join("assets/manifest.json"),
        br#"{"schema_version":1,"root":"assets/original","files":[]}"#,
    )
    .unwrap();
    assert!(AssetRoot::open(root.path()).is_err());
}

#[test]
fn 打开失败诊断不暴露绝对路径() {
    let root = tempfile::tempdir().unwrap();
    let error = AssetRoot::open(root.path()).unwrap_err();
    assert!(
        !error
            .to_string()
            .contains(&root.path().display().to_string())
    );
    assert!(error.to_string().contains("assets/manifest.json"));
    assert!(matches!(
        error,
        AssetError::Io {
            kind: std::io::ErrorKind::NotFound,
            ..
        }
    ));
}

fn 清单() -> Value {
    let hash = format!("{:x}", Sha256::digest(b"123456789"));
    json!({"schema_version":1,"baseline_id":"original-test","root":"assets/original",
        "text_normalization":"原创测试规范化规则",
        "source_archive":{"name":"main.pak","size":9,"sha256":hash},
        "package":{"path":"main.pak","size":9,"sha256":hash,"index_bytes":8,"entry_count":1,"extraction_check":"原创夹具"},
        "files":[{"path":"main.pak","kind":"package","size":9,"sha256":hash,"source_size":9,"source_sha256":hash},
                 {"path":"a.txt","kind":"text","size":2,"sha256":format!("{:x}",Sha256::digest(b"a\n")),"source_size":2,"source_sha256":format!("{:x}",Sha256::digest(b"a\n")),"source_encoding":"UTF-8"}]})
}

#[test]
fn 纯字节清单支持浏览器记录与内容校验() {
    let manifest = Manifest::parse(&serde_json::to_vec(&清单()).unwrap()).unwrap();
    assert_eq!(manifest.baseline_id(), "original-test");
    assert_eq!(manifest.records().len(), 2);
    assert_eq!(manifest.record("a.txt").unwrap().kind(), AssetKind::Text);
    assert!(manifest.validate_bytes("a.txt", b"a\n").unwrap().is_clean());
    assert!(!manifest.validate_bytes("a.txt", b"z\n").unwrap().is_clean());
    assert!(manifest.validate_bytes("unknown.txt", b"").is_err());
}

#[test]
fn 清单来源包分类和未知字段合同严格校验() {
    for (pointer, value) in [
        ("/baseline_id", json!("")),
        ("/source_archive/sha256", json!("ABC")),
        ("/files/1/source_encoding", json!("GBK")),
        ("/files/0/source_size", json!(10)),
        ("/package/entry_count", json!(2)),
        ("/package/index_bytes", json!(9)),
        ("/package/sha256", json!("0".repeat(64))),
        ("/files/1/path", json!("a.exe")),
        ("/files/1/path", json!("C:/secret.txt")),
        ("/files/1/path", json!("../a.txt")),
    ] {
        let mut value_manifest = 清单();
        *value_manifest.pointer_mut(pointer).unwrap() = value;
        assert!(
            Manifest::parse(&serde_json::to_vec(&value_manifest).unwrap()).is_err(),
            "应拒绝：{pointer}"
        );
    }
    let mut value = 清单();
    value["unrecognized"] = json!(true);
    assert!(Manifest::parse(&serde_json::to_vec(&value).unwrap()).is_err());
}

#[test]
fn 稀疏动画帧保留仿射资源和文本字段() {
    let clip = parse_reanim(include_str!(
        "../../../tests/fixtures/assets-minimal/sparse-frames.reanim"
    ))
    .unwrap();
    let frames = clip.tracks()[0].frames();
    assert_eq!(frames.len(), 2);
    assert_eq!(
        (frames[0].x, frames[0].y, frames[0].sx, frames[0].sy),
        (Some(2.0), Some(-3.0), Some(0.5), Some(2.0))
    );
    assert_eq!(
        (frames[0].kx, frames[0].ky, frames[0].f, frames[0].a),
        (Some(10.0), Some(20.0), Some(-1.0), Some(0.2))
    );
    assert_eq!(frames[0].i.as_deref(), Some("IMAGE_A"));
    assert_eq!(frames[0].font.as_deref(), Some("FONT_A"));
    assert_eq!(frames[0].text.as_deref(), Some("a&b"));
    assert_eq!(frames[1].x, None);
}

#[test]
fn 原版小数帧率和显式文本空白不会丢失() {
    let clip =
        parse_reanim("<fps>10.5</fps><track><name>a</name><t><text>  文本  </text></t></track>")
            .unwrap();
    assert_eq!(f64::from(clip.fps()), 10.5);
    assert_eq!(
        clip.tracks()[0].frames()[0].text.as_deref(),
        Some("  文本  ")
    );
}

#[test]
fn 多发射器独立保留曲线与字段树() {
    let system = parse_particle(include_str!(
        "../../../tests/fixtures/assets-minimal/multiple-emitters.xml"
    ))
    .unwrap();
    assert_eq!(system.emitters().len(), 2);
    let first = &system.emitters()[0];
    assert_eq!(first.name(), Some("a"));
    assert_eq!(
        first.property("ParticleScale").unwrap().text(),
        "[.4 .6] [.8 1.2]"
    );
    assert_eq!(
        first.property("Field").unwrap().children()[1].text(),
        "0,40 .1"
    );
    assert_eq!(system.emitters()[1].name(), None);
    assert_eq!(
        system.emitters()[1]
            .property("SystemDuration")
            .unwrap()
            .text(),
        "[20 30]"
    );
}

#[test]
fn 根目录枚举错误不可静默通过() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir(root.path().join("assets")).unwrap();
    fs::write(
        root.path().join("assets/manifest.json"),
        serde_json::to_vec(&清单()).unwrap(),
    )
    .unwrap();
    fs::write(root.path().join("assets/original"), "目录被替换为文件").unwrap();
    let report = AssetRoot::open(root.path()).unwrap().audit();
    assert!(report.diagnostics().iter().any(|d| d.path() == "."));
}

#[test]
fn 所有登记动画和粒子文本可只读解析() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let manifest = Manifest::parse(&fs::read(root.join("assets/manifest.json")).unwrap()).unwrap();
    let mut counts = [0, 0];
    for record in manifest.records() {
        let path = record.id().as_str();
        if path.ends_with(".reanim") {
            parse_reanim(&fs::read_to_string(root.join("assets/original").join(path)).unwrap())
                .unwrap_or_else(|e| panic!("{path}：{e}"));
            counts[0] += 1;
        } else if path.starts_with("particles/") && path.ends_with(".xml") {
            parse_particle(&fs::read_to_string(root.join("assets/original").join(path)).unwrap())
                .unwrap_or_else(|e| panic!("{path}：{e}"));
            counts[1] += 1;
        }
    }
    assert_eq!(counts, [146, 112]);
}

fn 写宿主夹具(root: &std::path::Path, manifest: &Value) {
    fs::create_dir_all(root.join("assets/original")).unwrap();
    fs::write(
        root.join("assets/manifest.json"),
        serde_json::to_vec(manifest).unwrap(),
    )
    .unwrap();
    fs::write(root.join("assets/original/main.pak"), b"123456789").unwrap();
    fs::write(root.join("assets/original/a.txt"), b"a\n").unwrap();
}

#[cfg(any(unix, windows))]
fn 目录链接(target: &std::path::Path, link: &std::path::Path) {
    #[cfg(unix)]
    std::os::unix::fs::symlink(target, link).unwrap();
    #[cfg(windows)]
    {
        // Junction 不需要 Windows 符号链接权限；全部输入在独立临时目录中。
        let output = std::process::Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(link.to_str().unwrap().replace('/', "\\"))
            .arg(target.to_str().unwrap().replace('/', "\\"))
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "创建原创目录链接失败：{:?}：{:?}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
#[cfg(any(unix, windows))]
fn 已登记目录链接不能读取根外文件() {
    let root = tempfile::tempdir().unwrap();
    let external = tempfile::tempdir().unwrap();
    fs::write(external.path().join("a.txt"), b"a\n").unwrap();
    let mut manifest = 清单();
    manifest["files"][1]["path"] = json!("linked/a.txt");
    写宿主夹具(root.path(), &manifest);
    fs::remove_file(root.path().join("assets/original/a.txt")).unwrap();
    目录链接(external.path(), &root.path().join("assets/original/linked"));
    let index = AssetRoot::open(root.path()).unwrap();
    assert!(matches!(
        index.read("linked/a.txt"),
        Err(AssetError::Io {
            kind: std::io::ErrorKind::InvalidInput,
            ..
        })
    ));
    let report = index.audit();
    assert!(!report.is_clean());
    assert!(report.diagnostics().iter().any(|d| d.path() == "linked"));
    assert!(!format!("{:?}", report).contains(&external.path().display().to_string()));
}

#[test]
#[cfg(any(unix, windows))]
fn 根目录链接和未登记链接都被拒绝() {
    let root = tempfile::tempdir().unwrap();
    let external = tempfile::tempdir().unwrap();
    写宿主夹具(root.path(), &清单());
    目录链接(external.path(), &root.path().join("assets/original/extra"));
    let report = AssetRoot::open(root.path()).unwrap().audit();
    assert!(report.diagnostics().iter().any(|d| matches!(d,pvz_assets::Diagnostic::Io {path,kind:std::io::ErrorKind::InvalidInput,..} if path=="extra")));

    let linked_root = tempfile::tempdir().unwrap();
    fs::create_dir(linked_root.path().join("assets")).unwrap();
    fs::write(
        linked_root.path().join("assets/manifest.json"),
        serde_json::to_vec(&清单()).unwrap(),
    )
    .unwrap();
    目录链接(
        &root.path().join("assets/original"),
        &linked_root.path().join("assets/original"),
    );
    let report = AssetRoot::open(linked_root.path()).unwrap().audit();
    assert!(report.diagnostics().iter().any(|d| d.path() == "."));
}

#[test]
fn 文本字节合同不能靠匹配哈希绕过() {
    for bytes in [b"a\r\n".as_slice(), b"\xef\xbb\xbfa\n", b"\xff\n", b"a"] {
        let mut value = 清单();
        value["files"][1]["size"] = json!(bytes.len());
        value["files"][1]["sha256"] = json!(format!("{:x}", Sha256::digest(bytes)));
        let manifest = Manifest::parse(&serde_json::to_vec(&value).unwrap()).unwrap();
        assert!(
            manifest
                .validate_bytes("a.txt", bytes)
                .unwrap()
                .diagnostics()
                .iter()
                .any(|d| matches!(d, pvz_assets::Diagnostic::InvalidText { .. }))
        );
    }
}

#[test]
fn 数值与_xml语法边界被拒绝() {
    for source in [
        "<fps>NaN</fps><track><name>a</name><t/></track>",
        "<fps>0</fps><track><name>a</name><t/></track>",
        "<fps>12</fps><track><name>a</name><t><x>inf</x></t></track>",
        "<fps>12</fps><track><name>a</name><t><x>1</x><x>2</x></t></track>",
        "<fps>12</fps><track><name>a</name><t/><t>内容</t></track>",
        "<fps>12</fps><track><name>a</name><t flag='1'/></track>",
        "<!DOCTYPE root><root><fps>12</fps><track><name>a</name><t/></track></root>",
        "<fps>12</fps><track><name>a</name><t><text>&unknown;</text></t></track>",
        "<fps>12</fps><track><name>a</name><t><x>1</y></t></track>",
    ] {
        assert!(parse_reanim(source).is_err(), "应拒绝：{source}");
    }
    for source in [
        "<Emitter><Field><FieldType>Friction</FieldType><X>0</X><X>1</X></Field></Emitter>",
        "<Emitter><Field><X>0</X></Field></Emitter>",
        "<Emitter><Name><Image>错误上下文</Image></Name></Emitter>",
        "<Emitter><Name>a</Name></Emitter>多余文本",
        "<root><Emitter><Name>a</Name></Emitter></root>",
    ] {
        assert!(parse_particle(source).is_err(), "应拒绝：{source}");
    }
}

#[test]
fn 大写指纹必须显式拒绝() {
    let mut value = 清单();
    value["files"][1]["sha256"] =
        json!(value["files"][1]["sha256"].as_str().unwrap().to_uppercase());
    assert!(Manifest::parse(&serde_json::to_vec(&value).unwrap()).is_err());
}

#[test]
fn 清单路径错误的调试诊断同样脱敏() {
    for pointer in ["/root", "/files/1/path"] {
        let mut value = 清单();
        *value.pointer_mut(pointer).unwrap() = json!("C:/private/input.txt");
        let error = Manifest::parse(&serde_json::to_vec(&value).unwrap()).unwrap_err();
        assert!(!format!("{error:?}").contains("C:/private"));
    }
    let mut value = 清单();
    value["baseline_id"] = json!("C:/private");
    assert!(Manifest::parse(&serde_json::to_vec(&value).unwrap()).is_err());
}

#[test]
#[cfg(any(unix, windows))]
fn 清单目录链接也不能跨越宿主边界() {
    let root = tempfile::tempdir().unwrap();
    let external = tempfile::tempdir().unwrap();
    fs::write(
        external.path().join("manifest.json"),
        serde_json::to_vec(&清单()).unwrap(),
    )
    .unwrap();
    目录链接(external.path(), &root.path().join("assets"));
    assert!(matches!(
        AssetRoot::open(root.path()),
        Err(AssetError::Io {
            kind: std::io::ErrorKind::InvalidInput,
            ..
        })
    ));
}

#[test]
fn 非法_xml字符和注释不能藏在文本中() {
    for value in ["\0", "&#0;", "\u{ffff}", "<!--a--b-->", "]]>"] {
        let source =
            format!("<fps>12</fps><track><name>a</name><t><text>{value}</text></t></track>");
        assert!(parse_reanim(&source).is_err(), "非法 XML 片段没有被拒绝");
    }
}

#[test]
fn 深层_xml在构树阶段拒绝而不会递归溢出() {
    let source = format!("{}{}", "<root>".repeat(5_000), "</root>".repeat(5_000));
    for error in [
        parse_reanim(&source).unwrap_err(),
        parse_particle(&source).unwrap_err(),
    ] {
        assert!(matches!(error,AssetError::Xml {message} if message.contains("深度")));
    }
}

#[test]
fn 原创文本截断和确定性字符突变不会触发_panic() {
    for source in [
        include_str!("../../../tests/fixtures/assets-minimal/sparse-frames.reanim"),
        include_str!("../../../tests/fixtures/assets-minimal/multiple-emitters.xml"),
    ] {
        for (position, character) in source.char_indices() {
            let _ = parse_reanim(&source[..position]);
            let _ = parse_particle(&source[..position]);
            for replacement in ["<", ">", "\0", "&"] {
                let mut mutant = source.to_owned();
                mutant.replace_range(position..position + character.len_utf8(), replacement);
                let _ = parse_reanim(&mutant);
                let _ = parse_particle(&mutant);
            }
        }
    }
}
