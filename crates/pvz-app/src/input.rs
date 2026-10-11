use crate::viewport::{LogicalPoint, Viewport};
use std::collections::BTreeSet;

/// 平台输入完成 DPI 转换之后的逻辑事件。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputEvent {
    PointerDown { id: u64, point: LogicalPoint },
    PointerUp { id: u64, point: LogicalPoint },
    PointerMove { id: u64, point: LogicalPoint },
    PointerCancel { id: u64 },
    Pause,
    Resume,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PointerPhase {
    Down,
    Up,
    Move,
    Cancel,
}
/// 捕获状态按指针隔离；越界或平台取消会结束捕获，不留下拖动状态。
#[derive(Default)]
pub struct InputAdapter {
    captured: BTreeSet<u64>,
}
impl InputAdapter {
    /// 窗口坐标先乘一次 DPI，纯视口只处理 framebuffer 像素。
    pub fn pointer(
        &mut self,
        view: Viewport,
        dpi: f32,
        id: u64,
        phase: PointerPhase,
        window_point: LogicalPoint,
    ) -> Option<InputEvent> {
        let point = if dpi.is_finite() && dpi > 0.0 {
            view.to_logical(LogicalPoint {
                x: window_point.x * dpi,
                y: window_point.y * dpi,
            })
        } else {
            None
        };
        match (phase, point) {
            (PointerPhase::Down, Some(point)) => {
                self.captured.insert(id);
                Some(InputEvent::PointerDown { id, point })
            }
            (PointerPhase::Move, Some(point)) => Some(InputEvent::PointerMove { id, point }),
            (PointerPhase::Up, Some(point)) if self.captured.remove(&id) => {
                Some(InputEvent::PointerUp { id, point })
            }
            _ if self.captured.remove(&id) => Some(InputEvent::PointerCancel { id }),
            _ => None,
        }
    }
    pub fn cancel_all(&mut self) -> Vec<InputEvent> {
        std::mem::take(&mut self.captured)
            .into_iter()
            .map(|id| InputEvent::PointerCancel { id })
            .collect()
    }
}
