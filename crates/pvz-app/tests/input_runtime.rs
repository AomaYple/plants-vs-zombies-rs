use pvz_app::{
    input::{InputAdapter, InputEvent, PointerPhase},
    runtime::Runtime,
    viewport::{LogicalPoint, Viewport},
};

#[test]
fn dpi窗口点变换与多指针取消互不干扰() {
    let view = Viewport::fit((1920.0, 1080.0), (800.0, 600.0));
    let mut input = InputAdapter::default();
    let point = LogicalPoint { x: 480.0, y: 270.0 };
    assert_eq!(
        input.pointer(view, 2.0, 1, PointerPhase::Down, point),
        Some(InputEvent::PointerDown {
            id: 1,
            point: LogicalPoint { x: 400.0, y: 300.0 }
        })
    );
    assert!(
        input
            .pointer(view, 2.0, 2, PointerPhase::Down, point)
            .is_some()
    );
    assert_eq!(
        input.pointer(
            view,
            2.0,
            1,
            PointerPhase::Move,
            LogicalPoint { x: 0.0, y: 0.0 }
        ),
        Some(InputEvent::PointerCancel { id: 1 })
    );
    assert_eq!(input.cancel_all(), [InputEvent::PointerCancel { id: 2 }]);
    assert_eq!(input.pointer(view, 2.0, 2, PointerPhase::Up, point), None);
}
#[test]
fn 松开只配对有效按下并明确处理平台取消() {
    let view = Viewport::fit((800.0, 600.0), (800.0, 600.0));
    let mut input = InputAdapter::default();
    let point = LogicalPoint { x: 100.0, y: 200.0 };
    assert_eq!(input.pointer(view, 1.0, 8, PointerPhase::Up, point), None);
    input.pointer(view, 1.0, 8, PointerPhase::Down, point);
    assert_eq!(
        input.pointer(view, 1.0, 8, PointerPhase::Cancel, point),
        Some(InputEvent::PointerCancel { id: 8 })
    );
    assert_eq!(input.pointer(view, 1.0, 8, PointerPhase::Up, point), None);
    assert_eq!(
        input.pointer(view, f32::INFINITY, 9, PointerPhase::Down, point),
        None
    );
}
#[test]
fn 暂停冻结模拟与动画渲染仍推进恢复不补暂停时间() {
    let mut runtime = Runtime::default();
    runtime.input(InputEvent::PointerDown {
        id: 1,
        point: LogicalPoint { x: 1.0, y: 2.0 },
    });
    runtime.advance(1.0 / 60.0);
    assert_eq!(runtime.game.sun(), 51);
    let tick = runtime.game.clock().current_tick();
    let animation = runtime.animation_seconds;
    runtime.input(InputEvent::Pause);
    runtime.input(InputEvent::PointerDown {
        id: 1,
        point: LogicalPoint { x: 1.0, y: 2.0 },
    });
    runtime.advance(5.0);
    assert_eq!(runtime.game.clock().current_tick(), tick);
    assert_eq!(runtime.animation_seconds, animation);
    assert_eq!(runtime.render_frames, 2);
    runtime.input(InputEvent::Resume);
    runtime.advance(1.0 / 60.0);
    assert_eq!(runtime.game.clock().current_tick(), tick + 1);
    assert_eq!(runtime.game.sun(), 51);
}
#[test]
fn 无效耗时不污染时钟长帧有明确开发上限() {
    let mut runtime = Runtime::default();
    for duration in [f64::NAN, f64::INFINITY, -1.0, 0.0] {
        runtime.advance(duration);
    }
    assert_eq!(runtime.animation_seconds, 0.0);
    runtime.advance(100.0);
    assert_eq!(runtime.animation_seconds, 0.25);
    assert_eq!(runtime.game.clock().current_tick(), 15);
}
