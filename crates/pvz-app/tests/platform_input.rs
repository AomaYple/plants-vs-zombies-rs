use bevy::{
    input::{
        ButtonState, InputPlugin,
        mouse::MouseButtonInput,
        touch::{TouchInput, TouchPhase},
    },
    prelude::*,
    window::{
        CursorLeft, CursorMoved, WindowEvent, WindowFocused, WindowResized,
        WindowScaleFactorChanged,
    },
};
use pvz_app::{
    input::InputEvent,
    platform_input::PlatformInput,
    viewport::{LogicalPoint, Viewport},
};

struct Harness {
    app: App,
    input: PlatformInput,
    window: Entity,
    view: Viewport,
    dpi: f32,
}

impl Harness {
    fn new() -> Self {
        let mut app = App::new();
        app.add_plugins(InputPlugin).add_message::<WindowEvent>();
        let window = app.world_mut().spawn_empty().id();
        Self {
            app,
            input: PlatformInput::default(),
            window,
            view: Viewport::fit((800.0, 600.0), (800.0, 600.0)),
            dpi: 1.0,
        }
    }

    fn batch(&mut self, events: impl IntoIterator<Item = WindowEvent>) -> Vec<InputEvent> {
        for event in events {
            match &event {
                WindowEvent::TouchInput(touch) => {
                    self.app.world_mut().write_message(*touch);
                }
                WindowEvent::MouseButtonInput(mouse) => {
                    self.app.world_mut().write_message(*mouse);
                }
                _ => {}
            }
            self.app.world_mut().write_message(event);
        }
        self.app.update();
        self.input.read(
            self.window,
            self.view,
            self.dpi,
            self.app.world().resource::<Messages<WindowEvent>>(),
        )
    }

    fn touch(&self, phase: TouchPhase) -> WindowEvent {
        touch(self.window, 7, phase, Vec2::new(100.0, 200.0))
    }

    fn mouse(&self, state: ButtonState) -> WindowEvent {
        WindowEvent::MouseButtonInput(MouseButtonInput {
            button: MouseButton::Left,
            state,
            window: self.window,
        })
    }
}

fn point() -> LogicalPoint {
    LogicalPoint { x: 100.0, y: 200.0 }
}

#[test]
fn 同批触摸开始结束保留完整点击() {
    let mut harness = Harness::new();
    let events = [
        harness.touch(TouchPhase::Started),
        harness.touch(TouchPhase::Ended),
    ];
    assert_eq!(
        harness.batch(events),
        [
            InputEvent::PointerDown {
                id: 7,
                point: point()
            },
            InputEvent::PointerUp {
                id: 7,
                point: point()
            }
        ]
    );
}

#[test]
fn 同批触摸开始取消保留完整阶段() {
    let mut harness = Harness::new();
    let events = [
        harness.touch(TouchPhase::Started),
        harness.touch(TouchPhase::Canceled),
    ];
    assert_eq!(
        harness.batch(events),
        [
            InputEvent::PointerDown {
                id: 7,
                point: point()
            },
            InputEvent::PointerCancel { id: 7 }
        ]
    );
}

#[test]
fn 同批鼠标按下松开保留完整点击() {
    let mut harness = Harness::new();
    let movement = WindowEvent::CursorMoved(CursorMoved {
        window: harness.window,
        position: Vec2::new(100.0, 200.0),
        delta: None,
    });
    harness.batch([movement]);
    let events = [
        harness.mouse(ButtonState::Pressed),
        harness.mouse(ButtonState::Released),
    ];
    assert_eq!(
        harness.batch(events),
        [
            InputEvent::PointerDown {
                id: u64::MAX,
                point: point()
            },
            InputEvent::PointerUp {
                id: u64::MAX,
                point: point()
            }
        ]
    );
}

fn touch(window: Entity, id: u64, phase: TouchPhase, position: Vec2) -> WindowEvent {
    WindowEvent::TouchInput(TouchInput {
        phase,
        position,
        window,
        force: None,
        id,
    })
}

fn cursor(window: Entity, position: Vec2) -> WindowEvent {
    WindowEvent::CursorMoved(CursorMoved {
        window,
        position,
        delta: None,
    })
}

