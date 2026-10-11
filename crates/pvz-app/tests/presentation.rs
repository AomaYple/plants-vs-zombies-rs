use pvz_app::geometry::{ClipRect, clipped_quad};
use pvz_app::{
    animation::{Affine, PreparedAnimation},
    audio::{AudioError, validate_audio},
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

#[test]
fn 绘制轨道排除纯marker和全隐藏图片保留原层序() {
    let clip = pvz_assets::parse_reanim("<fps>12</fps><track><name>anim_idle</name><t><f>0</f></t><t/></track><track><name>全隐藏</name><t><f>-1</f><i>原创图片</i></t><t/></track><track><name>底层</name><t><f>0</f><i>原创底层</i></t><t/></track><track><name>上层</name><t><f>-1</f><i>原创上层</i></t><t><f>0</f></t></track><track><name>清除图片</name><t><i></i></t><t/></track>").unwrap();
    let animation = PreparedAnimation::new(&clip);
    assert_eq!(animation.drawable_tracks(&(0..2)), [2, 3]);
    assert_eq!(animation.drawable_tracks(&(0..1)), [2]);
    assert!(animation.drawable_tracks(&(2..3)).is_empty());
}
