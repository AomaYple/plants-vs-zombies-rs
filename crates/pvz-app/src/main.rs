mod render;

use bevy::{
    app::AppExit,
    audio::{AudioSinkPlayback, Volume},
    prelude::*,
    tasks::{IoTaskPool, Task, futures::check_ready},
    window::{PrimaryWindow, WindowEvent},
    winit::WinitSettings,
};
use pvz_app::{
    input::InputEvent,
    platform_input::PlatformInput,
    resources::{LoadedResources, ResourceError},
    runtime::Runtime,
    viewport::{LogicalPoint, Viewport},
};

#[derive(clap::Parser, Clone)]
#[command(about = "跨平台视口与登记资源验证应用")]
struct Options {
    /// 使用原创占位探针，不加载原版资源。
    #[arg(long)]
    probe: bool,
    /// 隐藏窗口真正绘制指定数量的帧后自动退出。
    #[arg(long, value_parser = clap::value_parser!(u32).range(2..))]
    smoke: Option<u32>,
    /// 冒烟退出前把 GPU 截图写到忽略的 artifacts 目录。
    #[arg(long, requires = "smoke")]
    screenshot: bool,
    /// 在隐藏窗口冒烟中实际验证一次音效、音乐循环与游标暂停/恢复。
    #[arg(long, requires = "smoke")]
    audio_smoke: bool,
}
#[derive(Resource)]
struct AppState {
    options: Options,
    runtime: Runtime,
    input: PlatformInput,
    pointer: Option<LogicalPoint>,
    camera: Entity,
    status: Entity,
    error_text: Entity,
    scene: Option<render::Scene>,
    pending: Option<Task<Result<LoadedResources, ResourceError>>>,
    ready: bool,
    failure: Option<String>,
    effect: Option<Handle<AudioSource>>,
    music: Option<Handle<AudioSource>>,
    music_entity: Option<Entity>,
    last_view: (u32, u32, f32),
    screenshot_requested: bool,
    loading_started: f64,
    paused_audio_position: Option<std::time::Duration>,
}

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    let mut options = <Options as clap::Parser>::parse();
    #[cfg(target_arch = "wasm32")]
    let mut options = Options {
        probe: false,
        smoke: None,
        screenshot: false,
        audio_smoke: false,
    };
    options.probe |= cfg!(feature = "probe");
    if options.audio_smoke && (options.probe || options.smoke.is_none_or(|frames| frames < 120)) {
        <Options as clap::CommandFactory>::command()
            .error(
                clap::error::ErrorKind::ValueValidation,
                "音频冒烟需要登记资源模式和至少 120 帧",
            )
            .exit();
    }
    let smoke = options.smoke.is_some();
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "植物大战僵尸 · Bevy 视口验证".to_owned(),
            resolution: (1000, 750).into(),
            visible: !smoke,
            focused: !smoke,
            canvas: Some("#pvz-canvas".to_owned()),
            fit_canvas_to_parent: true,
            ..default()
        }),
        ..default()
    }))
    .insert_resource(ClearColor(Color::srgb(0.04, 0.06, 0.1)))
    .insert_resource(WinitSettings::continuous());
    app.add_systems(Startup, move |world: &mut World| {
        initialize(world, options.clone())
    });
    app.add_systems(Update, update);
    app.run();
}

fn initialize(world: &mut World, options: Options) {
    let font = world.resource_mut::<Assets<Font>>().add(Font::from_bytes(
        include_bytes!("../../../assets/third-party/noto-sans-sc.ttf").to_vec(),
    ));
    let camera = render::create_camera(world);
    let (status, error_text) = render::diagnostics(world, &font, options.probe);
    let pending = if options.probe {
        None
    } else {
        #[cfg(not(target_arch = "wasm32"))]
        let task = IoTaskPool::get().spawn(LoadedResources::load());
        #[cfg(target_arch = "wasm32")]
        let task = IoTaskPool::get().spawn_local(LoadedResources::load());
        Some(task)
    };
    let ready = options.probe;
    let loading_started = world.resource::<Time<Real>>().elapsed_secs_f64();
    world.insert_resource(AppState {
        options,
        runtime: Runtime::default(),
        input: PlatformInput::default(),
        pointer: None,
        camera,
        status,
        error_text,
        scene: None,
        pending,
        ready,
        failure: None,
        effect: None,
        music: None,
        music_entity: None,
        last_view: (0, 0, 0.0),
        screenshot_requested: false,
        loading_started,
        paused_audio_position: None,
    });
}

