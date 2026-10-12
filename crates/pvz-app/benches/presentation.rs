use criterion::{Criterion, criterion_group, criterion_main};
use pvz_app::viewport::{LogicalPoint, Viewport};
use pvz_app::{
    animation::{Affine, PreparedAnimation},
    geometry::{ClipRect, clipped_quad},
};
use pvz_assets::AnimationFrame;
use std::hint::black_box;

fn presentation(c: &mut Criterion) {
    // 原创 schema 1 / 规则 2.0 内容；计时包含 JSON 解析、校验和装配，不含文件 I/O。
    let day_json = include_str!("../../../content/levels/development-day.json");
    c.bench_function(
        &format!(
            "内容/schema1规则2.0开发白天JSON_{}字节_解析校验与Runtime装配",
            day_json.len()
        ),
        |b| {
            b.iter(|| {
                pvz_app::runtime::Runtime::from_day_json(black_box(day_json))
                    .expect("原创基准内容有效")
            })
        },
    );
    c.bench_function(
        "表现/800x600逻辑视口合同-3840x2160输入1点",
        |b| {
            let viewport = Viewport::fit((3840.0, 2160.0), (800.0, 600.0));
            b.iter(|| {
                viewport.to_logical(black_box(LogicalPoint {
                    x: 1920.0,
                    y: 1080.0,
                }))
            })
        },
    );
    let frame = AnimationFrame {
        x: Some(740.0),
        y: Some(455.0),
        kx: Some(35.0),
        ky: Some(-10.0),
        ..Default::default()
    };
    c.bench_function(
        "表现/双轴角度合同-1四边形100x90-800x600裁剪",
        |b| {
            b.iter(|| {
                clipped_quad(
                    Affine::from_frame(black_box(&frame)),
                    [100.0, 90.0],
                    ClipRect {
                        min: [0.0, 0.0],
                        max: [800.0, 600.0],
                    },
                )
            })
        },
    );
    // 原创基准夹具：8 轨道、每轨道 128 帧，含 marker 与稀疏图片轨道。
    // 构造和解析不计入索引基准，不读取或嵌入原版资源。
    let mut source = String::from("<fps>12</fps><track><name>anim_benchmark</name>");
    source.push_str("<t><f>-1</f></t><t><f>0</f></t>");
    for _ in 2..127 {
        source.push_str("<t/>");
    }
    source.push_str("<t><f>-1</f></t></track>");
    for track in 0..7 {
        source.push_str(&format!(
            "<track><name>原创图层{track}</name><t><x>{track}</x><i>原创图片{track}</i><f>0</f></t>"
        ));
        for frame in 1..128 {
            if frame % 16 == 0 {
                source.push_str(&format!("<t><x>{frame}</x><ky>15</ky></t>"));
            } else {
                source.push_str("<t/>");
            }
        }
        source.push_str("</track>");
    }
    let clip = pvz_assets::parse_reanim(&source).expect("原创基准夹具语法有效");
    let animation = PreparedAnimation::new(&clip);
    let segment = animation
        .marker_segment("anim_benchmark")
        .expect("开发marker存在");
    c.bench_function(
        "表现/原创8轨道128帧_12fps开发动画时基索引",
        |b| b.iter(|| animation.frame_at(black_box(123.25), &segment)),
    );
}
criterion_group!(benches, presentation);
criterion_main!(benches);
