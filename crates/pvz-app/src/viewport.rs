/// 逻辑或物理坐标中的二维点。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LogicalPoint {
    pub x: f32,
    pub y: f32,
}
/// 居中等比视口；尺寸与偏移均使用 framebuffer 物理像素。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    pub offset: LogicalPoint,
    pub size: (f32, f32),
    pub scale: f32,
}
impl Viewport {
    pub fn fit(window_size: (f32, f32), logical_size: (f32, f32)) -> Self {
        let invalid = Self {
            offset: LogicalPoint { x: 0.0, y: 0.0 },
            size: (0.0, 0.0),
            scale: 0.0,
        };
        if [window_size.0, window_size.1, logical_size.0, logical_size.1]
            .iter()
            .any(|n| !n.is_finite() || *n <= 0.0)
        {
            return invalid;
        }
        let scale = (window_size.0 / logical_size.0).min(window_size.1 / logical_size.1);
        if !scale.is_finite() || scale <= 0.0 {
            return invalid;
        }
        let size = (logical_size.0 * scale, logical_size.1 * scale);
        Self {
            offset: LogicalPoint {
                x: (window_size.0 - size.0) / 2.0,
                y: (window_size.1 - size.1) / 2.0,
            },
            size,
            scale,
        }
    }
    pub fn to_logical(self, physical_point: LogicalPoint) -> Option<LogicalPoint> {
        if self.scale <= 0.0 || !physical_point.x.is_finite() || !physical_point.y.is_finite() {
            return None;
        }
        let x = physical_point.x - self.offset.x;
        let y = physical_point.y - self.offset.y;
        if x < 0.0 || y < 0.0 || x >= self.size.0 || y >= self.size.1 {
            return None;
        }
        Some(LogicalPoint {
            x: x / self.scale,
            y: y / self.scale,
        })
    }
}
