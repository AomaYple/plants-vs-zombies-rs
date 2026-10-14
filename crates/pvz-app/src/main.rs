mod render;
#[cfg(not(target_arch = "wasm32"))]
mod screenshot_wait;

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
    resources::{LoadedResources, ResourceError, SoundCue},
    runtime::{Runtime, sound_cue},
    viewport::{LogicalPoint, Viewport},
};

#[derive(clap::Parser, Clone)]
#[command(about = "白天可玩切片与跨平台登记资源验证")]
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
    /// 隐藏窗口中的固定切片输入脚本，证据写入独立目录。
    #[arg(long, requires = "smoke")]
    slice_smoke: bool,
    /// 隐藏窗口调用与 F 键相同的逻辑入口；加速等待开发失败并校验终局。
    #[arg(long, requires = "smoke", conflicts_with_all = ["probe", "slice_smoke", "audio_smoke"])]
    failure_smoke: bool,
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
    #[cfg(not(target_arch = "wasm32"))]
    screenshot_wait: Option<screenshot_wait::ScreenshotWait>,
    loading_started: f64,
    paused_audio_position: Option<std::time::Duration>,
    effects: std::collections::BTreeMap<SoundCue, Handle<AudioSource>>,
    feedback: String,
}

fn main() -> AppExit {
    #[cfg(not(target_arch = "wasm32"))]
    let mut options = <Options as clap::Parser>::parse();
    #[cfg(target_arch = "wasm32")]
    let mut options = Options {
        probe: false,
        smoke: None,
        screenshot: false,
        audio_smoke: false,
        slice_smoke: false,
        failure_smoke: false,
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
    if options.slice_smoke && options.probe {
        <Options as clap::CommandFactory>::command()
            .error(
                clap::error::ErrorKind::ValueValidation,
                "切片冒烟需要登记资源模式",
            )
            .exit();
    }
    if options.failure_smoke && options.probe {
        <Options as clap::CommandFactory>::command()
            .error(
                clap::error::ErrorKind::ValueValidation,
                "失败冒烟需要登记资源模式，不能与探针 feature 共用",
            )
            .exit();
    }
    let smoke = options.smoke.is_some();
    let mut app = App::new();
    // 冒烟不能在异步管线仍编译时把不完整截图当作绘制成功；正常游戏保留异步编译。
    app.add_plugins(
        DefaultPlugins
            .set(bevy::render::RenderPlugin {
                synchronous_pipeline_compilation: smoke,
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "植物大战僵尸 · 白天开发切片".to_owned(),
                    resolution: (1000, 750).into(),
                    visible: !smoke,
                    focused: !smoke,
                    canvas: Some("#pvz-canvas".to_owned()),
                    fit_canvas_to_parent: true,
                    ..default()
                }),
                ..default()
            }),
    )
    .insert_resource(ClearColor(Color::srgb(0.04, 0.06, 0.1)))
    .insert_resource(WinitSettings::continuous());
    app.add_systems(Startup, move |world: &mut World| {
        initialize(world, options.clone())
    });
    app.add_systems(Update, update);
    app.run()
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
    let runtime = if options.probe {
        Runtime::default()
    } else {
        Runtime::day()
    };
    world.insert_resource(AppState {
        options,
        runtime,
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
        #[cfg(not(target_arch = "wasm32"))]
        screenshot_wait: None,
        loading_started,
        paused_audio_position: None,
        effects: std::collections::BTreeMap::new(),
        feedback: String::new(),
    });
}

