use pvz_app::viewport::{LogicalPoint, Viewport};

#[test]
fn 四比三等比填满() {
    let view = Viewport::fit((800.0, 600.0), (800.0, 600.0));
    assert_eq!(view.scale, 1.0);
    assert_eq!(view.size, (800.0, 600.0));
    assert_eq!(
        view.to_logical(LogicalPoint { x: 400.0, y: 300.0 }),
        Some(LogicalPoint { x: 400.0, y: 300.0 })
    );
}
#[test]
fn 十六比九留边并拒绝边外输入() {
    let view = Viewport::fit((1920.0, 1080.0), (800.0, 600.0));
    assert_eq!(view.offset, LogicalPoint { x: 240.0, y: 0.0 });
    assert_eq!(view.scale, 1.8);
    assert_eq!(view.to_logical(LogicalPoint { x: 239.0, y: 100.0 }), None);
    assert_eq!(
        view.to_logical(LogicalPoint {
            x: 1680.0,
            y: 100.0
        }),
        None
    );
    assert_eq!(
        view.to_logical(LogicalPoint { x: 960.0, y: 540.0 }),
        Some(LogicalPoint { x: 400.0, y: 300.0 })
    );
}
#[test]
fn 高dpi只改变物理比例() {
    let view = Viewport::fit((3200.0, 2400.0), (800.0, 600.0));
    assert_eq!(view.scale, 4.0);
    assert_eq!(
        view.to_logical(LogicalPoint {
            x: 2000.0,
            y: 1200.0
        }),
        Some(LogicalPoint { x: 500.0, y: 300.0 })
    );
}
#[test]
fn 无效尺寸和非有限坐标不映射() {
    for size in [
        (0.0, 600.0),
        (-1.0, 600.0),
        (f32::NAN, 600.0),
        (800.0, f32::INFINITY),
    ] {
        assert_eq!(
            Viewport::fit(size, (800.0, 600.0)).to_logical(LogicalPoint { x: 0.0, y: 0.0 }),
            None
        );
    }
    let view = Viewport::fit((800.0, 600.0), (800.0, 600.0));
    assert_eq!(
        view.to_logical(LogicalPoint {
            x: f32::NAN,
            y: 10.0
        }),
        None
    );
}