#[test]
fn 多指针鼠标交错与同批重复点击保持原始顺序() {
    let mut harness = Harness::new();
    let window = harness.window;
    let position = Vec2::new(100.0, 200.0);
    harness.batch([cursor(window, position)]);
    let events = [
        touch(window, 1, TouchPhase::Started, position),
        harness.mouse(ButtonState::Pressed),
        touch(window, 2, TouchPhase::Started, position),
        touch(window, 1, TouchPhase::Ended, position),
        harness.mouse(ButtonState::Released),
        touch(window, 2, TouchPhase::Canceled, position),
        harness.mouse(ButtonState::Pressed),
        harness.mouse(ButtonState::Released),
    ];
    assert_eq!(
        harness.batch(events),
        [
            InputEvent::PointerDown {
                id: 1,
                point: point()
            },
            InputEvent::PointerDown {
                id: u64::MAX,
                point: point()
            },
            InputEvent::PointerDown {
                id: 2,
                point: point()
            },
            InputEvent::PointerUp {
                id: 1,
                point: point()
            },
            InputEvent::PointerUp {
                id: u64::MAX,
                point: point()
            },
            InputEvent::PointerCancel { id: 2 },
            InputEvent::PointerDown {
                id: u64::MAX,
                point: point()
            },
            InputEvent::PointerUp {
                id: u64::MAX,
                point: point()
            },
        ]
    );
    assert!(harness.batch([]).is_empty());
    let releases = [
        touch(window, 1, TouchPhase::Ended, position),
        harness.mouse(ButtonState::Released),
    ];
    assert!(harness.batch(releases).is_empty());
}

#[test]
fn 其他窗口相同指针编号与取消消息不影响主窗口() {
    let mut harness = Harness::new();
    let other = harness.app.world_mut().spawn_empty().id();
    let position = Vec2::new(100.0, 200.0);
    let events = [
        harness.touch(TouchPhase::Started),
        touch(other, 7, TouchPhase::Started, position),
        cursor(other, position),
        WindowEvent::MouseButtonInput(MouseButtonInput {
            window: other,
            button: MouseButton::Left,
            state: ButtonState::Pressed,
        }),
        WindowEvent::CursorLeft(CursorLeft { window: other }),
        WindowEvent::WindowFocused(WindowFocused {
            window: other,
            focused: false,
        }),
        WindowEvent::WindowResized(WindowResized {
            window: other,
            width: 320.0,
            height: 240.0,
        }),
        WindowEvent::WindowScaleFactorChanged(WindowScaleFactorChanged {
            window: other,
            scale_factor: 2.0,
        }),
        touch(other, 7, TouchPhase::Canceled, position),
        harness.touch(TouchPhase::Ended),
    ];
    assert_eq!(
        harness.batch(events),
        [
            InputEvent::PointerDown {
                id: 7,
                point: point()
            },
            InputEvent::PointerUp {
                id: 7,
                point: point()
            }
        ]
    );
}

#[test]
fn 窗口逻辑像素只应用一次dpi并映射留边() {
    let mut harness = Harness::new();
    harness.view = Viewport::fit((1920.0, 1080.0), (800.0, 600.0));
    harness.dpi = 2.0;
    let window = harness.window;
    let position = Vec2::new(480.0, 270.0);
    let events = [
        cursor(window, position),
        harness.mouse(ButtonState::Pressed),
        harness.mouse(ButtonState::Released),
        touch(window, 3, TouchPhase::Started, position),
        touch(window, 3, TouchPhase::Ended, position),
    ];
    let center = LogicalPoint { x: 400.0, y: 300.0 };
    assert_eq!(
        harness.batch(events),
        [
            InputEvent::PointerMove {
                id: u64::MAX,
                point: center
            },
            InputEvent::PointerDown {
                id: u64::MAX,
                point: center
            },
            InputEvent::PointerUp {
                id: u64::MAX,
                point: center
            },
            InputEvent::PointerDown {
                id: 3,
                point: center
            },
            InputEvent::PointerUp {
                id: 3,
                point: center
            },
        ]
    );
}

#[test]
fn 越界与离窗分别结束捕获且不残留松开() {
    let mut harness = Harness::new();
    let window = harness.window;
    let events = [
        harness.touch(TouchPhase::Started),
        touch(window, 7, TouchPhase::Moved, Vec2::new(-1.0, 200.0)),
        harness.touch(TouchPhase::Ended),
    ];
    assert_eq!(
        harness.batch(events),
        [
            InputEvent::PointerDown {
                id: 7,
                point: point()
            },
            InputEvent::PointerCancel { id: 7 }
        ]
    );
    harness.batch([cursor(window, Vec2::new(100.0, 200.0))]);
    let events = [
        harness.mouse(ButtonState::Pressed),
        WindowEvent::CursorLeft(CursorLeft { window }),
        harness.mouse(ButtonState::Released),
    ];
    assert_eq!(
        harness.batch(events),
        [
            InputEvent::PointerDown {
                id: u64::MAX,
                point: point()
            },
            InputEvent::PointerCancel { id: u64::MAX }
        ]
    );
    let outside = [
        touch(window, 7, TouchPhase::Started, Vec2::new(800.0, 200.0)),
        harness.touch(TouchPhase::Ended),
    ];
    assert!(harness.batch(outside).is_empty());
}

