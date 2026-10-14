use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use pvz_app::viewport::{LogicalPoint, Viewport};
use pvz_app::{
    animation::{Affine, PreparedAnimation},
    geometry::{ClipRect, clipped_quad},
    resources::{SunflowerIdle, ZOMBIE_TRACKS, ZombieWalk},
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

    // 原创代表性工作量：29 轨道、12fps、所选区段 4..29；不复制原版轨道。
    // marker 和 27 条主体轨道各 30 帧，末条仅 6 帧；解析、预装配与核对在计时外。
    let mut source = String::from("<fps>12</fps><track><name>anim_idle</name>");
    for frame in 0..30 {
        source.push_str(match frame {
            0 | 29 => "<t><f>-1</f></t>",
            4 => "<t><f>0</f></t>",
            _ => "<t/>",
        });
    }
    source.push_str("</track>");
    for track in 1..28 {
        source.push_str(&format!("<track><name>原创idle图层{track}</name>"));
        for frame in 0..30 {
            if frame == 0 {
                let image = if track == 27 { "" } else { "原创idle图片" };
                let visible = if track == 26 { -1 } else { 0 };
                source.push_str(&format!(
                    "<t><x>{track}</x><y>{track}</y><kx>35</kx><ky>-10</ky><i>{image}</i><f>{visible}</f></t>"
                ));
            } else if frame == 19 && track == 24 {
                source.push_str("<t><i/></t>");
            } else if frame == 19 && track == 25 {
                source.push_str("<t><f>-1</f></t>");
            } else if frame == 20 && track == 24 {
                source.push_str("<t><i>原创idle图片</i></t>");
            } else if frame == 20 && track == 25 {
                source.push_str("<t><f>0</f></t>");
            } else {
                source.push_str(&format!("<t><x>{frame}</x></t>"));
            }
        }
        source.push_str("</track>");
    }
    source.push_str("<track><name>原创idle真实短轨道</name>");
    for frame in 0..6 {
        source.push_str(if frame == 4 {
            "<t><x>8</x><y>8</y><i>原创idle短图</i><f>0</f></t>"
        } else {
            "<t/>"
        });
    }
    source.push_str("</track>");
    let clip = pvz_assets::parse_reanim(&source).expect("原创向日葵基准夹具语法有效");
    assert_eq!(clip.fps(), 12.0);
    assert_eq!(clip.tracks().len(), 29);
    assert!(clip.tracks()[..28].iter().all(|t| t.frame_count() == 30));
    assert_eq!(clip.tracks()[28].frame_count(), 6);
    let animation = PreparedAnimation::new(&clip);
    assert_eq!(animation.marker_segment("anim_idle"), Some(4..29));
    assert_eq!(animation.tracks.iter().map(Vec::len).sum::<usize>(), 846);
    assert_eq!(
        animation.drawable_tracks(&(4..29)),
        (1..26).chain([28]).collect::<Vec<_>>()
    );
    let short_first = std::ptr::from_ref(&animation.tracks[28][4]);
    let short_second = std::ptr::from_ref(&animation.tracks[28][5]);
    assert!(animation.tracks[28].get(6).is_none());
    let idle = SunflowerIdle::new(animation).expect("原创向日葵基准所选idle有效");
    for (seconds, expected) in [(0.0, short_first), (1.0 / 12.0, short_second)] {
        let (_, frame) = idle
            .sample(seconds)
            .find(|(layer, _)| *layer == 28)
            .expect("真实短轨道的源帧4和5均可绘");
        assert_eq!(std::ptr::from_ref(frame), expected, "采样借用展开帧");
        assert_eq!(frame.i.as_deref(), Some("原创idle短图"));
    }
    for step in 2..25 {
        assert!(
            idle.sample(f64::from(step) / 12.0)
                .all(|(layer, _)| layer != 28)
        );
    }
    let sampled: Vec<_> = idle.sample(1.25).collect();
    assert_eq!(
        sampled.iter().map(|(layer, _)| *layer).collect::<Vec<_>>(),
        (1..24).collect::<Vec<_>>()
    );
    assert!(sampled.iter().all(|(_, frame)| frame.x == Some(19.0)));
    c.bench_function(
        "表现/原创29轨道25帧_12fps向日葵所选idle采样",
        |b| {
            b.iter(|| {
                let mut count = 0;
                for (layer, frame) in idle.sample(black_box(1.25)) {
                    black_box((layer, frame));
                    count += 1;
                }
                black_box(count);
            });
        },
    );

    // 固定 5×9 原点、40×50 图像、800×600 裁剪；全部植物借用同一缓存。
    // 计时包括生产采样、仿射与裁剪输出分配，不含 Bevy 网格池、纹理上传或 GPU 提交。
    let origins: Vec<[f32; 2]> = (0..5)
        .flat_map(|row| {
            (0..9).map(move |column| [60.0 + column as f32 * 80.0, 100.0 + row as f32 * 90.0])
        })
        .collect();
    assert_eq!(origins.len(), 45);
    let clip_rect = ClipRect {
        min: [0.0, 0.0],
        max: [800.0, 600.0],
    };
    let mut group = c.benchmark_group("表现/原创向日葵idle采样与多植物仿射裁剪");
    for plants in [1, 45] {
        let origins = &origins[..plants];
        let mut expected_vertices = 0;
        for origin in origins {
            for (_, frame) in idle.sample(1.25) {
                let mut affine = Affine::from_frame(frame);
                affine.tx += origin[0];
                affine.ty += origin[1];
                let vertices = clipped_quad(affine, [40.0, 50.0], clip_rect);
                assert_eq!(vertices.len(), 4, "固定输入的四个顶点均在裁剪范围内");
                assert_eq!(
                    vertices[0].point,
                    [
                        origin[0] + 19.0,
                        origin[1] + frame.y.expect("原创图层具有y坐标"),
                    ]
                );
                assert!(vertices.iter().all(|vertex| {
                    vertex.point.iter().all(|n| n.is_finite())
                        && (0.0..=800.0).contains(&vertex.point[0])
                        && (0.0..=600.0).contains(&vertex.point[1])
                        && vertex.uv.iter().all(|n| (0.0..=1.0).contains(n))
                }));
                expected_vertices += vertices.len();
            }
        }
        assert_eq!(
            expected_vertices,
            plants * 23 * 4,
            "完整消费全部植物和可绘图层"
        );
        group.bench_with_input(
            BenchmarkId::from_parameter(plants),
            &origins,
            |b, origins| {
                b.iter(|| {
                    let mut count = 0;
                    for origin in *origins {
                        for (_, frame) in idle.sample(black_box(1.25)) {
                            let mut affine = Affine::from_frame(frame);
                            affine.tx += origin[0];
                            affine.ty += origin[1];
                            count += black_box(clipped_quad(affine, [40.0, 50.0], clip_rect)).len();
                        }
                    }
                    black_box(count);
                });
            },
        );
    }
    group.finish();

    let mut source = String::from("<fps>12</fps><track><name>anim_walk</name>");
    for frame in 0..504 {
        source.push_str(if frame == 0 || frame == 91 {
            "<t><f>-1</f></t>"
        } else if frame == 44 {
            "<t><f>0</f></t>"
        } else {
            "<t/>"
        });
    }
    source.push_str("</track>");
    for (layer, name) in ZOMBIE_TRACKS.iter().enumerate() {
        source.push_str(&format!("<track><name>{name}</name>"));
        for frame in 0..504 {
            source.push_str(&format!(
                "<t><x>{}</x><y>{}</y><i>原创walk图</i><f>0</f></t>",
                frame % 7,
                layer % 5
            ));
        }
        source.push_str("</track>");
    }
    for n in 0..22 {
        source.push_str(&format!("<track><name>原创非白名单{n}</name>"));
        for frame in 0..504 {
            source.push_str(&format!(
                "<t><x>{}</x><y>{}</y><i>原创walk图</i><f>0</f></t>",
                frame % 7,
                (19 + n) % 5
            ));
        }
        source.push_str("</track>");
    }
    let prepared =
        PreparedAnimation::new(&pvz_assets::parse_reanim(&source).expect("原创僵尸基准夹具有效"));
    assert_eq!(prepared.tracks.len(), 42);
    assert!(prepared.tracks.iter().all(|track| track.len() == 504));
    assert_eq!(prepared.marker_segment("anim_walk"), Some(44..91));
    assert_eq!(
        prepared.drawable_tracks(&(44..91)),
        (1..42).collect::<Vec<_>>()
    );
    assert_eq!(prepared.tracks.iter().map(Vec::len).sum::<usize>(), 21168);
    let walk = ZombieWalk::new(prepared).expect("原创僵尸 walk 有效");
    for (seconds, expected) in [(0.0, 2.0), (46.0 / 12.0, 6.0), (47.0 / 12.0, 2.0)] {
        let frames: Vec<_> = walk.sample(seconds).collect();
        assert_eq!(frames.len(), 19);
        assert!(frames.iter().all(|(_, frame)| frame.x == Some(expected)));
    }
    c.bench_function(
        "表现/原创42轨道504帧_19身体47帧_12fps普通僵尸walk采样",
        |b| {
            b.iter(|| {
                let mut count = 0;
                for item in walk.sample(black_box(1.25)) {
                    black_box(item);
                    count += 1;
                }
                black_box(count);
            });
        },
    );
    let clip_rect = ClipRect {
        min: [0.0, 0.0],
        max: [800.0, 600.0],
    };
    let sampled: Vec<_> = walk.sample(1.25).collect();
    assert_eq!(sampled.len(), 19);
    assert!(
        sampled.iter().all(|(_, frame)| frame.x == Some(3.0)),
        "1.25秒对应源帧59"
    );
    for objects in [1usize, 10] {
        let origins: Vec<_> = (0..objects)
            .map(|i| [80.0 + (i % 2) as f32 * 300.0, 100.0 + (i / 2) as f32 * 80.0])
            .collect();
        let expected = objects * 19 * 4;
        let mut output = 0;
        for origin in &origins {
            for (_, frame) in walk.sample(1.25) {
                let mut affine = Affine::from_frame(frame);
                affine.tx += origin[0];
                affine.ty += origin[1];
                output += clipped_quad(affine, [40.0, 50.0], clip_rect).len();
            }
        }
        assert_eq!(output, expected);
        c.bench_function(
            &format!("表现/原创普通僵尸walk采样与多对象仿射裁剪/{objects}"),
            |b| {
                b.iter(|| {
                    let mut count = 0;
                    for origin in &origins {
                        for (_, frame) in walk.sample(black_box(1.25)) {
                            let mut affine = Affine::from_frame(frame);
                            affine.tx += origin[0];
                            affine.ty += origin[1];
                            count += black_box(clipped_quad(affine, [40.0, 50.0], clip_rect)).len();
                        }
                    }
                    black_box(count);
                });
            },
        );
    }
}
criterion_group!(benches, presentation);
criterion_main!(benches);
