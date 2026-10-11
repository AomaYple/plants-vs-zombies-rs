use bevy::{
    asset::RenderAssetUsages,
    camera::ScalingMode,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    sprite::Anchor,
};
use pvz_app::{
    animation::Affine,
    geometry::{ClipRect, SpriteVertex, clipped_quad},
    resources::{CpuImage, LoadedResources, VisualKind},
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
    if probe {
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
            "阳光"
        },
        [18.0, 12.0],
        22.0,
        Color::WHITE,
    );
    if !probe {
        for (text, x) in [
            ("向日葵 50", 140.0),
            ("豌豆射手 100", 295.0),
            ("铲除", 465.0),
            ("暂停/恢复", 557.0),
            ("重开", 705.0),
        ] {
            label(
                world,
                font,
                text,
                [x, 14.0],
                18.0,
                Color::srgb(1.0, 0.9, 0.45),
            );
        }
    }
    let status = label(world, font, "等待首帧", [18.0, 46.0], 15.0, Color::WHITE);
    let error = label(
        world,
        font,
        "",
        [18.0, 200.0],
        16.0,
        Color::srgb(1.0, 0.2, 0.2),
    );
    if probe {
        label(
            world,
            font,
            "右侧原创夹具：双轴仿射、红蓝层序、裁剪",
            [410.0, 568.0],
            14.0,
            Color::WHITE,
        );
    }
    (status, error)
}
fn logical_clip() -> ClipRect {
    ClipRect {
        min: [0.0, 0.0],
        max: [800.0, 600.0],
    }
}
struct TrackMesh {
    entity: Entity,
    mesh: Handle<Mesh>,
    material: Handle<ColorMaterial>,
}
pub struct Scene {
    poses: BTreeMap<VisualKind, Vec<pvz_assets::AnimationFrame>>,
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
        let images = resources
            .images
            .into_iter()
            .map(|(id, image)| {
                let size = [f32::from(image.width), f32::from(image.height)];
                (id, (texture(world, image), size))
            })
            .collect();
        Self {
            poses: resources.poses,
            images,
            tracks: Vec::new(),
        }
    }
    pub fn update(&mut self, world: &mut World, game: &pvz_core::GameState) {
        use pvz_core::{BoardGeometry, MowerStatus, PlantKind, UNITS_PER_PIXEL};
        let Some(board) = game.board() else {
            return;
        };
        let pixel = |value: i32| value as f32 / UNITS_PER_PIXEL as f32;
        let mut actors = Vec::new();
        for plant in board.plants() {
            let center = plant.cell.center();
            actors.push((
                match plant.kind {
                    PlantKind::Sunflower => VisualKind::Sunflower,
                    PlantKind::Peashooter => VisualKind::Peashooter,
                },
                [pixel(center.x) - 40.0, pixel(center.y) - 48.0],
                1.0 + f32::from(plant.cell.row) * 10.0,
            ));
        }
        for zombie in board.zombies() {
            actors.push((
                VisualKind::Zombie,
                [
                    pixel(zombie.x) - 40.0,
                    pixel(BoardGeometry::row_y(zombie.row)) - 90.0,
                ],
                2.0 + f32::from(zombie.row) * 10.0,
            ));
        }
        for mower in board
            .mowers()
            .iter()
            .filter(|m| m.status != MowerStatus::Spent)
        {
            actors.push((
                VisualKind::LawnMower,
                [
                    pixel(mower.x) - 40.0,
                    pixel(BoardGeometry::row_y(mower.row)) - 20.0,
                ],
                3.0 + f32::from(mower.row) * 10.0,
            ));
        }
        for pea in board.peas() {
            actors.push((
                VisualKind::Pea,
                [pixel(pea.x), pixel(BoardGeometry::row_y(pea.row)) - 20.0],
                55.0,
            ));
        }
        for sun in board.suns() {
            actors.push((
                VisualKind::Sun,
                [pixel(sun.position.x), pixel(sun.position.y)],
                60.0,
            ));
        }
        actors.sort_by(|a, b| a.2.total_cmp(&b.2));
        let mut used = 0;
        for (kind, origin, depth) in actors {
            for (layer, frame) in self.poses[&kind].iter().enumerate() {
                let id = frame.i.as_ref().expect("加载构图已经检查图像");
                let (texture, size) = &self.images[id];
                let mut transform = Affine::from_frame(frame);
                transform.tx += origin[0];
                transform.ty += origin[1];
                let vertices = clipped_quad(transform, *size, logical_clip());
                if vertices.len() < 3 {
                    continue;
                }
                let color = Color::WHITE.with_alpha(frame.a.unwrap_or(1.0).clamp(0.0, 1.0));
                let depth = depth + layer as f32 * 0.01;
                if used == self.tracks.len() {
                    let (entity, mesh, material) =
                        polygon(world, &vertices, color, Some(texture.clone()), depth);
                    self.tracks.push(TrackMesh {
                        entity,
                        mesh,
                        material,
                    });
                } else {
                    let output = &self.tracks[used];
                    *world
                        .resource_mut::<Assets<Mesh>>()
                        .get_mut(&output.mesh)
                        .expect("构图网格存在") = mesh(&vertices);
                    {
                        let mut materials = world.resource_mut::<Assets<ColorMaterial>>();
                        let mut material =
                            materials.get_mut(&output.material).expect("构图材质存在");
                        material.texture = Some(texture.clone());
                        material.color = color;
                    }
                    world
                        .get_mut::<Transform>(output.entity)
                        .expect("构图变换存在")
                        .translation
                        .z = depth;
                    *world
                        .get_mut::<Visibility>(output.entity)
                        .expect("构图可见性存在") = Visibility::Visible;
                }
                used += 1;
            }
        }
        for output in &self.tracks[used..] {
            *world
                .get_mut::<Visibility>(output.entity)
                .expect("构图可见性存在") = Visibility::Hidden;
        }
    }
}