fn poll_resources(world: &mut World, state: &mut AppState) {
    if let Some(result) = state.pending.as_mut().and_then(check_ready) {
        state.pending = None;
        state.ready = true;
        match result {
            Ok(mut resources) => {
                println!(
                    "登记资源已校验：背景、{} 张图像、PeaShooter；音乐 {}Hz/{} 声道/{} 样本",
                    resources.images.len(),
                    resources.music_info.sample_rate,
                    resources.music_info.channels,
                    resources.music_info.samples
                );
                let mut sources = world.resource_mut::<Assets<AudioSource>>();
                state.effect = Some(sources.add(AudioSource {
                    bytes: std::mem::take(&mut resources.effect).into(),
                }));
                state.music = Some(sources.add(AudioSource {
                    bytes: std::mem::take(&mut resources.music).into(),
                }));
                state.scene = Some(render::Scene::upload(world, resources));
            }
            Err(error) => {
                let error = error.to_string();
                eprintln!("{error}");
                world
                    .get_mut::<Text2d>(state.error_text)
                    .expect("错误文本存在")
                    .0
                    .clone_from(&error);
                state.failure = Some(error);
            }
        }
    }
    if state.pending.is_some()
        && world.resource::<Time<Real>>().elapsed_secs_f64() - state.loading_started > 30.0
    {
        state.pending = None;
        state.ready = true;
        let error = "资源下载或解码超过开发等待上限（30 秒）".to_owned();
        world
            .get_mut::<Text2d>(state.error_text)
            .expect("错误文本存在")
            .0
            .clone_from(&error);
        eprintln!("{error}");
        state.failure = Some(error);
    }
}

fn apply_input(state: &mut AppState, event: InputEvent) {
    match event {
        InputEvent::PointerDown { point, .. }
        | InputEvent::PointerUp { point, .. }
        | InputEvent::PointerMove { point, .. } => state.pointer = Some(point),
        InputEvent::PointerCancel { .. } => state.pointer = None,
        _ => {}
    }
    if matches!(
        event,
        InputEvent::PointerDown { .. }
            | InputEvent::PointerUp { .. }
            | InputEvent::PointerCancel { .. }
    ) {
        println!("逻辑输入探针：{event:?}");
    }
    state.runtime.input(event);
}

fn keyboard_audio(world: &mut World, state: &mut AppState, pause: bool, effect: bool, music: bool) {
    if pause {
        state.runtime.input(if state.runtime.paused {
            InputEvent::Resume
        } else {
            InputEvent::Pause
        });
        if let Some(sink) = state.music_entity.and_then(|e| world.get::<AudioSink>(e)) {
            let before = sink.position();
            if state.runtime.paused {
                sink.pause();
            } else {
                sink.play();
            }
            println!(
                "音乐游标探针：暂停={} 位置={before:?} 操作后位置={:?}",
                sink.is_paused(),
                sink.position()
            );
        }
    }
    if effect && let Some(source) = &state.effect {
        world.spawn((AudioPlayer::new(source.clone()), PlaybackSettings::DESPAWN));
        println!("音效探针：已提交一次播放请求");
    }
    if music && let Some(source) = &state.music {
        if let Some(entity) = state.music_entity.take() {
            world.despawn(entity);
            println!("音乐探针：已停止循环");
        } else {
            let settings = PlaybackSettings::LOOP.with_volume(Volume::Linear(0.25));
            state.music_entity = Some(
                world
                    .spawn((
                        AudioPlayer::new(source.clone()),
                        if state.runtime.paused {
                            settings.paused()
                        } else {
                            settings
                        },
                    ))
                    .id(),
            );
            println!("音乐探针：已提交循环请求；Web 需要用户手势解锁");
        }
    }
}

