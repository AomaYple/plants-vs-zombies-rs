use pvz_app::geometry::{ClipRect, clipped_quad};
use pvz_app::{
    animation::{Affine, PreparedAnimation},
    audio::{AudioError, validate_audio},
    resources::SunflowerIdle,
};
use pvz_assets::AnimationFrame;

#[test]
fn 两轴独立角度缩放平移() {
    let frame = AnimationFrame {
        x: Some(10.0),
        y: Some(20.0),
        kx: Some(30.0),
        ky: Some(90.0),
        sx: Some(2.0),
        sy: Some(3.0),
        ..Default::default()
    };
    let transform = Affine::from_frame(&frame);
    let point = transform.transform([1.0, 1.0]);
    assert!((point[0] - 8.5).abs() < 0.001);
    assert!((point[1] - (22.0 + 3.0 * 30_f32.to_radians().cos())).abs() < 0.001);
}
#[test]
fn 稀疏继承空图片及marker区段保留图层顺序() {
    // 原创最小夹具：三个轨道，只验证开发合同，不宣称原版语义。
    let clip = pvz_assets::parse_reanim("<fps>12</fps><track><name>anim_idle</name><t><f>-1</f></t><t><f>0</f></t><t/><t><f>-1</f></t></track><track><name>底层</name><t><x>7</x><i>原创</i></t><t><y>8</y></t><t><i></i></t></track><track><name>上层</name><t/></track>").unwrap();
    let prepared = PreparedAnimation::new(&clip);
    assert_eq!(prepared.marker_segment("anim_idle"), Some(1..3));
    assert_eq!(prepared.names, ["anim_idle", "底层", "上层"]);
    assert_eq!(prepared.tracks[1][1].x, Some(7.0));
    assert_eq!(prepared.tracks[1][1].i.as_deref(), Some("原创"));
    assert_eq!(prepared.tracks[1][2].i.as_deref(), Some(""));
}
#[test]
fn mo3拒绝而非交给异步后端() {
    assert_eq!(
        validate_audio(b"MO3\0original-fixture"),
        Err(AudioError::UnsupportedMo3)
    );
    assert_eq!(validate_audio(b"OggSbroken"), Err(AudioError::InvalidOgg));
    assert_eq!(validate_audio(b""), Err(AudioError::InvalidOgg));
}

#[test]
fn 裁剪保持uv并拒绝无效边界与完全越界() {
    let frame = AnimationFrame {
        x: Some(-10.0),
        y: Some(5.0),
        ..Default::default()
    };
    let transform = Affine::from_frame(&frame);
    let clip = ClipRect {
        min: [0.0, 0.0],
        max: [20.0, 20.0],
    };
    let vertices = clipped_quad(transform, [20.0, 10.0], clip);
    assert_eq!(vertices.len(), 4);
    assert!(vertices.iter().all(|v| v.point[0] >= 0.0
        && v.point[0] <= 10.0
        && v.point[1] >= 5.0
        && v.point[1] <= 15.0));
    let edge = vertices.iter().find(|v| v.point == [0.0, 5.0]).unwrap();
    assert_eq!(edge.uv, [0.5, 0.0]);
    assert!(
        clipped_quad(
            transform,
            [20.0, 10.0],
            ClipRect {
                min: [30.0, 30.0],
                max: [40.0, 40.0]
            }
        )
        .is_empty()
    );
    assert!(
        clipped_quad(
            transform,
            [20.0, 10.0],
            ClipRect {
                min: [0.0, 0.0],
                max: [f32::NAN, 20.0]
            }
        )
        .is_empty()
    );
}
#[test]
fn 仿射错切越过四边仍在裁剪范围() {
    let frame = AnimationFrame {
        x: Some(-15.0),
        y: Some(-10.0),
        kx: Some(25.0),
        ky: Some(40.0),
        sx: Some(3.0),
        sy: Some(2.0),
        ..Default::default()
    };
    let vertices = clipped_quad(
        Affine::from_frame(&frame),
        [100.0, 90.0],
        ClipRect {
            min: [0.0, 0.0],
            max: [80.0, 60.0],
        },
    );
    assert!(vertices.len() >= 3);
    assert!(vertices.iter().all(|v| v.point[0] >= -0.001
        && v.point[0] <= 80.001
        && v.point[1] >= -0.001
        && v.point[1] <= 60.001
        && v.uv.iter().all(|n| *n >= -0.001 && *n <= 1.001)));
}
#[test]
fn 动画使用资源时基循环不依赖模拟tick() {
    let clip = pvz_assets::parse_reanim(
        "<fps>12</fps><track><name>anim_idle</name><t><f>0</f></t><t/><t/><t><f>-1</f></t></track>",
    )
    .unwrap();
    let animation = PreparedAnimation::new(&clip);
    let range = animation.marker_segment("anim_idle").unwrap();
    assert_eq!(animation.frame_at(1.0 / 12.0, &range), 1);
    assert_eq!(animation.frame_at(3.0 / 12.0, &range), 0);
    assert_eq!(animation.frame_at(f64::NAN, &range), 0);
    assert_eq!(animation.marker_segment("不存在"), None);
}