fn poll_resources(world: &mut World, state: &mut AppState) {
    if let Some(result) = state.pending.as_mut().and_then(check_ready) {
        state.pending = None;
        state.ready = true;
        match result {
            Ok(mut resources) => {
                println!(
                    "切片登记资源已校验：背景、{} 张图像、六类构图；音乐 {}Hz/{} 声道/{} 样本",
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
                for (cue, bytes) in std::mem::take(&mut resources.effects) {
                    state.effects.insert(
                        cue,
                        sources.add(AudioSource {
                            bytes: bytes.into(),
                        }),
                    );
                }
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
    if (state.options.probe || state.options.smoke.is_some())
        && matches!(
            event,
            InputEvent::PointerDown { .. }
                | InputEvent::PointerUp { .. }
                | InputEvent::PointerCancel { .. }
        )
    {
        println!("逻辑输入探针：{event:?}");
    }
    if event == InputEvent::LoadFailureScenario && !state.options.probe {
        state.feedback = "已载入开发失败场景（F）".to_owned();
    }
    state.runtime.input(event);
}

fn keyboard_audio(world: &mut World, state: &mut AppState, pause: bool, effect: bool, music: bool) {
    if pause {
        state.runtime.input(InputEvent::TogglePause);
        if state.options.probe
            && let Some(sink) = state.music_entity.and_then(|e| world.get::<AudioSink>(e))
        {
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
        #[cfg(not(target_arch = "wasm32"))]
        if state.options.smoke.is_some()
            && state.screenshot_requested
            && state.screenshot_wait.is_some()
        {
            smoke(world, &mut state);
            return;
        }
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
        let events = state.input.read(window_entity, size, dpi, render::LOGICAL_SIZE, world.resource::<Messages<WindowEvent>>());
        for event in events {
            match event {
                InputEvent::TogglePause => keyboard_audio(world, &mut state, true, false, false),
                InputEvent::PlayEffect if state.options.probe || state.options.audio_smoke => keyboard_audio(world, &mut state, false, true, false),
                InputEvent::ToggleMusic => keyboard_audio(world, &mut state, false, false, true),
                _ => apply_input(&mut state, event),
            }
        }
        if state.ready && state.failure.is_some() { state.runtime.render_frames += 1; }
        if state.ready && state.failure.is_none() {
            slice_script(&mut state);
            failure_script(&mut state);
            let elapsed = if state.options.slice_smoke || state.options.failure_smoke { 0.25 } else { world.resource::<Time<Real>>().delta_secs_f64() };
            state.runtime.advance(elapsed);
        }
        audio_smoke(world, &mut state);
        consume_events(world, &mut state);
        sync_music(world, &state);
        let mut scene = state.scene.take();
        if let Some(scene) = &mut scene { scene.update(world, &state.runtime.game, state.runtime.animation_seconds); }
        state.scene = scene;
        let audio_status = if state.options.probe { "占位模式无音频".to_owned() } else if let Some(sink) = state.music_entity.and_then(|e| world.get::<AudioSink>(e)) { format!("音乐 {:.2}s 暂停={}", sink.position().as_secs_f64(), sink.is_paused()) } else { "音乐已停止/等待 E、M 用户手势".to_owned() };
        if state.options.probe { world.get_mut::<Text2d>(state.status).expect("状态文本存在").0 = format!("tick {} · 动画 {:.2}s · 帧 {} · 阳光 {} · DPI {dpi:.2}\nP/空格 暂停 · E 音效 · M 循环 · {audio_status}\n逻辑输入 {:?} · 模式 {}", state.runtime.game.clock().current_tick(), state.runtime.animation_seconds, state.runtime.render_frames, state.runtime.game.sun(), state.pointer, if state.failure.is_some() { "资源失败" } else if !state.ready { "加载中" } else if state.options.probe { "原创探针" } else { "登记资源" });
        } else {
            let board = state.runtime.game.board().expect("正常游戏拥有棋盘");
            let selection = if state.runtime.shovel { "铲除" } else { match board.selected() { Some(pvz_core::PlantKind::Sunflower) => "向日葵", Some(pvz_core::PlantKind::Peashooter) => "豌豆射手", None => "请选卡" } };
            let status = if state.failure.is_some() { "资源加载失败" } else if !state.ready { "加载资源" } else if board.paused() { "已暂停" } else { match board.status() { pvz_core::GameStatus::Running => "守住草坪", pvz_core::GameStatus::Won => "胜利！", pvz_core::GameStatus::Lost => "失败，请重开" } };
            world.get_mut::<Text2d>(state.status).expect("状态文本存在").0 = format!("阳光 {}  ·  选择：{selection}  ·  {status}  ·  {} / {} 只已出现\n开发规则 2.0（尚未还原） · 冷却(日/豌) {}/{}秒 · F 失败场景 · {}", state.runtime.game.sun(), board.spawned(), board.rules().wave.len(), board.cooldown_remaining(pvz_core::PlantKind::Sunflower, state.runtime.game.tick()).div_ceil(60), board.cooldown_remaining(pvz_core::PlantKind::Peashooter, state.runtime.game.tick()).div_ceil(60), state.feedback);
        }
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

fn consume_events(world: &mut World, state: &mut AppState) {
    use pvz_core::Event;
    for event in state.runtime.take_events() {
        if let Some(cue) = sound_cue(&event)
            && let Some(source) = state.effects.get(&cue)
        {
            world.spawn((AudioPlayer::new(source.clone()), PlaybackSettings::DESPAWN));
        }
        let feedback = match event {
            Event::Planted { .. } => Some("种植成功"),
            Event::SunCollected { .. } => Some("已收集阳光"),
            Event::ProjectileHit { .. } => Some("豌豆命中"),
            Event::MowerStarted { .. } => Some("割草机启动！"),
            Event::PlantRemoved { .. } => Some("植物已移除"),
            Event::Victory => Some("本波僵尸已消灭"),
            Event::Defeat => Some("僵尸突破防线"),
            Event::Restarted => Some("已重开"),
            Event::CommandRejected { reason } => Some(match reason {
                pvz_core::Rejection::NotEnoughSun => "阳光不足",
                pvz_core::Rejection::CoolingDown => "卡片冷却中",
                pvz_core::Rejection::Occupied => "此格已有植物",
                pvz_core::Rejection::NoSelection => "请先选择植物",
                pvz_core::Rejection::Paused => "请先恢复游戏",
                pvz_core::Rejection::Ended => "本局结束，请重开",
                _ => "当前操作无效",
            }),
            _ => None,
        };
        if let Some(text) = feedback {
            state.feedback = text.to_owned();
        }
        if (state.options.slice_smoke || state.options.failure_smoke)
            && !matches!(
                event,
                Event::TickStarted { .. } | Event::TickCompleted { .. }
            )
        {
            println!(
                "切片冒烟事件 帧末tick={} {event:?}",
                state.runtime.game.tick()
            );
        }
    }
}

fn sync_music(world: &mut World, state: &AppState) {
    let Some(entity) = state.music_entity else {
        return;
    };
    // 模拟执行后的状态同时交给待创建和现成的 sink，迟到后端不依赖已消费的事件。
    if let Some(mut settings) = world.get_mut::<PlaybackSettings>(entity) {
        settings.paused = state.runtime.paused;
    }
    if let Some(sink) = world.get::<AudioSink>(entity) {
        if state.runtime.paused {
            sink.pause();
        } else {
            sink.play();
        }
    }
}

/// 只提交正常命令，脚本不修改阳光、伤害或模拟状态。
fn slice_script(state: &mut AppState) {
    use pvz_core::{Cell, Command, PlantKind};
    if !state.options.slice_smoke {
        return;
    }
    let frame = state.runtime.render_frames;
    if frame == 0 {
        state.runtime.command(Command::SelectCard {
            kind: PlantKind::Sunflower,
        });
        state.runtime.command(Command::Plant {
            cell: Cell { row: 1, column: 0 },
        });
        state.runtime.command(Command::SelectCard {
            kind: PlantKind::Peashooter,
        });
        state.runtime.command(Command::Plant {
            cell: Cell { row: 2, column: 0 },
        });
    }
    if frame == 5 {
        state.runtime.command(Command::Pause);
    }
    if frame == 8 {
        state.runtime.command(Command::Resume);
    }
    let ids: Vec<_> = state
        .runtime
        .game
        .board()
        .expect("切片冒烟拥有棋盘")
        .suns()
        .iter()
        .map(|s| s.id)
        .collect();
    for id in ids {
        state.runtime.command(Command::CollectSun { id });
    }
}

/// 只通过正常入口的场景选择，不修改阳光、伤害或模拟规则。
fn failure_script(state: &mut AppState) {
    if state.options.failure_smoke && state.runtime.render_frames == 0 {
        apply_input(state, InputEvent::LoadFailureScenario);
        println!("失败场景冒烟：调用 F 键共用逻辑入口；不代表设备键盘实跑");
    }
}

fn smoke(world: &mut World, state: &mut AppState) {
    #[cfg(not(target_arch = "wasm32"))]
    if let Some(wait) = &mut state.screenshot_wait {
        if let Some(result) = wait.poll(world.resource::<Time<Real>>().elapsed()) {
            if let Err(error) = &result {
                eprintln!("{error}");
            }
            world.write_message(if result.is_ok() && state.failure.is_none() {
                AppExit::Success
            } else {
                AppExit::error()
            });
        }
        return;
    }

    if !state
        .options
        .smoke
        .is_some_and(|frames| state.runtime.render_frames >= u64::from(frames))
        || state.screenshot_requested
    {
        return;
    }
    state.screenshot_requested = true;
    if state.options.failure_smoke
        && state
            .runtime
            .game
            .board()
            .is_none_or(|board| board.status() != pvz_core::GameStatus::Lost)
    {
        state.failure = Some("失败冒烟未在指定帧数内观察到开发关卡失败".to_owned());
    }
    #[cfg(not(target_arch = "wasm32"))]
    if state.options.screenshot {
        use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};
        let path = if state.options.probe {
            "artifacts/viewport/probe.png"
        } else if state.options.failure_smoke {
            "artifacts/vertical-slice/failure.png"
        } else {
            "artifacts/vertical-slice/registered.png"
        };
        state.screenshot_wait = Some(screenshot_wait::ScreenshotWait::new(
            world.resource::<Time<Real>>().elapsed(),
        ));
        println!(
            "GPU 截图等待冻结：tick={} animation_seconds={:.9} render_frames={}",
            state.runtime.game.clock().current_tick(),
            state.runtime.animation_seconds,
            state.runtime.render_frames
        );
        world.spawn(Screenshot::primary_window()).observe(
            move |capture: On<ScreenshotCaptured>,
                  time: Res<Time<Real>>,
                  mut state: ResMut<AppState>| {
                let saved = save_screenshot(
                    &capture.image,
                    std::path::Path::new(path),
                    state.options.probe,
                );
                println!(
                    "GPU 截图：{}×{} 保存={} 路径={path}",
                    capture.image.width(),
                    capture.image.height(),
                    saved.is_ok()
                );
                if let Some(wait) = &mut state.screenshot_wait {
                    wait.complete(time.elapsed(), saved);
                }
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

#[cfg(not(target_arch = "wasm32"))]
fn save_screenshot(image: &Image, path: &std::path::Path, probe: bool) -> Result<(), String> {
    let image = image
        .clone()
        .try_into_dynamic()
        .map_err(|error| format!("GPU 截图保存失败：转换图像：{error}"))?;
    if probe {
        // 只检查右下角原创红蓝夹具实际出现，不对 GPU/驱动的整帧像素强制完全相等。
        let (width, height) = (u64::from(image.width()), u64::from(image.height()));
        let (mut red, mut blue) = (0, 0);
        for (x, y, pixel) in image::GenericImageView::pixels(&image) {
            if u64::from(x) * 5 < width * 4
                || u64::from(y) * 3 < height * 2
                || u64::from(y) * 15 >= height * 14
            {
                continue;
            }
            let [r, g, b, a] = pixel.0;
            red += usize::from(a > 200 && r > 180 && g < 100 && b < 100);
            blue += usize::from(a > 200 && r < 100 && g > 100 && b > 180);
            if red >= 8 && blue >= 8 {
                break;
            }
        }
        if red < 8 || blue < 8 {
            return Err("GPU 探针绘制未完成：截图缺少右下角原创红蓝夹具".to_owned());
        }
    }
    std::fs::create_dir_all(path.parent().expect("截图具有父目录"))
        .map_err(|error| format!("GPU 截图保存失败：创建目录：{error}"))?;
    image
        .save(path)
        .map_err(|error| format!("GPU 截图保存失败：写入图像：{error}"))
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use bevy::input::{
        ButtonState,
        keyboard::{Key, KeyboardInput, NativeKey},
        touch::{TouchInput, TouchPhase},
    };
    use std::time::Duration;

    fn harness() -> World {
        let mut world = World::new();
        world.insert_resource(Time::<Real>::default());
        world.insert_resource(Messages::<AppExit>::default());
        world.insert_resource(Messages::<WindowEvent>::default());
        world.insert_resource(ButtonInput::<KeyCode>::default());
        world.spawn((
            PrimaryWindow,
            Window {
                resolution: (800, 600).into(),
                ..default()
            },
        ));
        let status = world.spawn(Text2d::default()).id();
        let error_text = world.spawn(Text2d::default()).id();
        let camera = world.spawn_empty().id();
        world.insert_resource(AppState {
            options: Options {
                probe: false,
                smoke: None,
                screenshot: false,
                audio_smoke: false,
                slice_smoke: false,
                failure_smoke: false,
            },
            runtime: Runtime::day(),
            input: PlatformInput::default(),
            pointer: None,
            camera,
            status,
            error_text,
            scene: None,
            pending: None,
            ready: true,
            failure: None,
            effect: None,
            music: None,
            music_entity: None,
            last_view: (800, 600, 1.0),
            screenshot_requested: false,
            screenshot_wait: None,
            loading_started: 0.0,
            paused_audio_position: None,
            effects: std::collections::BTreeMap::new(),
            feedback: String::new(),
        });
        world
    }

    fn batch(world: &mut World, points: &[(f32, f32)], key_code: KeyCode) {
        let window = world
            .query_filtered::<Entity, With<PrimaryWindow>>()
            .single(world)
            .unwrap();
        for (id, &(x, y)) in points.iter().enumerate() {
            world.write_message(WindowEvent::TouchInput(TouchInput {
                phase: TouchPhase::Started,
                position: Vec2::new(x, y),
                window,
                force: None,
                id: id as u64,
            }));
        }
        world.write_message(WindowEvent::KeyboardInput(KeyboardInput {
            key_code,
            logical_key: Key::Unidentified(NativeKey::Unidentified),
            state: ButtonState::Pressed,
            text: None,
            repeat: false,
            window,
        }));
        // 与引擎同帧更新按钮快照后进入 update 的入口一致。
        world.resource_mut::<ButtonInput<KeyCode>>().press(key_code);
        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_secs_f64(1.0 / 60.0));
        update(world);
    }

    #[test]
    fn 实际update入口先种植再重开得到空新局() {
        let mut world = harness();
        batch(&mut world, &[(180.0, 25.0), (120.0, 148.0)], KeyCode::KeyR);
        let state = world.resource::<AppState>();
        assert!(state.runtime.game.board().unwrap().plants().is_empty());
        assert_eq!(state.runtime.game.sun(), 150);
    }

    #[test]
    fn 实际update入口先种植再暂停仍完成种植() {
        let mut world = harness();
        world
            .resource_mut::<AppState>()
            .runtime
            .command(pvz_core::Command::SelectCard {
                kind: pvz_core::PlantKind::Sunflower,
            });
        batch(&mut world, &[(120.0, 148.0)], KeyCode::KeyP);
        let state = world.resource::<AppState>();
        assert!(state.runtime.paused);
        assert_eq!(state.runtime.game.board().unwrap().plants().len(), 1);
        assert_eq!(state.runtime.game.sun(), 100);
    }

    #[test]
    fn 实际update入口按_f切换明确失败场景() {
        let mut world = harness();
        batch(&mut world, &[], KeyCode::KeyF);
        let state = world.resource::<AppState>();
        assert_eq!(
            state.runtime.game.level().as_str(),
            "development-failure-slice"
        );
        assert_eq!(
            state.runtime.game.board().unwrap().status(),
            pvz_core::GameStatus::Running
        );
    }

    fn music_harness() -> World {
        let mut world = harness();
        world.resource_mut::<AppState>().music = Some(Handle::default());
        world
    }

    fn sunflower_vertices(world: &mut World) -> Vec<[f32; 3]> {
        use bevy::mesh::VertexAttributeValues;
        // 行深度加源轨道索引，直接找到真实 Scene 的底层网格组件。
        let handle = world
            .query::<(&Transform, &Mesh2d)>()
            .iter(world)
            .find(|(transform, _)| (transform.translation.z - 11.01).abs() < 0.001)
            .expect("向日葵底层网格实际存在")
            .1
            .0
            .clone();
        let mesh = world.resource::<Assets<Mesh>>().get(&handle).unwrap();
        match mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap() {
            VertexAttributeValues::Float32x3(vertices) => vertices.clone(),
            other => panic!("向日葵顶点格式异常：{other:?}"),
        }
    }

    #[test]
    fn 实际update向日葵消费表现时钟暂停保持相位恢复后改变网格() {
        let mut world = harness();
        let scene = render::sunflower_scene_fixture(&mut world);
        let mut runtime = Runtime::failure_day();
        runtime.command(pvz_core::Command::SelectCard {
            kind: pvz_core::PlantKind::Sunflower,
        });
        runtime.command(pvz_core::Command::Plant {
            cell: pvz_core::Cell { row: 1, column: 0 },
        });
        // 先推进种植，再独立提交暂停：同批末尾暂停按核心合同不推进 tick。
        runtime.advance(1.0 / 60.0);
        assert_eq!(runtime.game.board().unwrap().plants().len(), 1);
        runtime.command(pvz_core::Command::Pause);
        runtime.advance(1.0 / 60.0);
        assert!(runtime.paused);
        // 此相位与 0 以及模拟 tick 都不同：12fps 第二帧的底层 x=9。
        runtime.animation_seconds = 0.12;
        assert_eq!(runtime.game.clock().current_tick(), 1);
        let state = &mut *world.resource_mut::<AppState>();
        state.runtime = runtime;
        state.scene = Some(scene);
        let expected = vec![
            [-311.0, 104.0, 0.0],
            [-309.0, 104.0, 0.0],
            [-309.0, 102.0, 0.0],
            [-311.0, 102.0, 0.0],
        ];
        for _ in 0..2 {
            world
                .resource_mut::<Time<Real>>()
                .advance_by(Duration::from_secs_f64(0.01));
            update(&mut world);
            assert_eq!(world.resource::<AppState>().runtime.animation_seconds, 0.12);
            assert_eq!(sunflower_vertices(&mut world), expected);
        }
        world
            .resource_mut::<AppState>()
            .runtime
            .command(pvz_core::Command::Resume);
        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_secs_f64(0.01));
        update(&mut world);
        assert!(!world.resource::<AppState>().runtime.paused);
        assert_eq!(
            world.resource::<AppState>().runtime.animation_seconds,
            0.12,
            "恢复命令当帧不承接暂停耗时"
        );
        assert_eq!(sunflower_vertices(&mut world), expected);
        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_secs_f64(0.14));
        update(&mut world);
        assert!((world.resource::<AppState>().runtime.animation_seconds - 0.26).abs() < 1e-9);
        assert_eq!(
            sunflower_vertices(&mut world),
            [
                [-313.0, 104.0, 0.0],
                [-311.0, 104.0, 0.0],
                [-311.0, 102.0, 0.0],
                [-313.0, 102.0, 0.0]
            ]
        );
    }

    #[test]
    fn 实际update截图等待冻结运行时场景而真实时钟继续泵动() {
        let mut world = harness();
        let mut scene = render::sunflower_scene_fixture(&mut world);
        let mut runtime = Runtime::failure_day();
        runtime.command(pvz_core::Command::SelectCard {
            kind: pvz_core::PlantKind::Sunflower,
        });
        runtime.command(pvz_core::Command::Plant {
            cell: pvz_core::Cell { row: 1, column: 0 },
        });
        runtime.advance(1.0 / 60.0);
        scene.update(&mut world, &runtime.game, 0.12);
        let before_mesh = sunflower_vertices(&mut world);
        let before_digest = runtime.game.state_digest();
        let before_animation = runtime.animation_seconds;
        let before_frames = runtime.render_frames;
        {
            let state = &mut *world.resource_mut::<AppState>();
            state.options.smoke = Some(4);
            state.options.screenshot = true;
            state.screenshot_requested = true;
            state.screenshot_wait = Some(screenshot_wait::ScreenshotWait::new(Duration::ZERO));
            state.runtime = runtime;
            state.scene = Some(scene);
        }
        let before_real = world.resource::<Time<Real>>().elapsed();
        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_secs(1));
        update(&mut world);
        let state = world.resource::<AppState>();
        assert_eq!(state.runtime.game.state_digest(), before_digest);
        assert_eq!(state.runtime.animation_seconds, before_animation);
        assert_eq!(state.runtime.render_frames, before_frames);
        assert!(state.screenshot_wait.is_some(), "等待状态尚未完成");
        assert!(world.resource::<Time<Real>>().elapsed() > before_real);
        assert_eq!(sunflower_vertices(&mut world), before_mesh);

        let mut exits = world.resource::<Messages<AppExit>>().get_cursor();
        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_secs(29));
        update(&mut world);
        assert!(world.resource::<AppState>().screenshot_wait.is_some());
        let messages: Vec<_> = exits.read(world.resource::<Messages<AppExit>>()).collect();
        assert_eq!(messages.len(), 1, "30秒边界只交付一次退出");
        assert!(
            matches!(messages[0], AppExit::Error(_)),
            "等待超时必须传播错误退出"
        );
        update(&mut world);
        assert_eq!(
            exits.read(world.resource::<Messages<AppExit>>()).count(),
            0,
            "完成后的等待轮询不重复退出"
        );
    }

    fn audio_batch(world: &mut World, keys: &[KeyCode]) {
        let window = world
            .query_filtered::<Entity, With<PrimaryWindow>>()
            .single(world)
            .unwrap();
        for &key_code in keys {
            world.write_message(WindowEvent::KeyboardInput(KeyboardInput {
                key_code,
                logical_key: Key::Unidentified(NativeKey::Unidentified),
                state: ButtonState::Pressed,
                text: None,
                repeat: false,
                window,
            }));
        }
        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_secs_f64(1.0 / 60.0));
        update(world);
    }

    fn assert_music_request(world: &World, paused: bool) {
        let state = world.resource::<AppState>();
        let entity = state.music_entity.unwrap();
        assert!(world.get::<AudioPlayer>(entity).is_some());
        // 后端尚未创建 sink；真实播放请求必须保留正确的初始暂停状态。
        assert!(world.get::<AudioSink>(entity).is_none());
        assert_eq!(
            world.get::<PlaybackSettings>(entity).unwrap().paused,
            paused
        );
    }

    #[test]
    fn 实际update同批先暂停再音乐使用最终模拟状态() {
        let mut world = music_harness();
        audio_batch(&mut world, &[KeyCode::KeyP, KeyCode::KeyM]);
        assert!(world.resource::<AppState>().runtime.paused);
        assert_music_request(&world, true);
    }

    #[test]
    fn 实际update同批先音乐再暂停使用最终模拟状态() {
        let mut world = music_harness();
        audio_batch(&mut world, &[KeyCode::KeyM, KeyCode::KeyP]);
        assert!(world.resource::<AppState>().runtime.paused);
        assert_music_request(&world, true);
    }

    #[test]
    fn 实际update迟到sink前的音乐请求在暂停帧持续保持暂停() {
        let mut world = music_harness();
        audio_batch(&mut world, &[KeyCode::KeyM]);
        assert_music_request(&world, false);
        audio_batch(&mut world, &[KeyCode::KeyP]);
        assert_music_request(&world, true);
        for _ in 0..3 {
            audio_batch(&mut world, &[]);
            assert_music_request(&world, true);
        }
    }

    #[test]
    fn 实际update迟到sink前恢复和重开更新音乐请求() {
        for key in [KeyCode::KeyP, KeyCode::KeyR] {
            let mut world = music_harness();
            audio_batch(&mut world, &[KeyCode::KeyP]);
            audio_batch(&mut world, &[KeyCode::KeyM]);
            assert_music_request(&world, true);
            let entity = world.resource::<AppState>().music_entity.unwrap();
            audio_batch(&mut world, &[key]);
            assert!(!world.resource::<AppState>().runtime.paused);
            assert_eq!(world.resource::<AppState>().music_entity, Some(entity));
            assert_music_request(&world, false);
            audio_batch(&mut world, &[]);
            assert_music_request(&world, false);
        }
    }

    #[test]
    fn 实际update终局拒绝暂停后音乐仍遵循模拟状态() {
        for keys in [
            [KeyCode::KeyP, KeyCode::KeyM],
            [KeyCode::KeyM, KeyCode::KeyP],
        ] {
            let mut world = music_harness();
            let state = &mut *world.resource_mut::<AppState>();
            for _ in 0..1000 {
                if state.runtime.game.board().unwrap().status() != pvz_core::GameStatus::Running {
                    break;
                }
                state.runtime.advance(0.25);
            }
            assert_ne!(
                state.runtime.game.board().unwrap().status(),
                pvz_core::GameStatus::Running
            );
            state.runtime.take_events();
            audio_batch(&mut world, &keys);
            let state = world.resource::<AppState>();
            assert!(!state.runtime.paused);
            assert_eq!(state.feedback, "本局结束，请重开");
            assert_music_request(&world, false);
        }
    }

    #[test]
    fn 实际update纯probe暂停入口同步等待后端的音乐请求() {
        let mut world = music_harness();
        let state = &mut *world.resource_mut::<AppState>();
        state.options.probe = true;
        state.runtime = Runtime::default();
        audio_batch(&mut world, &[KeyCode::KeyP, KeyCode::KeyM]);
        assert!(world.resource::<AppState>().runtime.game.board().is_none());
        assert_music_request(&world, true);
        audio_batch(&mut world, &[KeyCode::KeyP]);
        assert_music_request(&world, false);
    }

    #[test]
    fn 失败冒烟期限前继续等待而期限未达失败时只错误退出一次() {
        let mut world = harness();
        world.resource_scope(|world, mut state: Mut<AppState>| {
            state.options.failure_smoke = true;
            state.options.smoke = Some(2);
            failure_script(&mut state);
            state.runtime.advance(0.25);
            smoke(world, &mut state);
            assert!(world.resource::<Messages<AppExit>>().is_empty());
            assert!(state.failure.is_none());
            state.runtime.advance(0.25);
            assert_eq!(
                state.runtime.game.board().unwrap().status(),
                pvz_core::GameStatus::Running
            );
            smoke(world, &mut state);
            smoke(world, &mut state);
            assert_eq!(
                state.failure.as_deref(),
                Some("失败冒烟未在指定帧数内观察到开发关卡失败")
            );
        });
        let mut reader = world.resource::<Messages<AppExit>>().get_cursor();
        let exits: Vec<_> = reader.read(world.resource::<Messages<AppExit>>()).collect();
        assert!(matches!(exits.as_slice(), [AppExit::Error(_)]));
    }

    #[test]
    fn 失败冒烟期限到达且已失败时只成功退出一次() {
        let mut world = harness();
        world.resource_scope(|world, mut state: Mut<AppState>| {
            state.options.failure_smoke = true;
            failure_script(&mut state);
            for _ in 0..1000 {
                if state.runtime.game.board().unwrap().status() == pvz_core::GameStatus::Lost {
                    break;
                }
                state.runtime.advance(0.25);
            }
            assert_eq!(
                state.runtime.game.board().unwrap().status(),
                pvz_core::GameStatus::Lost
            );
            state.options.smoke = Some(u32::try_from(state.runtime.render_frames).unwrap());
            smoke(world, &mut state);
            smoke(world, &mut state);
            assert!(state.failure.is_none());
        });
        let mut reader = world.resource::<Messages<AppExit>>().get_cursor();
        let exits: Vec<_> = reader.read(world.resource::<Messages<AppExit>>()).collect();
        assert!(matches!(exits.as_slice(), [AppExit::Success]));
    }

    #[test]
    fn 实际截图消费者无回调时最终非零退出且只退出一次() {
        let mut world = harness();
        world.resource_scope(|world, mut state: Mut<AppState>| {
            state.options.smoke = Some(2);
            state.options.screenshot = true;
            state.runtime.render_frames = 2;
            smoke(world, &mut state);
            assert!(world.resource::<Messages<AppExit>>().is_empty());
            world
                .resource_mut::<Time<Real>>()
                .advance_by(Duration::from_secs(31));
            smoke(world, &mut state);
            smoke(world, &mut state);
        });
        let mut reader = world.resource::<Messages<AppExit>>().get_cursor();
        let exits: Vec<_> = reader.read(world.resource::<Messages<AppExit>>()).collect();
        assert_eq!(exits.len(), 1);
        assert!(matches!(exits[0], AppExit::Error(_)));
    }
    fn screenshot_result(
        completion: Option<Result<(), String>>,
        failed: bool,
        elapsed: Duration,
    ) -> Vec<AppExit> {
        let mut world = harness();
        world.resource_scope(|world, mut state: Mut<AppState>| {
            state.options.smoke = Some(2);
            state.options.screenshot = true;
            state.runtime.render_frames = 2;
            state.failure = failed.then(|| "资源加载失败".to_owned());
            smoke(world, &mut state);
            world.resource_mut::<Time<Real>>().advance_by(elapsed);
            if let Some(result) = completion {
                state
                    .screenshot_wait
                    .as_mut()
                    .unwrap()
                    .complete(world.resource::<Time<Real>>().elapsed(), result);
            }
            smoke(world, &mut state);
            smoke(world, &mut state);
        });
        let mut reader = world.resource::<Messages<AppExit>>().get_cursor();
        reader
            .read(world.resource::<Messages<AppExit>>())
            .cloned()
            .collect()
    }

    #[test]
    fn 实际截图消费者成功保存正常退出() {
        assert!(matches!(
            screenshot_result(Some(Ok(())), false, Duration::from_secs(1)).as_slice(),
            [AppExit::Success]
        ));
    }

    #[test]
    fn 实际截图消费者保存失败非零退出() {
        assert!(matches!(
            screenshot_result(
                Some(Err("GPU 截图保存失败".to_owned())),
                false,
                Duration::from_secs(1)
            )
            .as_slice(),
            [AppExit::Error(_)]
        ));
    }

    #[test]
    fn 实际截图消费者资源先失败后成功回调仍非零退出() {
        assert!(matches!(
            screenshot_result(Some(Ok(())), true, Duration::from_secs(1)).as_slice(),
            [AppExit::Error(_)]
        ));
    }

    #[test]
    fn 实际截图消费者资源先失败且无回调到边界非零退出() {
        assert!(screenshot_result(None, true, Duration::from_millis(29999)).is_empty());
        assert!(matches!(
            screenshot_result(None, true, Duration::from_secs(30)).as_slice(),
            [AppExit::Error(_)]
        ));
    }

    #[test]
    fn 真实截图保存路径隔离并检查成功图像和不可写父目录() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("正常/截图.png");
        save_screenshot(&Image::default(), &path, false).unwrap();
        assert_eq!(image::open(path).unwrap().width(), 1);
        let blocker = temp.path().join("文件不是目录");
        std::fs::write(&blocker, b"fixture").unwrap();
        let error =
            save_screenshot(&Image::default(), &blocker.join("截图.png"), false).unwrap_err();
        assert!(error.contains("GPU 截图保存失败"));
        assert!(!blocker.join("截图.png").exists());
    }

    #[test]
    fn 探针截图缺少任一红蓝夹具必须拒绝保存() {
        for colors in [vec![], vec![[230, 25, 25, 255]], vec![[25, 153, 255, 255]]] {
            let temp = tempfile::tempdir().unwrap();
            let path = temp.path().join("探针.png");
            let image = probe_image(&colors);
            assert!(save_screenshot(&image, &path, true).is_err());
            assert!(!path.exists(), "不完整画面不能作为通过截图保存");
        }
    }

    #[test]
    fn 探针截图真实红蓝区域存在才成功且普通截图不受限制() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("探针.png");
        let image = probe_image(&[[230, 25, 25, 255], [25, 153, 255, 255]]);
        save_screenshot(&image, &path, true).unwrap();
        assert_eq!(image::open(&path).unwrap().width(), 100);
        save_screenshot(&Image::default(), &temp.path().join("普通.png"), false).unwrap();
    }

    fn probe_image(colors: &[[u8; 4]]) -> Image {
        let mut rgba = image::RgbaImage::from_pixel(100, 100, image::Rgba([8, 15, 26, 255]));
        for (index, color) in colors.iter().enumerate() {
            for x in 80 + index as u32 * 10..90 + index as u32 * 10 {
                for y in 70..80 {
                    rgba.put_pixel(x, y, image::Rgba(*color));
                }
            }
        }
        Image::from_dynamic(
            image::DynamicImage::ImageRgba8(rgba),
            true,
            bevy::asset::RenderAssetUsages::default(),
        )
    }
}