fn update(world: &mut World) {
    world.resource_scope(|world, mut state: Mut<AppState>| {
        poll_resources(world, &mut state);
        let mut window_query = world.query_filtered::<(Entity, &Window), With<PrimaryWindow>>();
        let Ok((window_entity, window)) = window_query.single(world) else { return; };
        let size = (window.physical_width(), window.physical_height());
        let dpi = window.scale_factor();
        let view = Viewport::fit((size.0 as f32, size.1 as f32), render::LOGICAL_SIZE);
        if state.last_view != (size.0, size.1, dpi) {
            println!("视口探针：模式={} framebuffer={}×{} DPI={dpi:.3} 比例={:.3} 偏移=({:.1},{:.1}) 中心映射={:?}", if state.options.probe { "原创占位" } else { "登记资源" }, size.0, size.1, view.scale, view.offset.x, view.offset.y, view.to_logical(LogicalPoint { x: size.0 as f32 / 2.0, y: size.1 as f32 / 2.0 }));
            render::update_viewport(world, state.camera, view);
            state.last_view = (size.0, size.1, dpi);
        }
        let keyboard = world.resource::<ButtonInput<KeyCode>>();
        let pause = keyboard.just_pressed(KeyCode::KeyP) || keyboard.just_pressed(KeyCode::Space);
        let effect = keyboard.just_pressed(KeyCode::KeyE);
        let music = keyboard.just_pressed(KeyCode::KeyM);
        keyboard_audio(world, &mut state, pause, effect, music);
        let events = state.input.read(window_entity, view, dpi, world.resource::<Messages<WindowEvent>>());
        for event in events { apply_input(&mut state, event); }
        if state.ready { state.runtime.advance(world.resource::<Time<Real>>().delta_secs_f64()); }
        audio_smoke(world, &mut state);
        if let Some(scene) = &state.scene { scene.update(world, state.runtime.animation_seconds); }
        let audio_status = if state.options.probe { "占位模式无音频".to_owned() } else if let Some(sink) = state.music_entity.and_then(|e| world.get::<AudioSink>(e)) { format!("音乐 {:.2}s 暂停={}", sink.position().as_secs_f64(), sink.is_paused()) } else { "音乐已停止/等待 E、M 用户手势".to_owned() };
        world.get_mut::<Text2d>(state.status).expect("状态文本存在").0 = format!("tick {} · 动画 {:.2}s · 帧 {} · 阳光 {} · DPI {dpi:.2}\nP/空格 暂停 · E 音效 · M 循环 · {audio_status}\n逻辑输入 {:?} · 模式 {}", state.runtime.game.clock().current_tick(), state.runtime.animation_seconds, state.runtime.render_frames, state.runtime.game.sun(), state.pointer, if state.failure.is_some() { "资源失败" } else if !state.ready { "加载中" } else if state.options.probe { "原创探针" } else { "登记资源" });
        if let Some(error) = &state.failure { world.get_mut::<Text2d>(state.error_text).expect("错误文本存在").0.clone_from(error); }
        smoke(world, &mut state);
    });
}

fn audio_smoke(world: &mut World, state: &mut AppState) {
    if !state.options.audio_smoke || state.failure.is_some() {
        return;
    }
    let frame = state.runtime.render_frames;
    match frame {
        10 => keyboard_audio(world, state, false, true, true),
        30 | 70 => keyboard_audio(world, state, true, false, false),
        40 => {
            state.paused_audio_position = state
                .music_entity
                .and_then(|e| world.get::<AudioSink>(e))
                .filter(|s| s.is_paused())
                .map(AudioSinkPlayback::position);
        }
        60 => {
            let position = state
                .music_entity
                .and_then(|e| world.get::<AudioSink>(e))
                .filter(|s| s.is_paused())
                .map(AudioSinkPlayback::position);
            if position.is_none() || position != state.paused_audio_position {
                state.failure = Some("音频冒烟失败：没有可用音频设备或暂停游标发生变化".to_owned());
            }
            println!(
                "音频暂停游标检查：初值={:?} 后值={position:?} 通过={}",
                state.paused_audio_position,
                state.failure.is_none()
            );
        }
        100 => {
            let position = state
                .music_entity
                .and_then(|e| world.get::<AudioSink>(e))
                .filter(|s| !s.is_paused())
                .map(AudioSinkPlayback::position);
            let passed = position
                .zip(state.paused_audio_position)
                .is_some_and(|(after, before)| after > before);
            if !passed {
                state.failure = Some("音频冒烟失败：恢复后游标没有继续推进".to_owned());
            }
            println!(
                "音频恢复游标检查：暂停={:?} 恢复={position:?} 通过={passed}",
                state.paused_audio_position
            );
            keyboard_audio(world, state, false, false, true);
        }
        _ => {}
    }
}

fn smoke(world: &mut World, state: &mut AppState) {
    if !state
        .options
        .smoke
        .is_some_and(|frames| state.runtime.render_frames >= u64::from(frames))
        || state.screenshot_requested
    {
        return;
    }
    state.screenshot_requested = true;
    #[cfg(not(target_arch = "wasm32"))]
    if state.options.screenshot {
        use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};
        let path = if state.options.probe {
            "artifacts/viewport/probe.png"
        } else {
            "artifacts/viewport/registered.png"
        };
        let failed = state.failure.is_some();
        world.spawn(Screenshot::primary_window()).observe(
            move |capture: On<ScreenshotCaptured>, mut exit: MessageWriter<AppExit>| {
                let saved = std::fs::create_dir_all("artifacts/viewport").is_ok()
                    && capture
                        .image
                        .clone()
                        .try_into_dynamic()
                        .is_ok_and(|image| image.save(path).is_ok());
                println!(
                    "GPU 截图：{}×{} 保存={} 路径={path}",
                    capture.image.width(),
                    capture.image.height(),
                    saved
                );
                exit.write(if saved && !failed {
                    AppExit::Success
                } else {
                    AppExit::error()
                });
            },
        );
        return;
    }
    println!(
        "窗口冒烟绘制结束：{} 帧，资源状态={}",
        state.runtime.render_frames,
        if state.failure.is_none() {
            "通过"
        } else {
            "失败"
        }
    );
    world.write_message(if state.failure.is_none() {
        AppExit::Success
    } else {
        AppExit::error()
    });
}