// 原创夹具；白名单名称只验证资源选择接口，不复制原版变换。
fn original_walk() -> PreparedAnimation {
    let mut xml = String::from(
        "<fps>12</fps><track><name>anim_walk</name><t><f>-1</f></t><t><f>0</f></t><t/><t/><t><f>-1</f></t></track><track><name>原创附件</name><t><f>0</f><i>原创附件图</i></t><t/><t/><t/></track>",
    );
    for (n, name) in pvz_app::resources::ZOMBIE_TRACKS.iter().enumerate() {
        if n == 1 {
            xml.push_str("<track><name>原创非身体</name><t><f>0</f><i>原创附件图</i></t><t/><t/><t/></track>");
        }
        xml.push_str(&format!("<track><name>{name}</name>"));
        xml.push_str(match n {
            0 => "<t><x>7</x><a>0.5</a><i>原创红图</i><f>0</f></t><t/><t><x>9</x><a>2</a><i>原创蓝图</i></t><t><i/></t>",
            1 => "<t><f>-1</f><a>-1</a><i>原创绿图</i></t><t/><t><f>0</f></t><t><f>-1</f></t>",
            18 => "<t><x>20</x><y>4</y><sx>2</sx><sy>3</sy><i>原创绿图</i><f>0</f></t><t/>",
            _ => "<t><f>-1</f><i>原创绿图</i></t><t/><t/><t/>",
        });
        xml.push_str("</track>");
    }
    PreparedAnimation::new(&pvz_assets::parse_reanim(&xml).unwrap())
}

#[test]
fn 普通僵尸walk借用源帧过滤附件保留源索引() {
    let walk = pvz_app::resources::ZombieWalk::new(original_walk()).unwrap();
    let snapshot = |seconds| {
        walk.sample(seconds)
            .map(|(layer, frame)| (layer, frame.i.as_deref().unwrap(), frame.x))
            .collect::<Vec<_>>()
    };
    assert_eq!(
        snapshot(0.0),
        [(2, "原创红图", Some(7.0)), (21, "原创绿图", Some(20.0))]
    );
    assert_eq!(
        snapshot(1.0 / 12.0),
        [(2, "原创蓝图", Some(9.0)), (4, "原创绿图", None)]
    );
    assert!(snapshot(2.0 / 12.0).is_empty());
    assert_eq!(snapshot(3.0 / 12.0), snapshot(0.0));
    assert!(std::ptr::eq(
        walk.sample(0.0).next().unwrap().1,
        walk.sample(0.0).next().unwrap().1
    ));
}

#[test]
fn 普通僵尸walk时间边界循环与异常时间() {
    let mut animation = original_walk();
    for (layer, track) in animation.tracks.iter_mut().enumerate() {
        if layer == 2 {
            *track = (0..5)
                .map(|n| AnimationFrame {
                    x: Some(n as f32),
                    i: Some("原创图".to_owned()),
                    f: Some(0.0),
                    ..Default::default()
                })
                .collect();
        } else if layer != 0 {
            for frame in track {
                frame.f = Some(-1.0);
            }
        }
    }
    let walk = pvz_app::resources::ZombieWalk::new(animation).unwrap();
    for (seconds, x) in [
        (0.0, 1.0),
        (1.0 / 12.0 - 1e-9, 1.0),
        (1.0 / 12.0, 2.0),
        (1.0 / 12.0 + 1e-9, 2.0),
        (2.0 / 12.0, 3.0),
        (3.0 / 12.0 - 1e-9, 3.0),
        (3.0 / 12.0, 1.0),
        (3.0 / 12.0 + 1e-9, 1.0),
        (6.0 / 12.0, 1.0),
        (-1.0, 1.0),
        (f64::NAN, 1.0),
        (f64::INFINITY, 1.0),
        (f64::NEG_INFINITY, 1.0),
    ] {
        let frames: Vec<_> = walk.sample(seconds).collect();
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].0, 2);
        assert_eq!(frames[0].1.x, Some(x), "采样时间 {seconds:?}");
    }
    let frames: Vec<_> = walk.sample(f64::MAX).collect();
    assert_eq!(frames.len(), 1);
    assert!([Some(1.0), Some(2.0), Some(3.0)].contains(&frames[0].1.x));
}

