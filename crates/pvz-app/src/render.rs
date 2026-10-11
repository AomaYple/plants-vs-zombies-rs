use bevy::{
    asset::RenderAssetUsages,
    camera::ScalingMode,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    sprite::Anchor,
};
use pvz_app::{
    animation::{Affine, PreparedAnimation},
    geometry::{ClipRect, SpriteVertex, clipped_quad},
    resources::{CpuImage, DEVELOPMENT_IDLE_MARKER, LoadedResources},
    viewport::Viewport as LogicalViewport,
};
use std::collections::BTreeMap;

pub const LOGICAL_SIZE: (f32, f32) = (800.0, 600.0);
pub fn world_point(point: [f32; 2], depth: f32) -> Vec3 {
    Vec3::new(point[0] - 400.0, 300.0 - point[1], depth)
}
pub fn create_camera(world: &mut World) -> Entity {
    world
        .spawn((
            Camera2d,
            Projection::Orthographic(OrthographicProjection {
                scaling_mode: ScalingMode::Fixed {
                    width: 800.0,
                    height: 600.0,
                },
                ..OrthographicProjection::default_2d()
            }),
        ))
        .id()
}
pub fn update_viewport(world: &mut World, camera: Entity, viewport: LogicalViewport) {
    let mut camera = world.get_mut::<Camera>(camera).expect("验证相机存在");
    camera.is_active = viewport.scale > 0.0;
    if viewport.scale > 0.0 {
        camera.viewport = Some(bevy::camera::Viewport {
            physical_position: UVec2::new(
                viewport.offset.x.round() as u32,
                viewport.offset.y.round() as u32,
            ),
            physical_size: UVec2::new(
                viewport.size.0.round() as u32,
                viewport.size.1.round() as u32,
            ),
            ..Default::default()
        });
    }
}
fn texture(world: &mut World, image: CpuImage) -> Handle<Image> {
    world.resource_mut::<Assets<Image>>().add(Image::new(
        Extent3d {
            width: u32::from(image.width),
            height: u32::from(image.height),
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        image.rgba,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    ))
}
fn mesh(vertices: &[SpriteVertex]) -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vertices
            .iter()
            .map(|v| world_point(v.point, 0.0).to_array())
            .collect::<Vec<_>>(),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_UV_0,
        vertices.iter().map(|v| v.uv).collect::<Vec<_>>(),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        vec![[0.0, 0.0, 1.0]; vertices.len()],
    );
    mesh.insert_indices(Indices::U32(
        (1..vertices.len().saturating_sub(1))
            .flat_map(|i| [0, (i + 1) as u32, i as u32])
            .collect(),
    ));
    mesh
}
fn rectangle_vertices(min: [f32; 2], max: [f32; 2]) -> Vec<SpriteVertex> {
    [
        [min[0], min[1]],
        [max[0], min[1]],
        [max[0], max[1]],
        [min[0], max[1]],
    ]
    .into_iter()
    .zip([[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]])
    .map(|(point, uv)| SpriteVertex { point, uv })
    .collect()
}
fn polygon(
    world: &mut World,
    vertices: &[SpriteVertex],
    color: Color,
    texture: Option<Handle<Image>>,
    depth: f32,
) -> (Entity, Handle<Mesh>, Handle<ColorMaterial>) {
    debug_assert!(vertices.len() >= 3, "渲染资产必须有实际顶点消费者");
    let mesh = world.resource_mut::<Assets<Mesh>>().add(mesh(vertices));
    let material = world
        .resource_mut::<Assets<ColorMaterial>>()
        .add(ColorMaterial {
            color,
            texture,
            ..Default::default()
        });
    let entity = world
        .spawn((
            Mesh2d(mesh.clone()),
            MeshMaterial2d(material.clone()),
            Transform::from_xyz(0.0, 0.0, depth),
        ))
        .id();
    (entity, mesh, material)
}
pub fn label(
    world: &mut World,
    font: &Handle<Font>,
    text: &str,
    point: [f32; 2],
    size: f32,
    color: Color,
) -> Entity {
    world
        .spawn((
            Text2d::new(text),
            TextFont {
                font: font.clone().into(),
                font_size: FontSize::Px(size),
                ..Default::default()
            },
            TextColor(color),
            Anchor::TOP_LEFT,
            Transform::from_translation(world_point(point, 90.0)),
        ))
        .id()
}
pub fn diagnostics(world: &mut World, font: &Handle<Font>, probe: bool) -> (Entity, Entity) {
    polygon(
        world,
        &rectangle_vertices([0.0, 0.0], [800.0, 600.0]),
        Color::srgb(0.10, 0.22, 0.20),
        None,
        -10.0,
    );
    for x in (0..=800).step_by(80) {
        polygon(
            world,
            &rectangle_vertices([x as f32, 106.0], [x as f32 + 0.6, 600.0]),
            Color::srgba(0.4, 0.9, 0.7, 0.3),
            None,
            50.0,
        );
    }
    for y in (106..=600).step_by(100) {
        polygon(
            world,
            &rectangle_vertices([0.0, y as f32], [800.0, y as f32 + 0.6]),
            Color::srgba(0.4, 0.9, 0.7, 0.3),
            None,
            50.0,
        );
    }
    for (offset, kx, ky, color) in [
        (0.0, 35.0, -10.0, Color::srgb(0.9, 0.1, 0.1)),
        (25.0, -20.0, 25.0, Color::srgb(0.1, 0.6, 1.0)),
    ] {
        let frame = pvz_assets::AnimationFrame {
            x: Some(740.0 + offset),
            y: Some(455.0),
            kx: Some(kx),
            ky: Some(ky),
            ..Default::default()
        };
        polygon(
            world,
            &clipped_quad(Affine::from_frame(&frame), [100.0, 90.0], logical_clip()),
            color,
            None,
            60.0 + offset / 25.0,
        );
    }
    polygon(
        world,
        &rectangle_vertices([0.0, 0.0], [800.0, 102.0]),
        Color::srgb(0.03, 0.06, 0.1),
        None,
        80.0,
    );
    label(
        world,
        font,
        if probe {
            "原创占位探针 · 800×600 开发逻辑尺寸"
        } else {
            "登记资源探针 · 开发动画合同"
        },
        [18.0, 12.0],
        22.0,
        Color::WHITE,
    );
    let status = label(world, font, "等待首帧", [18.0, 46.0], 16.0, Color::WHITE);
    let error = label(
        world,
        font,
        "",
        [18.0, 200.0],
        16.0,
        Color::srgb(1.0, 0.2, 0.2),
    );
    label(
        world,
        font,
        "右侧原创夹具：双轴仿射、红蓝层序、裁剪",
        [410.0, 568.0],
        14.0,
        Color::WHITE,
    );
    (status, error)
}
fn logical_clip() -> ClipRect {
    ClipRect {
        min: [0.0, 0.0],
        max: [800.0, 600.0],
    }
}
struct TrackMesh {
    source_index: usize,
    entity: Entity,
    mesh: Handle<Mesh>,
    material: Handle<ColorMaterial>,
}
pub struct Scene {
    animation: PreparedAnimation,
    segment: std::ops::Range<usize>,
    images: BTreeMap<String, (Handle<Image>, [f32; 2])>,
    tracks: Vec<TrackMesh>,
}
impl Scene {
    pub fn upload(world: &mut World, resources: LoadedResources) -> Self {
        let width = f32::from(resources.background.width);
        let height = f32::from(resources.background.height);
        let background = texture(world, resources.background);
        let mut vertices = rectangle_vertices([0.0, 0.0], [800.0, 600.0]);
        for vertex in &mut vertices {
            vertex.uv = [
                (220.0 + vertex.uv[0] * 800.0) / width,
                vertex.uv[1] * 600.0 / height,
            ];
        }
        polygon(world, &vertices, Color::WHITE, Some(background), 0.0);
        let images: BTreeMap<String, (Handle<Image>, [f32; 2])> = resources
            .images
            .into_iter()
            .map(|(id, image)| {
                let size = [f32::from(image.width), f32::from(image.height)];
                (id, (texture(world, image), size))
            })
            .collect();
        let segment = resources
            .animation
            .marker_segment(DEVELOPMENT_IDLE_MARKER)
            .expect("加载时已验证 marker");
        let tracks = resources
            .animation
            .drawable_tracks(&segment)
            .into_iter()
            .filter_map(|source_index| {
                let initial = resources.animation.tracks[source_index]
                    .iter()
                    .skip(segment.start)
                    .take(segment.len())
                    .find_map(|frame| {
                        if frame.f.is_some_and(|n| n < 0.0) {
                            return None;
                        }
                        let (texture, size) = frame.i.as_ref().and_then(|id| images.get(id))?;
                        let vertices = sprite_vertices(frame, *size);
                        (vertices.len() >= 3).then_some((frame, texture, vertices))
                    })?;
                let (entity, mesh, material) = polygon(
                    world,
                    &initial.2,
                    Color::WHITE.with_alpha(initial.0.a.unwrap_or(1.0).clamp(0.0, 1.0)),
                    Some(initial.1.clone()),
                    1.0 + source_index as f32,
                );
                Some(TrackMesh {
                    source_index,
                    entity,
                    mesh,
                    material,
                })
            })
            .collect();
        Self {
            animation: resources.animation,
            segment,
            images,
            tracks,
        }
    }
    pub fn update(&self, world: &mut World, seconds: f64) {
        let index = self.animation.frame_at(seconds, &self.segment);
        for output in &self.tracks {
            let track = &self.animation.tracks[output.source_index];
            let frame = track.get(index);
            let image = frame
                .and_then(|f| f.i.as_ref())
                .and_then(|id| self.images.get(id));
            let visible = frame.is_some_and(|f| !f.f.is_some_and(|n| n < 0.0)) && image.is_some();
            *world
                .get_mut::<Visibility>(output.entity)
                .expect("动画网格可见性存在") = if visible {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
            if let (Some(frame), Some((texture, size))) = (frame.filter(|_| visible), image) {
                let vertices = sprite_vertices(frame, *size);
                if vertices.len() < 3 {
                    *world
                        .get_mut::<Visibility>(output.entity)
                        .expect("动画网格可见性存在") = Visibility::Hidden;
                    continue;
                }
                *world
                    .resource_mut::<Assets<Mesh>>()
                    .get_mut(&output.mesh)
                    .expect("动画网格存在") = mesh(&vertices);
                let mut materials = world.resource_mut::<Assets<ColorMaterial>>();
                let mut material = materials.get_mut(&output.material).expect("动画材质存在");
                material.texture = Some(texture.clone());
                material.color = Color::WHITE.with_alpha(frame.a.unwrap_or(1.0).clamp(0.0, 1.0));
            }
        }
    }
}
fn sprite_vertices(frame: &pvz_assets::AnimationFrame, size: [f32; 2]) -> Vec<SpriteVertex> {
    let mut transform = Affine::from_frame(frame);
    transform.tx += 365.0;
    transform.ty += 310.0;
    clipped_quad(transform, size, logical_clip())
}