#[test]
fn 同批失焦按消息位置取消并在重新聚焦后接受输入() {
    let mut harness = Harness::new();
    let window = harness.window;
    let events = [
        harness.touch(TouchPhase::Started),
        WindowEvent::WindowFocused(WindowFocused {
            window,
            focused: false,
        }),
        harness.touch(TouchPhase::Ended),
        harness.touch(TouchPhase::Started),
        WindowEvent::WindowFocused(WindowFocused {
            window,
            focused: true,
        }),
        harness.touch(TouchPhase::Started),
        harness.touch(TouchPhase::Ended),
    ];
    assert_eq!(
        harness.batch(events),
        [
            InputEvent::PointerDown {
                id: 7,
                point: point()
            },
            InputEvent::PointerCancel { id: 7 },
            InputEvent::PointerDown {
                id: 7,
                point: point()
            },
            InputEvent::PointerUp {
                id: 7,
                point: point()
            },
        ]
    );
}

#[test]
fn 同批视口消息与跨帧视口变化都结束捕获() {
    let mut harness = Harness::new();
    let window = harness.window;
    for change in [
        WindowEvent::WindowResized(WindowResized {
            window,
            width: 1000.0,
            height: 750.0,
        }),
        WindowEvent::WindowScaleFactorChanged(WindowScaleFactorChanged {
            window,
            scale_factor: 2.0,
        }),
    ] {
        let events = [
            harness.touch(TouchPhase::Started),
            change,
            harness.touch(TouchPhase::Ended),
        ];
        assert_eq!(
            harness.batch(events),
            [
                InputEvent::PointerDown {
                    id: 7,
                    point: point()
                },
                InputEvent::PointerCancel { id: 7 }
            ]
        );
    }
    let start = harness.touch(TouchPhase::Started);
    harness.batch([start]);
    harness.view = Viewport::fit((1000.0, 750.0), (800.0, 600.0));
    assert_eq!(harness.batch([]), [InputEvent::PointerCancel { id: 7 }]);
    let end = harness.touch(TouchPhase::Ended);
    assert!(harness.batch([end]).is_empty());
}

#[test]
fn 指定窗口切换不会继承旧窗口捕获() {
    let mut harness = Harness::new();
    let start = harness.touch(TouchPhase::Started);
    harness.batch([start]);
    harness.window = harness.app.world_mut().spawn_empty().id();
    let end = harness.touch(TouchPhase::Ended);
    assert_eq!(harness.batch([end]), [InputEvent::PointerCancel { id: 7 }]);
}

#[test]
fn 帧末视口快照不提前吞掉尺寸消息前的松开() {
    let mut harness = Harness::new();
    let window = harness.window;
    let start = harness.touch(TouchPhase::Started);
    harness.batch([start]);
    harness.view = Viewport::fit((1000.0, 750.0), (800.0, 600.0));
    let events = [
        harness.touch(TouchPhase::Ended),
        WindowEvent::WindowResized(WindowResized {
            window,
            width: 1000.0,
            height: 750.0,
        }),
        touch(window, 7, TouchPhase::Started, Vec2::new(125.0, 250.0)),
        touch(window, 7, TouchPhase::Ended, Vec2::new(125.0, 250.0)),
    ];
    assert_eq!(
        harness.batch(events),
        [
            InputEvent::PointerUp {
                id: 7,
                point: point()
            },
            InputEvent::PointerDown {
                id: 7,
                point: point()
            },
            InputEvent::PointerUp {
                id: 7,
                point: point()
            },
        ]
    );
}

#[test]
fn 重新聚焦后无光标移动仍能配对鼠标点击() {
    let mut harness = Harness::new();
    let window = harness.window;
    harness.batch([cursor(window, Vec2::new(100.0, 200.0))]);
    let events = [
        harness.mouse(ButtonState::Pressed),
        WindowEvent::WindowFocused(WindowFocused {
            window,
            focused: false,
        }),
        harness.mouse(ButtonState::Released),
        WindowEvent::WindowFocused(WindowFocused {
            window,
            focused: true,
        }),
        harness.mouse(ButtonState::Pressed),
        harness.mouse(ButtonState::Released),
    ];
    assert_eq!(
        harness.batch(events),
        [
            InputEvent::PointerDown {
                id: u64::MAX,
                point: point()
            },
            InputEvent::PointerCancel { id: u64::MAX },
            InputEvent::PointerDown {
                id: u64::MAX,
                point: point()
            },
            InputEvent::PointerUp {
                id: u64::MAX,
                point: point()
            },
        ]
    );
}

#[test]
fn 鼠标按钮使用消息位置处的光标而非帧末坐标() {
    let mut harness = Harness::new();
    let window = harness.window;
    harness.batch([cursor(window, Vec2::new(100.0, 200.0))]);
    let moved = LogicalPoint { x: 300.0, y: 400.0 };
    let events = [
        harness.mouse(ButtonState::Pressed),
        cursor(window, Vec2::new(moved.x, moved.y)),
        harness.mouse(ButtonState::Released),
    ];
    assert_eq!(
        harness.batch(events),
        [
            InputEvent::PointerDown {
                id: u64::MAX,
                point: point()
            },
            InputEvent::PointerMove {
                id: u64::MAX,
                point: moved
            },
            InputEvent::PointerUp {
                id: u64::MAX,
                point: moved
            },
        ]
    );
}