#[test]
fn 普通僵尸walk拒绝无区段缺身体和不可绘缓存() {
    let mut missing_marker = original_walk();
    missing_marker.names[0] = "其他".to_owned();
    let mut negative_marker = original_walk();
    for frame in &mut negative_marker.tracks[0] {
        frame.f = Some(-1.0);
    }
    let mut empty_marker = original_walk();
    empty_marker.tracks[0].clear();
    let mut missing_body = original_walk();
    missing_body.names.remove(2);
    missing_body.tracks.remove(2);
    let mut hidden = original_walk();
    let mut empty_images = original_walk();
    for layer in 0..hidden.names.len() {
        if pvz_app::resources::ZOMBIE_TRACKS.contains(&hidden.names[layer].as_str()) {
            for frame in &mut hidden.tracks[layer] {
                frame.f = Some(-1.0);
            }
            for frame in &mut empty_images.tracks[layer] {
                frame.i = Some(String::new());
            }
        }
    }
    for (animation, reason) in [
        (missing_marker, "没有普通僵尸 walk marker 区段"),
        (negative_marker, "没有普通僵尸 walk marker 区段"),
        (empty_marker, "没有普通僵尸 walk marker 区段"),
        (missing_body, "普通僵尸 walk 指定身体轨道不存在"),
        (hidden, "普通僵尸 walk 没有可绘制身体轨道"),
        (empty_images, "普通僵尸 walk 没有可绘制身体轨道"),
    ] {
        let failure = match pvz_app::resources::ZombieWalk::new(animation) {
            Ok(_) => panic!("无效 walk 应拒绝：{reason}"),
            Err(failure) => failure,
        };
        assert_eq!(failure.path, "reanim/Zombie.reanim");
        assert_eq!(failure.reason, reason);
    }
}

#[test]
fn 绘制轨道排除纯marker和全隐藏图片保留原层序() {
    let clip = pvz_assets::parse_reanim("<fps>12</fps><track><name>anim_idle</name><t><f>0</f></t><t/></track><track><name>全隐藏</name><t><f>-1</f><i>原创图片</i></t><t/></track><track><name>底层</name><t><f>0</f><i>原创底层</i></t><t/></track><track><name>上层</name><t><f>-1</f><i>原创上层</i></t><t><f>0</f></t></track><track><name>清除图片</name><t><i></i></t><t/></track>").unwrap();
    let animation = PreparedAnimation::new(&clip);
    assert_eq!(animation.drawable_tracks(&(0..2)), [2, 3]);
    assert_eq!(animation.drawable_tracks(&(0..1)), [2]);
    assert!(animation.drawable_tracks(&(2..3)).is_empty());
}

#[test]
fn 向日葵idle采样继承清空隐藏和短轨道并保留源层序() {
    // 原创夹具，仅验证本项目开发采样合同。
    let source = "<fps>12</fps><track><name>anim_idle</name><t><f>-1</f></t><t><f>0</f></t><t/><t/><t><f>-1</f></t></track><track><name>底层</name><t><x>7</x><i>原创底图</i></t><t/><t><x>9</x></t><t><i></i></t></track><track><name>上层</name><t><f>-1</f><i>原创上图</i></t><t/><t><f>0</f></t><t><f>-1</f></t></track><track><name>短轨道</name><t><i>原创短图</i></t><t/></track>";
    let idle = SunflowerIdle::new(PreparedAnimation::new(
        &pvz_assets::parse_reanim(source).unwrap(),
    ))
    .unwrap();
    let snapshot = |seconds| {
        idle.sample(seconds)
            .map(|(layer, frame)| (layer, frame.i.as_deref().unwrap(), frame.x))
            .collect::<Vec<_>>()
    };
    assert_eq!(
        snapshot(0.0),
        [(1, "原创底图", Some(7.0)), (3, "原创短图", None)]
    );
    assert_eq!(
        snapshot(1.0 / 12.0),
        [(1, "原创底图", Some(9.0)), (2, "原创上图", None)]
    );
    assert!(snapshot(2.0 / 12.0).is_empty());
    assert_eq!(snapshot(3.0 / 12.0), snapshot(0.0));
}

