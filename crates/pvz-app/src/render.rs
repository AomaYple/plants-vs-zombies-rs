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
    resources::{CpuImage, LoadedResources, SunflowerIdle, VisualKind, ZombieWalk},
    viewport::Viewport as LogicalViewport,
};
use std::collections::BTreeMap;

pub const LOGICAL_SIZE: (f32, f32) = (800.0, 600.0);
fn world_point(point: [f32; 2], depth: f32) -> Vec3 {
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
fn label(
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

fn draw_frame(
    world: &mut World,
    tracks: &mut Vec<TrackMesh>,
    used: &mut usize,
    frame: &pvz_assets::AnimationFrame,
    image: &(Handle<Image>, [f32; 2]),
    origin: [f32; 2],
    depth: f32,
) {
    let (texture, size) = image;
    let mut transform = Affine::from_frame(frame);
    transform.tx += origin[0];
    transform.ty += origin[1];
    let vertices = clipped_quad(transform, *size, logical_clip());
    if vertices.len() < 3 {
        return;
    }
    let color = Color::WHITE.with_alpha(frame.a.unwrap_or(1.0).clamp(0.0, 1.0));
    if *used == tracks.len() {
        let (entity, mesh, material) =
            polygon(world, &vertices, color, Some(texture.clone()), depth);
        tracks.push(TrackMesh {
            entity,
            mesh,
            material,
        });
    } else {
        let output = &tracks[*used];
        *world
            .resource_mut::<Assets<Mesh>>()
            .get_mut(&output.mesh)
            .expect("构图网格存在") = mesh(&vertices);
        {
            let mut materials = world.resource_mut::<Assets<ColorMaterial>>();
            let mut material = materials.get_mut(&output.material).expect("构图材质存在");
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
    *used += 1;
}

pub struct Scene {
    poses: BTreeMap<VisualKind, Vec<pvz_assets::AnimationFrame>>,
    images: BTreeMap<String, (Handle<Image>, [f32; 2])>,
    sunflower_idle: SunflowerIdle,
    zombie_walk: ZombieWalk,
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
            sunflower_idle: resources.sunflower_idle,
            zombie_walk: resources.zombie_walk,
            tracks: Vec::new(),
        }
    }
    pub fn update(
        &mut self,
        world: &mut World,
        game: &pvz_core::GameState,
        animation_seconds: f64,
    ) {
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
        let images = &self.images;
        let tracks = &mut self.tracks;
        for (kind, origin, depth) in actors {
            let mut draw = |(layer, frame): (usize, &pvz_assets::AnimationFrame)| {
                let id = frame.i.as_ref().expect("加载构图已经检查图像");
                draw_frame(
                    world,
                    tracks,
                    &mut used,
                    frame,
                    &images[id],
                    origin,
                    depth + layer as f32 * 0.01,
                );
            };
            match kind {
                VisualKind::Sunflower => self
                    .sunflower_idle
                    .sample(animation_seconds)
                    .for_each(&mut draw),
                VisualKind::Zombie => self
                    .zombie_walk
                    .sample(animation_seconds)
                    .for_each(&mut draw),
                _ => self.poses[&kind].iter().enumerate().for_each(&mut draw),
            }
        }
        for output in &tracks[used..] {
            *world
                .get_mut::<Visibility>(output.entity)
                .expect("构图可见性存在") = Visibility::Hidden;
        }
    }
}

#[cfg(test)]
pub(crate) fn sunflower_scene_fixture(world: &mut World) -> Scene {
    use bevy::camera::visibility::VisibilityClass;
    use pvz_app::animation::PreparedAnimation;
    // 与 DefaultPlugins 中 VisibilityPlugin 的 Mesh2d 注册合同一致。
    // 仅注册 CPU World 前置，不启动渲染、窗口或插件调度。
    world.register_required_components::<Mesh2d, Visibility>();
    world.register_required_components::<Mesh2d, VisibilityClass>();
    world.init_resource::<Assets<Image>>();
    world.init_resource::<Assets<Mesh>>();
    world.init_resource::<Assets<ColorMaterial>>();
    let bottom = texture(
        world,
        CpuImage {
            width: 2,
            height: 2,
            rgba: [255, 0, 0, 255].repeat(4),
        },
    );
    let upper = texture(
        world,
        CpuImage {
            width: 2,
            height: 2,
            rgba: [0, 0, 255, 255].repeat(4),
        },
    );
    let short = texture(
        world,
        CpuImage {
            width: 2,
            height: 2,
            rgba: [0, 255, 0, 255].repeat(4),
        },
    );
    // 原创离散夹具：三帧区段，底图/上图/短图各有独立颜色与纹理。
    let source = "<fps>12</fps><track><name>anim_idle</name><t><f>-1</f></t><t><f>0</f></t><t/><t/><t><f>-1</f></t></track><track><name>底层</name><t><x>7</x><a>0.5</a><i>原创底图</i></t><t/><t><x>9</x><a>2</a></t><t><i></i></t></track><track><name>上层</name><t><f>-1</f><a>-1</a><i>原创上图</i></t><t/><t><f>0</f></t><t><f>-1</f></t></track><track><name>短轨道</name><t><x>20</x><y>4</y><sx>2</sx><sy>3</sy><i>原创短图</i></t><t/></track>";
    let idle = SunflowerIdle::new(PreparedAnimation::new(
        &pvz_assets::parse_reanim(source).expect("原创向日葵夹具解析成功"),
    ))
    .expect("原创向日葵夹具具有 idle 图层");
    let images = [
        ("原创底图".to_owned(), (bottom, [2.0, 2.0])),
        ("原创上图".to_owned(), (upper, [2.0, 2.0])),
        ("原创短图".to_owned(), (short, [2.0, 2.0])),
    ]
    .into_iter()
    .collect();
    // 割草机等静态消费者具有合法图像，置于裁剪区外以隔离动画图层断言。
    let static_frame = pvz_assets::AnimationFrame {
        x: Some(1000.0),
        i: Some("原创底图".to_owned()),
        ..Default::default()
    };
    let source = format!(
        "<fps>12</fps><track><name>anim_walk</name><t><f>0</f></t></track>{}",
        pvz_app::resources::ZOMBIE_TRACKS
            .iter()
            .enumerate()
            .map(|(n, name)| format!(
                "<track><name>{name}</name><t><x>1000</x><f>{}</f><i>原创底图</i></t></track>",
                if n == 0 { 0 } else { -1 }
            ))
            .collect::<String>()
    );
    let zombie_walk = ZombieWalk::new(PreparedAnimation::new(
        &pvz_assets::parse_reanim(&source).unwrap(),
    ))
    .unwrap();
    Scene {
        poses: [
            VisualKind::Peashooter,
            VisualKind::LawnMower,
            VisualKind::Pea,
            VisualKind::Sun,
        ]
        .into_iter()
        .map(|kind| (kind, vec![static_frame.clone()]))
        .collect(),
        images,
        sunflower_idle: idle,
        zombie_walk,
        tracks: Vec::new(),
    }
}

#[cfg(test)]
pub(crate) fn zombie_scene_fixture(world: &mut World) -> Scene {
    use pvz_app::animation::PreparedAnimation;
    let mut scene = sunflower_scene_fixture(world);
    // 独立 World 中复用 CPU 注册前置与三张原创 2×2 纹理。
    let mut xml = String::from(
        "<fps>12</fps><track><name>anim_walk</name><t><f>-1</f></t><t><f>0</f></t><t/><t/><t><f>-1</f></t></track><track><name>原创附件</name><t><f>0</f><i>原创底图</i></t><t/><t/><t/></track>",
    );
    for (n, name) in pvz_app::resources::ZOMBIE_TRACKS.iter().enumerate() {
        if n == 1 {
            xml.push_str(
                "<track><name>原创非身体</name><t><f>0</f><i>原创底图</i></t><t/><t/><t/></track>",
            );
        }
        xml.push_str(&format!("<track><name>{name}</name>"));
        xml.push_str(match n {
            0 => "<t><x>7</x><a>0.5</a><i>原创底图</i><f>0</f></t><t/><t><x>9</x><a>2</a><i>原创上图</i></t><t><i/></t>",
            1 => "<t><f>-1</f><a>-1</a><i>原创短图</i></t><t/><t><f>0</f></t><t><f>-1</f></t>",
            18 => "<t><x>20</x><y>4</y><sx>2</sx><sy>3</sy><i>原创短图</i><f>0</f></t><t/>",
            _ => "<t><f>-1</f><i>原创短图</i></t><t/><t/><t/>",
        });
        xml.push_str("</track>");
    }
    scene.zombie_walk = ZombieWalk::new(PreparedAnimation::new(
        &pvz_assets::parse_reanim(&xml).unwrap(),
    ))
    .unwrap();
    scene.sunflower_idle = SunflowerIdle::new(PreparedAnimation::new(&pvz_assets::parse_reanim("<fps>12</fps><track><name>anim_idle</name><t><f>0</f></t></track><track><name>裁剪外</name><t><x>1000</x><i>原创底图</i></t></track>").unwrap())).unwrap();
    scene
}

#[cfg(test)]
pub(crate) fn zombie_test_game() -> pvz_core::GameState {
    let mut definition: pvz_data::DayLevelDefinition = serde_json::from_str(include_str!(
        "../../../content/levels/development-failure.json"
    ))
    .unwrap();
    definition.initial_sun = 150;
    definition.rules.zombie_speed = 1;
    definition.rules.mower_trigger_x = 4000;
    definition.rules.defeat_x = -20000;
    definition.rules.zombie_health = 20;
    definition.rules.pea_damage = 20;
    definition.rules.pea_speed = 100000;
    definition.rules.shoot_interval = 1;
    definition.rules.wave = vec![
        pvz_data::DaySpawnDefinition {
            at: 1,
            row: 0,
            x: 30000,
        },
        pvz_data::DaySpawnDefinition {
            at: 1,
            row: 2,
            x: 50000,
        },
    ];
    let mut game = pvz_core::GameState::from_day_definition(&definition).unwrap();
    game.step(&[]);
    assert_eq!(game.board().unwrap().zombies().len(), 2);
    game
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::mesh::VertexAttributeValues;
    use pvz_app::animation::PreparedAnimation;
    use pvz_core::{Command, PlantKind};

    fn sunflower_game() -> pvz_core::GameState {
        let mut runtime = pvz_app::runtime::Runtime::failure_day();
        runtime.command(Command::SelectCard {
            kind: PlantKind::Sunflower,
        });
        runtime.command(Command::Plant {
            cell: pvz_core::Cell { row: 1, column: 0 },
        });
        runtime.advance(1.0 / 30.0);
        runtime.game
    }

    fn layer_material(world: &World, scene: &Scene, slot: usize) -> Handle<ColorMaterial> {
        world
            .get::<MeshMaterial2d<ColorMaterial>>(scene.tracks[slot].entity)
            .expect("向日葵图层材质组件存在")
            .0
            .clone()
    }

    fn positions(world: &World, scene: &Scene, slot: usize) -> Vec<[f32; 3]> {
        let mesh = world
            .resource::<Assets<Mesh>>()
            .get(&scene.tracks[slot].mesh)
            .expect("向日葵图层网格存在");
        match mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .expect("顶点位置存在")
        {
            VertexAttributeValues::Float32x3(values) => values.clone(),
            other => panic!("向日葵顶点格式异常: {other:?}"),
        }
    }

    fn uvs(world: &World, scene: &Scene, slot: usize) -> Vec<[f32; 2]> {
        let mesh = world
            .resource::<Assets<Mesh>>()
            .get(&scene.tracks[slot].mesh)
            .unwrap();
        match mesh.attribute(Mesh::ATTRIBUTE_UV_0).expect("纹理坐标存在") {
            VertexAttributeValues::Float32x2(values) => values.clone(),
            other => panic!("向日葵纹理坐标格式异常：{other:?}"),
        }
    }

    fn alpha(world: &World, scene: &Scene, slot: usize) -> f32 {
        world
            .resource::<Assets<ColorMaterial>>()
            .get(&scene.tracks[slot].material)
            .unwrap()
            .color
            .alpha()
    }

    fn asset_counts(world: &World) -> [usize; 3] {
        [
            world.resource::<Assets<Image>>().len(),
            world.resource::<Assets<Mesh>>().len(),
            world.resource::<Assets<ColorMaterial>>().len(),
        ]
    }

    type SceneTrackSnapshot = (Vec<[f32; 3]>, bool, Option<AssetId<Image>>, f32);

    fn scene_snapshot(world: &World, scene: &Scene) -> Vec<SceneTrackSnapshot> {
        scene
            .tracks
            .iter()
            .enumerate()
            .map(|(slot, track)| {
                let material = world
                    .resource::<Assets<ColorMaterial>>()
                    .get(&track.material)
                    .expect("构图材质存在");
                (
                    positions(world, scene, slot),
                    world
                        .get::<Visibility>(track.entity)
                        .is_some_and(|visibility| *visibility != Visibility::Hidden),
                    material.texture.as_ref().map(Handle::id),
                    material.color.alpha(),
                )
            })
            .collect()
    }

    fn replace_idle(scene: &mut Scene, source: &str) {
        scene.sunflower_idle = SunflowerIdle::new(PreparedAnimation::new(
            &pvz_assets::parse_reanim(source).unwrap(),
        ))
        .unwrap();
    }

    fn terminal_runtime(with_peashooter: bool) -> pvz_app::runtime::Runtime {
        let mut runtime = pvz_app::runtime::Runtime::failure_day();
        runtime.command(Command::SelectCard {
            kind: PlantKind::Sunflower,
        });
        runtime.command(Command::Plant {
            cell: pvz_core::Cell { row: 1, column: 0 },
        });
        if with_peashooter {
            runtime.command(Command::SelectCard {
                kind: PlantKind::Peashooter,
            });
            runtime.command(Command::Plant {
                cell: pvz_core::Cell { row: 0, column: 0 },
            });
        }
        runtime.advance(1.0 / 60.0);
        assert_eq!(runtime.game.sun(), if with_peashooter { 0 } else { 100 });
        let planted = runtime.game.board().unwrap().plants();
        assert_eq!(planted.len(), if with_peashooter { 2 } else { 1 });
        runtime
    }

    fn combined_walk_scene(world: &mut World) -> Scene {
        // 两类缓存引用相同的三张原创纹理；临时 World 仅构造纯缓存，不参与正式池统计。
        let mut fixture_world = World::new();
        let zombie = zombie_scene_fixture(&mut fixture_world);
        let mut scene = sunflower_scene_fixture(world);
        scene.zombie_walk = zombie.zombie_walk;
        scene
    }

    fn replay_snapshots(
        with_peashooter: bool,
        consume_scene: bool,
    ) -> Vec<(pvz_core::StateDigest, Vec<pvz_core::Event>, f64)> {
        let mut world = World::new();
        let mut scene = consume_scene.then(|| combined_walk_scene(&mut world));
        let mut runtime = terminal_runtime(with_peashooter);
        let update_scene =
            |runtime: &pvz_app::runtime::Runtime, scene: &mut Option<Scene>, world: &mut World| {
                if let Some(scene) = scene {
                    let digest = runtime.game.state_digest();
                    let zombies = runtime.game.board().unwrap().zombies().to_vec();
                    let plants = runtime.game.board().unwrap().plants().to_vec();
                    for extra in [0.12, 1.25, 0.0] {
                        scene.update(world, &runtime.game, runtime.animation_seconds + extra);
                    }
                    assert_eq!(runtime.game.state_digest(), digest);
                    assert_eq!(runtime.game.board().unwrap().zombies(), zombies);
                    assert_eq!(runtime.game.board().unwrap().plants(), plants);
                }
            };
        update_scene(&runtime, &mut scene, &mut world);
        let mut snapshots = vec![(
            runtime.game.state_digest(),
            runtime.take_events(),
            runtime.animation_seconds,
        )];
        assert_eq!(
            snapshots[0]
                .1
                .iter()
                .filter(|event| matches!(event, pvz_core::Event::Planted { .. }))
                .count(),
            if with_peashooter { 2 } else { 1 }
        );
        assert_eq!(
            snapshots[0]
                .1
                .iter()
                .filter_map(|event| match event {
                    pvz_core::Event::SunSpent { amount, .. } => Some(*amount),
                    _ => None,
                })
                .sum::<u32>(),
            if with_peashooter { 150 } else { 50 }
        );
        runtime.advance(0.25);
        update_scene(&runtime, &mut scene, &mut world);
        snapshots.push((
            runtime.game.state_digest(),
            runtime.take_events(),
            runtime.animation_seconds,
        ));
        runtime.command(Command::Pause);
        runtime.advance(1.0 / 60.0);
        update_scene(&runtime, &mut scene, &mut world);
        snapshots.push((
            runtime.game.state_digest(),
            runtime.take_events(),
            runtime.animation_seconds,
        ));
        runtime.advance(100.0);
        update_scene(&runtime, &mut scene, &mut world);
        snapshots.push((
            runtime.game.state_digest(),
            runtime.take_events(),
            runtime.animation_seconds,
        ));
        runtime.command(Command::Resume);
        runtime.advance(1.0 / 60.0);
        update_scene(&runtime, &mut scene, &mut world);
        snapshots.push((
            runtime.game.state_digest(),
            runtime.take_events(),
            runtime.animation_seconds,
        ));
        runtime.advance(0.25);
        update_scene(&runtime, &mut scene, &mut world);
        snapshots.push((
            runtime.game.state_digest(),
            runtime.take_events(),
            runtime.animation_seconds,
        ));
        for _ in 0..1_000 {
            if runtime
                .game
                .board()
                .is_some_and(|board| board.status() != pvz_core::GameStatus::Running)
            {
                break;
            }
            let before = runtime.game.state_digest();
            if let Some(scene) = &mut scene {
                scene.update(&mut world, &runtime.game, runtime.animation_seconds + 0.12);
            }
            assert_eq!(
                runtime.game.state_digest(),
                before,
                "额外表现采样不改变核心"
            );
            runtime.advance(0.25);
            update_scene(&runtime, &mut scene, &mut world);
            snapshots.push((
                runtime.game.state_digest(),
                runtime.take_events(),
                runtime.animation_seconds,
            ));
        }
        assert_eq!(
            runtime.game.board().unwrap().status(),
            if with_peashooter {
                pvz_core::GameStatus::Won
            } else {
                pvz_core::GameStatus::Lost
            },
            "回放必须实际达到目标终局"
        );
        runtime.advance(100.0);
        update_scene(&runtime, &mut scene, &mut world);
        snapshots.push((
            runtime.game.state_digest(),
            runtime.take_events(),
            runtime.animation_seconds,
        ));
        runtime.command(Command::Restart);
        runtime.advance(1.0 / 60.0);
        update_scene(&runtime, &mut scene, &mut world);
        snapshots.push((
            runtime.game.state_digest(),
            runtime.take_events(),
            runtime.animation_seconds,
        ));
        runtime.command(Command::SelectCard {
            kind: PlantKind::Sunflower,
        });
        runtime.command(Command::Plant {
            cell: pvz_core::Cell { row: 1, column: 0 },
        });
        runtime.advance(1.0 / 60.0);
        update_scene(&runtime, &mut scene, &mut world);
        snapshots.push((
            runtime.game.state_digest(),
            runtime.take_events(),
            runtime.animation_seconds,
        ));
        runtime.command(Command::Shovel {
            cell: pvz_core::Cell { row: 1, column: 0 },
        });
        runtime.advance(1.0 / 60.0);
        update_scene(&runtime, &mut scene, &mut world);
        snapshots.push((
            runtime.game.state_digest(),
            runtime.take_events(),
            runtime.animation_seconds,
        ));
        for _ in 0..7 {
            runtime.advance(0.25);
            update_scene(&runtime, &mut scene, &mut world);
            snapshots.push((
                runtime.game.state_digest(),
                runtime.take_events(),
                runtime.animation_seconds,
            ));
        }
        assert!(
            runtime
                .game
                .board()
                .unwrap()
                .zombies()
                .iter()
                .any(|z| z.row == 0)
        );
        snapshots
    }

    #[test]
    fn 真实_scene固定区段端点与异常时间保持确定构图() {
        let mut world = World::new();
        let mut scene = sunflower_scene_fixture(&mut world);
        // 原创 30 帧夹具；区段规模与登记 idle 相同，不复制原版轨道。
        let mut source = String::from("<fps>12</fps><track><name>anim_idle</name>");
        for index in 0..30 {
            source.push_str(if (4..29).contains(&index) {
                "<t><f>0</f></t>"
            } else {
                "<t><f>-1</f></t>"
            });
        }
        source.push_str("</track><track><name>原创位移层</name>");
        for index in 0..30 {
            source.push_str(&format!("<t><x>{index}</x><i>原创底图</i></t>"));
        }
        source.push_str("</track>");
        replace_idle(&mut scene, &source);
        let game = sunflower_game();
        for (seconds, source_frame) in [
            (0.0, 4.0),
            (1.0 / 12.0, 5.0),
            (24.0 / 12.0, 28.0),
            (25.0 / 12.0, 4.0),
        ] {
            scene.update(&mut world, &game, seconds);
            assert_eq!(
                positions(&world, &scene, 0)[0],
                [-320.0 + source_frame, 104.0, 0.0],
                "固定表现时间对应明确的源帧位置"
            );
        }
        let first = scene_snapshot(&world, &scene);
        let counts = asset_counts(&world);
        for seconds in [-1.0, f64::NAN, f64::NEG_INFINITY, f64::INFINITY] {
            scene.update(&mut world, &game, seconds);
            assert_eq!(scene_snapshot(&world, &scene), first);
            assert_eq!(asset_counts(&world), counts);
        }
        scene.update(&mut world, &game, f64::MAX);
        let position = positions(&world, &scene, 0)[0];
        assert!((-316.0..=-292.0).contains(&position[0]));
        assert_eq!(&position[1..], &[104.0, 0.0]);
        assert_eq!(asset_counts(&world), counts);
    }

    #[test]
    fn 真实_scene向日葵_idle借用网格材质并按时间切换纹理() {
        let mut world = World::new();
        let mut scene = sunflower_scene_fixture(&mut world);
        let game = sunflower_game();
        let handles: Vec<_> = ["原创底图", "原创上图", "原创短图"]
            .into_iter()
            .map(|id| scene.images[id].0.clone())
            .collect();
        assert_ne!(handles[0], handles[1]);
        assert_ne!(handles[0], handles[2]);
        assert_ne!(handles[1], handles[2]);
        scene.update(&mut world, &game, 0.0);
        assert_eq!(scene.tracks.len(), 2, "首帧底层和短轨道各占一个池槽");
        let first_entities: Vec<_> = scene.tracks.iter().map(|track| track.entity).collect();
        assert!(first_entities.iter().all(|&entity| {
            world.get::<InheritedVisibility>(entity).is_some()
                && world.get::<ViewVisibility>(entity).is_some()
                && world
                    .get::<bevy::camera::visibility::VisibilityClass>(entity)
                    .is_some()
        }));
        let first_materials: Vec<_> = (0..2)
            .map(|slot| layer_material(&world, &scene, slot))
            .collect();
        let red = world
            .resource::<Assets<ColorMaterial>>()
            .get(&first_materials[0])
            .unwrap();
        let green = world
            .resource::<Assets<ColorMaterial>>()
            .get(&first_materials[1])
            .unwrap();
        assert_eq!(red.texture.as_ref(), Some(&handles[0]));
        assert_eq!(green.texture.as_ref(), Some(&handles[2]));
        assert_eq!(red.color.alpha(), 0.5);
        assert_eq!(green.color.alpha(), 1.0);
        assert_eq!(
            world.get::<Visibility>(first_entities[0]),
            Some(&Visibility::Inherited)
        );
        assert_eq!(
            world.get::<Visibility>(first_entities[1]),
            Some(&Visibility::Inherited)
        );
        let first_positions = positions(&world, &scene, 0);
        // row 1/column 0 的逻辑中心为 (120,244)，植物原点为 (80,196)。
        assert_eq!(
            first_positions,
            [
                [-313.0, 104.0, 0.0],
                [-311.0, 104.0, 0.0],
                [-311.0, 102.0, 0.0],
                [-313.0, 102.0, 0.0]
            ]
        );
        assert_eq!(
            positions(&world, &scene, 1),
            [
                [-300.0, 100.0, 0.0],
                [-296.0, 100.0, 0.0],
                [-296.0, 94.0, 0.0],
                [-300.0, 94.0, 0.0]
            ]
        );
        assert_eq!(
            uvs(&world, &scene, 0),
            [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]
        );
        assert_eq!(
            world
                .get::<Transform>(first_entities[0])
                .unwrap()
                .translation
                .z,
            11.01
        );
        assert_eq!(
            world
                .get::<Transform>(first_entities[1])
                .unwrap()
                .translation
                .z,
            11.03
        );
        let counts = asset_counts(&world);

        scene.update(&mut world, &game, 1.0 / 12.0);
        assert_eq!(scene.tracks.len(), 2, "第二帧复用同一池容量");
        assert_eq!(scene.tracks[0].entity, first_entities[0]);
        assert_eq!(scene.tracks[1].entity, first_entities[1]);
        let upper_material = layer_material(&world, &scene, 1);
        assert_eq!(upper_material, first_materials[1]);
        assert_eq!(
            world
                .resource::<Assets<ColorMaterial>>()
                .get(&upper_material)
                .unwrap()
                .texture
                .as_ref(),
            Some(&handles[1])
        );
        assert_eq!(
            positions(&world, &scene, 0),
            [
                [-311.0, 104.0, 0.0],
                [-309.0, 104.0, 0.0],
                [-309.0, 102.0, 0.0],
                [-311.0, 102.0, 0.0]
            ]
        );
        assert_eq!(alpha(&world, &scene, 0), 1.0);
        assert_eq!(alpha(&world, &scene, 1), 0.0);
        assert_eq!(
            world
                .get::<Transform>(first_entities[1])
                .unwrap()
                .translation
                .z,
            11.02
        );
        assert_eq!(
            world.get::<Visibility>(first_entities[1]),
            Some(&Visibility::Visible)
        );

        scene.update(&mut world, &game, 2.0 / 12.0);
        assert!(
            scene.tracks.iter().all(|track| {
                world.get::<Visibility>(track.entity) == Some(&Visibility::Hidden)
            })
        );
        scene.update(&mut world, &game, 3.0 / 12.0);
        assert_eq!(positions(&world, &scene, 0), first_positions);
        assert_eq!(scene.tracks[1].entity, first_entities[1]);
        assert_eq!(layer_material(&world, &scene, 1), first_materials[1]);
        assert_eq!(alpha(&world, &scene, 0), 0.5);
        assert_eq!(
            world
                .get::<Transform>(first_entities[1])
                .unwrap()
                .translation
                .z,
            11.03
        );
        assert!(
            scene
                .tracks
                .iter()
                .all(|track| world.get::<Visibility>(track.entity) == Some(&Visibility::Visible))
        );
        assert_eq!(asset_counts(&world), counts);
        assert_eq!(
            world
                .resource::<Assets<ColorMaterial>>()
                .get(&layer_material(&world, &scene, 0))
                .unwrap()
                .texture
                .as_ref(),
            Some(&handles[0])
        );
        assert_eq!(
            world
                .resource::<Assets<ColorMaterial>>()
                .get(&layer_material(&world, &scene, 1))
                .unwrap()
                .texture
                .as_ref(),
            Some(&handles[2])
        );
    }

    #[test]
    fn 真实_scene向日葵部分裁剪插值_uv且完全越界隐藏旧池() {
        let mut world = World::new();
        let mut scene = sunflower_scene_fixture(&mut world);
        let game = sunflower_game();
        scene.update(&mut world, &game, 0.0);
        let counts = asset_counts(&world);
        replace_idle(
            &mut scene,
            "<fps>12</fps><track><name>anim_idle</name><t><f>0</f></t><t/></track><track><name>裁剪</name><t><x>-81</x><i>原创底图</i></t><t><x>-83</x></t></track>",
        );
        scene.update(&mut world, &game, 0.0);
        assert_eq!(
            positions(&world, &scene, 0),
            [
                [-400.0, 104.0, 0.0],
                [-399.0, 104.0, 0.0],
                [-399.0, 102.0, 0.0],
                [-400.0, 102.0, 0.0]
            ]
        );
        assert_eq!(
            uvs(&world, &scene, 0),
            [[0.5, 0.0], [1.0, 0.0], [1.0, 1.0], [0.5, 1.0]]
        );
        assert_eq!(
            world.get::<Visibility>(scene.tracks[1].entity),
            Some(&Visibility::Hidden)
        );
        scene.update(&mut world, &game, 1.0 / 12.0);
        assert!(
            scene
                .tracks
                .iter()
                .all(|track| world.get::<Visibility>(track.entity) == Some(&Visibility::Hidden))
        );
        assert_eq!(asset_counts(&world), counts);
    }

    #[test]
    fn 真实_scene向日葵多株移除重开复用容量与资产() {
        let mut world = World::new();
        let mut scene = sunflower_scene_fixture(&mut world);
        let mut game = sunflower_game();
        // 正常命令与模拟步消除开发卡冷却，测试不直接写棋盘实体。
        for _ in 0..game.board().unwrap().rules().card_cooldown {
            game.step(&[]);
        }
        let second = pvz_core::Cell { row: 2, column: 1 };
        game.step(&[Command::Plant { cell: second }]);
        assert_eq!(game.board().unwrap().plants().len(), 2);
        scene.update(&mut world, &game, 0.0);
        assert_eq!(scene.tracks.len(), 4);
        assert_eq!(positions(&world, &scene, 2)[0], [-233.0, 8.0, 0.0]);
        assert_eq!(
            world
                .get::<Transform>(scene.tracks[2].entity)
                .unwrap()
                .translation
                .z,
            21.01
        );
        let counts = asset_counts(&world);
        let images: Vec<_> = scene
            .images
            .values()
            .map(|(handle, _)| handle.clone())
            .collect();
        let pool: Vec<_> = scene
            .tracks
            .iter()
            .map(|track| (track.entity, track.mesh.clone(), track.material.clone()))
            .collect();
        for seconds in [1.0 / 12.0, 2.0 / 12.0, 3.0 / 12.0, 4.0 / 12.0] {
            scene.update(&mut world, &game, seconds);
            assert_eq!(asset_counts(&world), counts);
            assert!(
                images
                    .iter()
                    .all(|handle| world.resource::<Assets<Image>>().contains(handle.id()))
            );
        }
        game.step(&[Command::Shovel { cell: second }]);
        scene.update(&mut world, &game, 0.0);
        assert_eq!(scene.tracks.len(), 4);
        assert!(
            scene.tracks[2..]
                .iter()
                .all(|track| world.get::<Visibility>(track.entity) == Some(&Visibility::Hidden))
        );
        game.step(&[Command::Restart]);
        scene.update(&mut world, &game, 0.0);
        assert!(
            scene
                .tracks
                .iter()
                .all(|track| world.get::<Visibility>(track.entity) == Some(&Visibility::Hidden))
        );
        game.step(&[
            Command::SelectCard {
                kind: PlantKind::Sunflower,
            },
            Command::Plant {
                cell: pvz_core::Cell { row: 1, column: 0 },
            },
        ]);
        scene.update(&mut world, &game, 1.0 / 12.0);
        assert_eq!(scene.tracks.len(), 4);
        assert_eq!(
            scene
                .tracks
                .iter()
                .map(|track| (track.entity, track.mesh.clone(), track.material.clone()))
                .collect::<Vec<_>>(),
            pool
        );
        assert_eq!(asset_counts(&world), counts);
        assert_eq!(
            world
                .resource::<Assets<ColorMaterial>>()
                .get(&scene.tracks[1].material)
                .unwrap()
                .texture
                .as_ref(),
            Some(&scene.images["原创上图"].0)
        );
    }

    #[test]
    fn 真实_scene四十五株向日葵共享缓存与纹理池() {
        let mut world = World::new();
        let mut scene = sunflower_scene_fixture(&mut world);
        let mut value: serde_json::Value = serde_json::from_str(include_str!(
            "../../../content/levels/development-failure.json"
        ))
        .expect("失败场景内容有效");
        value["initial_sun"] = 2_250.into();
        value["rules"]["card_cooldown"] = 1.into();
        value["rules"]["wave"] = serde_json::json!([{ "at": 1_000_000, "row": 0, "x": 6_000 }]);
        let mut runtime =
            pvz_app::runtime::Runtime::from_day_json(&value.to_string()).expect("45株夹具规则有效");
        runtime.command(Command::SelectCard {
            kind: PlantKind::Sunflower,
        });
        for row in 0..5 {
            for column in 0..9 {
                runtime.command(Command::Plant {
                    cell: pvz_core::Cell { row, column },
                });
                runtime.advance(1.0 / 60.0);
            }
        }
        assert_eq!(runtime.game.board().expect("棋盘存在").plants().len(), 45);
        scene.update(&mut world, &runtime.game, 0.0);
        assert_eq!(scene.tracks.len(), 90, "每株两层但只拥有一份缓存");
        let counts = asset_counts(&world);
        let entities = scene
            .tracks
            .iter()
            .map(|track| track.entity)
            .collect::<Vec<_>>();
        let first = scene_snapshot(&world, &scene);
        scene.update(&mut world, &runtime.game, 1.0 / 12.0);
        assert_eq!(scene.tracks.len(), 90);
        assert_eq!(
            scene
                .tracks
                .iter()
                .map(|track| track.entity)
                .collect::<Vec<_>>(),
            entities,
            "45株更新复用同一网格池"
        );
        assert_eq!(
            asset_counts(&world),
            counts,
            "更新不新增纹理、网格或材质资产"
        );
        assert_ne!(
            scene_snapshot(&world, &scene),
            first,
            "表现时钟驱动所有植物换帧"
        );
    }

    #[test]
    fn 真实_scene失败与胜利终局冻结向日葵并可重开再种植() {
        for with_peashooter in [false, true] {
            let mut world = World::new();
            let mut scene = sunflower_scene_fixture(&mut world);
            let mut runtime = terminal_runtime(with_peashooter);
            for _ in 0..1_000 {
                scene.update(&mut world, &runtime.game, runtime.animation_seconds);
                if runtime
                    .game
                    .board()
                    .is_some_and(|board| board.status() != pvz_core::GameStatus::Running)
                {
                    break;
                }
                runtime.advance(0.25);
            }
            let status = runtime.game.board().expect("终局棋盘存在").status();
            assert_eq!(
                status,
                if with_peashooter {
                    pvz_core::GameStatus::Won
                } else {
                    pvz_core::GameStatus::Lost
                }
            );
            assert!(runtime.game.board().unwrap().plants().iter().any(|plant| {
                plant.kind == PlantKind::Sunflower
                    && plant.cell == pvz_core::Cell { row: 1, column: 0 }
            }));
            let terminal = (
                runtime.game.state_digest(),
                runtime.animation_seconds,
                scene_snapshot(&world, &scene),
            );
            let events = runtime.take_events();
            assert_eq!(
                events
                    .iter()
                    .filter(|event| matches!(
                        (with_peashooter, event),
                        (true, pvz_core::Event::Victory) | (false, pvz_core::Event::Defeat)
                    ))
                    .count(),
                1
            );
            runtime.advance(100.0);
            scene.update(&mut world, &runtime.game, runtime.animation_seconds);
            assert_eq!(runtime.game.state_digest(), terminal.0);
            assert_eq!(runtime.animation_seconds, terminal.1);
            assert_eq!(scene_snapshot(&world, &scene), terminal.2);
            assert!(runtime.take_events().is_empty(), "终局不重复发事件");

            runtime.command(Command::Restart);
            runtime.advance(1.0 / 60.0);
            scene.update(&mut world, &runtime.game, runtime.animation_seconds);
            assert_eq!(
                runtime.game.board().unwrap().status(),
                pvz_core::GameStatus::Running
            );
            assert_eq!(runtime.game.tick(), 0);
            assert_eq!(runtime.animation_seconds, 0.0);
            assert!(scene
                .tracks
                .iter()
                .all(|track| world.get::<Visibility>(track.entity) == Some(&Visibility::Hidden)));
            runtime.command(Command::SelectCard {
                kind: PlantKind::Sunflower,
            });
            runtime.command(Command::Plant {
                cell: pvz_core::Cell { row: 1, column: 0 },
            });
            runtime.advance(1.0 / 60.0);
            scene.update(&mut world, &runtime.game, runtime.animation_seconds);
            assert!(
                scene.tracks.iter().any(
                    |track| world.get::<Visibility>(track.entity) == Some(&Visibility::Visible)
                )
            );
            assert_eq!(positions(&world, &scene, 0)[0], [-313.0, 104.0, 0.0]);
            let replanted = scene_snapshot(&world, &scene);
            runtime.advance(1.0 / 12.0);
            scene.update(&mut world, &runtime.game, runtime.animation_seconds);
            assert_eq!(positions(&world, &scene, 0)[0], [-311.0, 104.0, 0.0]);
            assert_ne!(
                scene_snapshot(&world, &scene),
                replanted,
                "新局再植后继续换帧"
            );
        }
    }

    #[test]
    fn 真实_scene暂停冻结表现时钟恢复后继续变化() {
        let mut world = World::new();
        let mut scene = sunflower_scene_fixture(&mut world);
        let mut runtime = terminal_runtime(false);
        scene.update(&mut world, &runtime.game, runtime.animation_seconds);
        runtime.command(Command::Pause);
        runtime.advance(1.0 / 60.0);
        scene.update(&mut world, &runtime.game, runtime.animation_seconds);
        let paused = (runtime.animation_seconds, scene_snapshot(&world, &scene));
        runtime.advance(100.0);
        scene.update(&mut world, &runtime.game, runtime.animation_seconds);
        assert_eq!(runtime.animation_seconds, paused.0);
        assert_eq!(scene_snapshot(&world, &scene), paused.1);
        runtime.command(Command::Resume);
        runtime.advance(1.0 / 60.0);
        assert_eq!(runtime.animation_seconds, paused.0);
        runtime.advance(1.0 / 12.0);
        scene.update(&mut world, &runtime.game, runtime.animation_seconds);
        assert!(runtime.animation_seconds > paused.0);
        assert_ne!(scene_snapshot(&world, &scene), paused.1);
    }

    #[test]
    fn 同版本核心回放与_scene更新保持摘要事件一致() {
        for with_peashooter in [false, true] {
            let core_only = replay_snapshots(with_peashooter, false);
            let with_scene = replay_snapshots(with_peashooter, true);
            assert_eq!(with_scene, core_only);
        }
    }

    fn pool_handles(scene: &Scene) -> Vec<(Entity, Handle<Mesh>, Handle<ColorMaterial>)> {
        scene
            .tracks
            .iter()
            .map(|track| (track.entity, track.mesh.clone(), track.material.clone()))
            .collect()
    }

    fn assert_zombie_phase(world: &World, scene: &Scene, game: &pvz_core::GameState, phase: usize) {
        use pvz_core::{BoardGeometry, UNITS_PER_PIXEL};
        let zombies = game.board().unwrap().zombies();
        assert!(matches!(phase, 0 | 1));
        for (actor, zombie) in zombies.iter().enumerate() {
            let origin = [
                zombie.x as f32 / UNITS_PER_PIXEL as f32 - 40.0,
                BoardGeometry::row_y(zombie.row) as f32 / UNITS_PER_PIXEL as f32 - 90.0,
            ];
            let bottom_x = if phase == 0 { 7.0 } else { 9.0 };
            for layer in 0..2 {
                let slot = actor * 2 + layer;
                let (local, size, source_index, expected_alpha, image) = match (phase, layer) {
                    (0, 0) => ([bottom_x, 0.0], [2.0, 2.0], 2usize, 0.5, "原创底图"),
                    (0, 1) => ([20.0, 4.0], [4.0, 6.0], 21usize, 1.0, "原创短图"),
                    (1, 0) => ([bottom_x, 0.0], [2.0, 2.0], 2usize, 1.0, "原创上图"),
                    (1, 1) => ([0.0, 0.0], [2.0, 2.0], 4usize, 0.0, "原创短图"),
                    _ => unreachable!(),
                };
                let x = origin[0] + local[0] - 400.0;
                let y = 300.0 - origin[1] - local[1];
                let expected = [
                    [x, y, 0.0],
                    [x + size[0], y, 0.0],
                    [x + size[0], y - size[1], 0.0],
                    [x, y - size[1], 0.0],
                ];
                for (actual, expected) in positions(world, scene, slot).iter().zip(expected) {
                    assert!(
                        actual
                            .iter()
                            .zip(expected)
                            .all(|(a, b)| (a - b).abs() < 0.001),
                        "row {} 源层 {source_index} 坐标：{actual:?}/{expected:?}",
                        zombie.row
                    );
                }
                assert_eq!(positions(world, scene, slot).len(), 4);
                assert_eq!(
                    uvs(world, scene, slot),
                    [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]
                );
                let expected_depth =
                    2.0 + f32::from(zombie.row) * 10.0 + source_index as f32 * 0.01;
                assert!(
                    (world
                        .get::<Transform>(scene.tracks[slot].entity)
                        .unwrap()
                        .translation
                        .z
                        - expected_depth)
                        .abs()
                        < 0.001
                );
                assert_eq!(alpha(world, scene, slot), expected_alpha);
                let material = world
                    .resource::<Assets<ColorMaterial>>()
                    .get(&scene.tracks[slot].material)
                    .unwrap();
                assert_eq!(material.texture.as_ref(), Some(&scene.images[image].0));
                assert_ne!(
                    world.get::<Visibility>(scene.tracks[slot].entity),
                    Some(&Visibility::Hidden)
                );
            }
        }
        assert!(
            scene.tracks[zombies.len() * 2..]
                .iter()
                .all(|track| world.get::<Visibility>(track.entity) == Some(&Visibility::Hidden))
        );
    }

    #[test]
    fn 真实_scene普通僵尸walk源深度纹理坐标alpha与多对象池复用() {
        let mut world = World::new();
        let mut scene = zombie_scene_fixture(&mut world);
        let game = zombie_test_game();
        assert_eq!(
            game.board()
                .unwrap()
                .zombies()
                .iter()
                .map(|z| z.x)
                .collect::<Vec<_>>(),
            [29999, 49999]
        );
        scene.update(&mut world, &game, 0.0);
        assert_eq!(scene.tracks.len(), 4);
        assert_zombie_phase(&world, &scene, &game, 0);
        let pool = pool_handles(&scene);
        let counts = asset_counts(&world);
        assert_eq!(counts, [3, 4, 4]);
        let first = scene_snapshot(&world, &scene);
        scene.update(&mut world, &game, 1.0 / 12.0);
        assert_zombie_phase(&world, &scene, &game, 1);
        assert_eq!(pool_handles(&scene), pool);
        assert_eq!(asset_counts(&world), counts);
        assert!(
            scene
                .tracks
                .iter()
                .all(|t| world.get::<Visibility>(t.entity) == Some(&Visibility::Visible))
        );
        scene.update(&mut world, &game, 2.0 / 12.0);
        assert!(
            scene
                .tracks
                .iter()
                .all(|t| world.get::<Visibility>(t.entity) == Some(&Visibility::Hidden))
        );
        assert_eq!(pool_handles(&scene), pool);
        assert_eq!(asset_counts(&world), counts);
        scene.update(&mut world, &game, 3.0 / 12.0);
        assert_zombie_phase(&world, &scene, &game, 0);
        assert_eq!(scene_snapshot(&world, &scene), first);
        assert_eq!(pool_handles(&scene), pool);
        assert_eq!(asset_counts(&world), counts);
    }

    #[test]
    fn 真实_scene普通僵尸walk实际击杀后只绘幸存者并隐藏尾槽() {
        let mut world = World::new();
        let mut scene = zombie_scene_fixture(&mut world);
        let mut game = zombie_test_game();
        scene.update(&mut world, &game, 0.0);
        let pool = pool_handles(&scene);
        let counts = asset_counts(&world);
        let dead = game
            .board()
            .unwrap()
            .zombies()
            .iter()
            .find(|z| z.row == 0)
            .unwrap()
            .id;
        let survivor = game
            .board()
            .unwrap()
            .zombies()
            .iter()
            .find(|z| z.row == 2)
            .unwrap()
            .id;
        let mut events = game
            .step(&[
                Command::SelectCard {
                    kind: PlantKind::Peashooter,
                },
                Command::Plant {
                    cell: pvz_core::Cell { row: 0, column: 0 },
                },
            ])
            .as_slice()
            .to_vec();
        for _ in 0..4 {
            if !game.board().unwrap().zombies().iter().any(|z| z.id == dead) {
                break;
            }
            events.extend_from_slice(game.step(&[]).as_slice());
        }
        assert!(events.contains(&pvz_core::Event::ZombieDied { id: dead }));
        assert!(
            events
                .iter()
                .any(|e| matches!(e,pvz_core::Event::ProjectileHit { zombie,.. } if *zombie==dead))
        );
        assert!(
            events.iter().any(
                |e| matches!(e,pvz_core::Event::Damaged { target,remaining:0 } if *target==dead)
            )
        );
        assert_eq!(game.board().unwrap().zombies().len(), 1);
        assert_eq!(game.board().unwrap().zombies()[0].id, survivor);
        assert_eq!(
            game.board().unwrap().status(),
            pvz_core::GameStatus::Running
        );
        for (seconds, phase) in [(0.0, 0), (1.0 / 12.0, 1), (3.0 / 12.0, 0)] {
            scene.update(&mut world, &game, seconds);
            assert_zombie_phase(&world, &scene, &game, phase);
            assert_eq!(pool_handles(&scene), pool);
            assert_eq!(asset_counts(&world), counts);
        }
    }

    fn clipping_walk(first_x: f32) -> ZombieWalk {
        let mut xml = String::from(
            "<fps>12</fps><track><name>anim_walk</name><t><f>0</f></t><t/><t><f>-1</f></t></track>",
        );
        for (index, name) in pvz_app::resources::ZOMBIE_TRACKS.iter().enumerate() {
            xml.push_str(&format!("<track><name>{name}</name><t><f>{}</f><x>{first_x}</x><i>原创底图</i></t><t><x>1000</x></t></track>",if index==0 {0}else {-1}));
        }
        ZombieWalk::new(PreparedAnimation::new(
            &pvz_assets::parse_reanim(&xml).unwrap(),
        ))
        .unwrap()
    }

    #[test]
    fn 真实_scene普通僵尸walk部分裁剪uv完全越界及恢复复用池() {
        let mut world = World::new();
        let mut scene = zombie_scene_fixture(&mut world);
        let game = zombie_test_game();
        scene.update(&mut world, &game, 0.0);
        let pool = pool_handles(&scene);
        let counts = asset_counts(&world);
        let original = std::mem::replace(
            &mut scene.zombie_walk,
            clipping_walk(-(29999.0 / 100.0 - 40.0) - 1.0),
        );
        scene.update(&mut world, &game, 0.0);
        let vertices = positions(&world, &scene, 0);
        assert_eq!(vertices.len(), 4);
        assert!((vertices[0][0] + 400.0).abs() < 0.001);
        assert!((vertices[1][0] + 399.0).abs() < 0.001);
        for vertex in vertices {
            assert!((-400.0..=400.0).contains(&vertex[0]) && (-300.0..=300.0).contains(&vertex[1]));
        }
        assert_eq!(
            uvs(&world, &scene, 0),
            [[0.5, 0.0], [1.0, 0.0], [1.0, 1.0], [0.5, 1.0]]
        );
        assert!(
            scene.tracks[2..]
                .iter()
                .all(|t| world.get::<Visibility>(t.entity) == Some(&Visibility::Hidden))
        );
        scene.update(&mut world, &game, 1.0 / 12.0);
        assert!(
            scene
                .tracks
                .iter()
                .all(|t| world.get::<Visibility>(t.entity) == Some(&Visibility::Hidden))
        );
        scene.zombie_walk = original;
        scene.update(&mut world, &game, 0.0);
        assert_zombie_phase(&world, &scene, &game, 0);
        assert_eq!(pool_handles(&scene), pool);
        assert_eq!(asset_counts(&world), counts);
    }

    #[test]
    fn 真实_scene普通僵尸walk额外采样不改变核心状态与实体字段() {
        let mut world = World::new();
        let mut scene = zombie_scene_fixture(&mut world);
        let game = zombie_test_game();
        let before = game.state_digest();
        let zombies = game.board().unwrap().zombies().to_vec();
        let plants = game.board().unwrap().plants().to_vec();
        for seconds in [0.0, 0.12, 1.25, f64::MAX, f64::NAN] {
            scene.update(&mut world, &game, seconds);
            assert_eq!(game.state_digest(), before);
            assert_eq!(game.board().unwrap().zombies(), zombies);
            assert_eq!(game.board().unwrap().plants(), plants);
        }
    }
    fn drawn_zombie_bottom(
        world: &World,
        scene: &Scene,
        game: &pvz_core::GameState,
        seconds: f64,
    ) -> bool {
        let Some(zombie) = game.board().unwrap().zombies().first() else {
            return false;
        };
        let phase = (seconds * 12.0).floor() as usize % 3;
        let depth = 2.0 + f32::from(zombie.row) * 10.0 + 0.02;
        let Some((slot, _)) = scene.tracks.iter().enumerate().find(|(_, track)| {
            world.get::<Visibility>(track.entity) != Some(&Visibility::Hidden)
                && (world.get::<Transform>(track.entity).unwrap().translation.z - depth).abs()
                    < 0.001
        }) else {
            assert_eq!(phase, 2, "可绘相位必须存在实际僵尸图层");
            return false;
        };
        assert_ne!(phase, 2);
        let local = if phase == 0 { 7.0 } else { 9.0 };
        let expected_x = zombie.x as f32 / pvz_core::UNITS_PER_PIXEL as f32 - 40.0 + local - 400.0;
        assert!((positions(world, scene, slot)[0][0] - expected_x).abs() < 0.001);
        let material = world
            .resource::<Assets<ColorMaterial>>()
            .get(&scene.tracks[slot].material)
            .unwrap();
        let image = if phase == 0 {
            "原创底图"
        } else {
            "原创上图"
        };
        assert_eq!(material.texture.as_ref(), Some(&scene.images[image].0));
        assert_eq!(material.color.alpha(), if phase == 0 { 0.5 } else { 1.0 });
        true
    }

    #[test]
    fn 真实_scene普通僵尸walk暂停双终局冻结重开和新波次复用池() {
        for defended in [false, true] {
            let mut world = World::new();
            let mut scene = combined_walk_scene(&mut world);
            let mut runtime = terminal_runtime(defended);
            let mut events = runtime.take_events();
            let mut observed = false;
            for _ in 0..1000 {
                runtime.advance(0.25);
                events.extend(runtime.take_events());
                scene.update(&mut world, &runtime.game, runtime.animation_seconds);
                if !runtime.game.board().unwrap().zombies().is_empty()
                    && drawn_zombie_bottom(&world, &scene, &runtime.game, runtime.animation_seconds)
                {
                    observed = true;
                    break;
                }
            }
            assert!(observed, "必须实际绘制存活僵尸动态层");
            runtime.command(Command::Pause);
            runtime.advance(1.0 / 60.0);
            events.extend(runtime.take_events());
            scene.update(&mut world, &runtime.game, runtime.animation_seconds);
            let paused = (
                runtime.game.state_digest(),
                runtime.animation_seconds,
                scene_snapshot(&world, &scene),
            );
            runtime.advance(100.0);
            scene.update(&mut world, &runtime.game, runtime.animation_seconds);
            assert_eq!(
                (
                    runtime.game.state_digest(),
                    runtime.animation_seconds,
                    scene_snapshot(&world, &scene)
                ),
                paused
            );
            assert!(runtime.take_events().is_empty());
            runtime.command(Command::Resume);
            runtime.advance(1.0 / 60.0);
            events.extend(runtime.take_events());
            assert_eq!(runtime.animation_seconds, paused.1);
            runtime.advance(1.0 / 12.0);
            events.extend(runtime.take_events());
            scene.update(&mut world, &runtime.game, runtime.animation_seconds);
            assert_ne!(scene_snapshot(&world, &scene), paused.2);
            assert!(drawn_zombie_bottom(
                &world,
                &scene,
                &runtime.game,
                runtime.animation_seconds
            ));
            for _ in 0..1000 {
                if runtime.game.board().unwrap().status() != pvz_core::GameStatus::Running {
                    break;
                }
                runtime.advance(0.25);
                events.extend(runtime.take_events());
                scene.update(&mut world, &runtime.game, runtime.animation_seconds);
            }
            assert_eq!(
                runtime.game.board().unwrap().status(),
                if defended {
                    pvz_core::GameStatus::Won
                } else {
                    pvz_core::GameStatus::Lost
                }
            );
            assert!(
                runtime
                    .game
                    .board()
                    .unwrap()
                    .plants()
                    .iter()
                    .any(|p| p.kind == PlantKind::Sunflower && p.cell.row == 1)
            );
            assert_eq!(
                events
                    .iter()
                    .filter(|e| matches!(
                        (defended, *e),
                        (true, pvz_core::Event::Victory) | (false, pvz_core::Event::Defeat)
                    ))
                    .count(),
                1
            );
            assert_eq!(runtime.game.board().unwrap().zombies().is_empty(), defended);
            let terminal = (
                runtime.game.state_digest(),
                runtime.animation_seconds,
                scene_snapshot(&world, &scene),
            );
            let counts = asset_counts(&world);
            let pool = pool_handles(&scene);
            runtime.advance(100.0);
            scene.update(&mut world, &runtime.game, runtime.animation_seconds);
            assert_eq!(
                (
                    runtime.game.state_digest(),
                    runtime.animation_seconds,
                    scene_snapshot(&world, &scene)
                ),
                terminal
            );
            assert!(runtime.take_events().is_empty());
            runtime.command(Command::Restart);
            runtime.advance(1.0 / 60.0);
            scene.update(&mut world, &runtime.game, runtime.animation_seconds);
            assert_eq!(runtime.game.tick(), 0);
            assert_eq!(runtime.animation_seconds, 0.0);
            assert!(
                scene
                    .tracks
                    .iter()
                    .all(|t| world.get::<Visibility>(t.entity) == Some(&Visibility::Hidden))
            );
            runtime.command(Command::SelectCard {
                kind: PlantKind::Sunflower,
            });
            runtime.command(Command::Plant {
                cell: pvz_core::Cell { row: 1, column: 0 },
            });
            runtime.advance(1.0 / 60.0);
            scene.update(&mut world, &runtime.game, runtime.animation_seconds);
            assert_eq!(runtime.game.board().unwrap().plants().len(), 1);
            let replanted = scene_snapshot(&world, &scene);
            runtime.advance(1.0 / 12.0);
            scene.update(&mut world, &runtime.game, runtime.animation_seconds);
            assert_ne!(
                scene_snapshot(&world, &scene),
                replanted,
                "再植向日葵继续动态idle"
            );
            for _ in 0..120 {
                if runtime.game.tick() >= 100 {
                    break;
                }
                runtime.advance(1.0 / 60.0);
                scene.update(&mut world, &runtime.game, runtime.animation_seconds);
            }
            assert!(
                !runtime.game.board().unwrap().zombies().is_empty(),
                "重开后新波次实际出现"
            );
            for _ in 0..6 {
                if drawn_zombie_bottom(&world, &scene, &runtime.game, runtime.animation_seconds) {
                    break;
                }
                runtime.advance(1.0 / 60.0);
                scene.update(&mut world, &runtime.game, runtime.animation_seconds);
            }
            assert!(drawn_zombie_bottom(
                &world,
                &scene,
                &runtime.game,
                runtime.animation_seconds
            ));
            assert_eq!(pool_handles(&scene), pool);
            assert_eq!(asset_counts(&world), counts);
        }
    }

    #[test]
    fn 普通僵尸walk同版本逐批回放包含伤害死亡暂停终局重开和移除() {
        for defended in [false, true] {
            let core_only = replay_snapshots(defended, false);
            let with_scene = replay_snapshots(defended, true);
            assert_eq!(with_scene, core_only);
            let events: Vec<_> = with_scene.iter().flat_map(|batch| batch.1.iter()).collect();
            assert!(events.iter().any(|e| matches!(e, pvz_core::Event::Paused)));
            assert!(events.iter().any(|e| matches!(e, pvz_core::Event::Resumed)));
            assert!(
                events
                    .iter()
                    .any(|e| matches!(e, pvz_core::Event::Restarted))
            );
            assert!(
                events
                    .iter()
                    .any(|e| matches!(e, pvz_core::Event::ZombieDied { .. }))
            );
            assert!(
                events
                    .iter()
                    .any(|e| matches!(e, pvz_core::Event::PlantRemoved { .. }))
            );
            assert!(
                events
                    .iter()
                    .any(|e| matches!(e, pvz_core::Event::ZombieSpawned { .. }))
            );
            if defended {
                assert!(
                    events
                        .iter()
                        .any(|e| matches!(e, pvz_core::Event::ProjectileHit { .. }))
                );
                assert!(
                    events
                        .iter()
                        .any(|e| matches!(e, pvz_core::Event::Damaged { .. }))
                );
                assert!(events.iter().any(|e| matches!(e, pvz_core::Event::Victory)));
            } else {
                assert!(events.iter().any(|e| matches!(e, pvz_core::Event::Defeat)));
            }
        }
    }
}