#[test]
fn 向日葵idle使用资源时基处理帧边界循环与异常时间() {
    // 原创逐帧位置夹具，让每个采样帧都能被精确辨别。
    let source = "<fps>12</fps><track><name>anim_idle</name><t><f>-1</f></t><t><f>0</f></t><t/><t/><t><f>-1</f></t></track><track><name>图像</name><t><x>0</x><i>原创图像</i></t><t><x>1</x></t><t><x>2</x></t><t><x>3</x></t><t><x>4</x></t></track>";
    let idle = SunflowerIdle::new(PreparedAnimation::new(
        &pvz_assets::parse_reanim(source).unwrap(),
    ))
    .unwrap();
    let boundary = 1.0 / 12.0;
    for (seconds, expected_x) in [
        (0.0, 1.0),
        (boundary - 1e-9, 1.0),
        (boundary, 2.0),
        (boundary + 1e-9, 2.0),
        (2.0 / 12.0, 3.0),
        (3.0 / 12.0 - 1e-9, 3.0),
        (3.0 / 12.0, 1.0),
        (3.0 / 12.0 + 1e-9, 1.0),
        (6.0 / 12.0, 1.0),
        (-1.0, 1.0),
        (f64::NAN, 1.0),
        (f64::INFINITY, 1.0),
        (f64::NEG_INFINITY, 1.0),
    ] {
        let actual: Vec<_> = idle
            .sample(seconds)
            .map(|(layer, frame)| (layer, frame.i.as_deref().unwrap(), frame.x))
            .collect();
        assert_eq!(
            actual,
            [(1, "原创图像", Some(expected_x))],
            "采样时间 {seconds:?}"
        );
    }
    let enormous: Vec<_> = idle.sample(f64::MAX).collect();
    assert_eq!(enormous.len(), 1);
    assert_eq!(enormous[0].0, 1);
    assert_eq!(enormous[0].1.i.as_deref(), Some("原创图像"));
    assert!([Some(1.0), Some(2.0), Some(3.0)].contains(&enormous[0].1.x));
}

#[test]
fn 向日葵idle构造拒绝缺失区段和不可绘图片并保留资源上下文() {
    // 原创失败夹具，不依赖原版程序或资源文件。
    let cases = [
        (
            "<fps>12</fps><track><name>其他</name><t><i>原创图像</i></t></track>",
            "没有向日葵 idle marker 区段",
        ),
        (
            "<fps>12</fps><track><name>anim_idle</name><t><f>-1</f></t><t/></track>",
            "没有向日葵 idle marker 区段",
        ),
        (
            "<fps>12</fps><track><name>anim_idle</name><t><f>0</f></t></track>",
            "向日葵 idle 没有可绘制轨道",
        ),
        (
            "<fps>12</fps><track><name>anim_idle</name><t><f>0</f></t></track><track><name>隐藏</name><t><f>-1</f><i>原创图像</i></t></track>",
            "向日葵 idle 没有可绘制轨道",
        ),
        (
            "<fps>12</fps><track><name>anim_idle</name><t><f>0</f></t></track><track><name>空图</name><t><i></i></t></track>",
            "向日葵 idle 没有可绘制轨道",
        ),
    ];
    for (source, expected_reason) in cases {
        let animation = PreparedAnimation::new(&pvz_assets::parse_reanim(source).unwrap());
        let error = match SunflowerIdle::new(animation) {
            Ok(_) => panic!("无效向日葵 idle 应被拒绝：{expected_reason}"),
            Err(error) => error,
        };
        assert_eq!(error.path, "reanim/SunFlower.reanim");
        assert_eq!(error.reason, expected_reason);
    }

    // 解析器先拒绝无时间点轨道；直接构造缓存输入验证该低层边界。
    let animation = PreparedAnimation {
        tracks: vec![vec![]],
        names: vec!["anim_idle".to_owned()],
        fps: 12.0,
    };
    let error = match SunflowerIdle::new(animation) {
        Ok(_) => panic!("无帧 marker 应被拒绝"),
        Err(error) => error,
    };
    assert_eq!(error.path, "reanim/SunFlower.reanim");
    assert_eq!(error.reason, "没有向日葵 idle marker 区段");
}

#[test]
fn 动画空段与反向区段只返回起点() {
    let animation = PreparedAnimation {
        tracks: vec![],
        names: vec![],
        fps: 12.0,
    };
    let start = 5;
    let end = 4;
    for segment in [start..start, start..end] {
        for seconds in [0.0, 1.0, -1.0, f64::NAN, f64::INFINITY, f64::MAX] {
            assert_eq!(animation.frame_at(seconds, &segment), start);
        }
    }
}
