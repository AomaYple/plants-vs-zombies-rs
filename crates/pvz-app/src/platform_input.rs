use bevy::{
    ecs::message::MessageCursor,
    input::{ButtonState, touch::TouchPhase},
    prelude::*,
    window::WindowEvent,
};

use crate::{
    input::{InputAdapter, InputEvent, PointerPhase},
    viewport::{LogicalPoint, Viewport},
};

/// 仅消费指定窗口的原始有序消息，不从帧末按钮或触摸快照重建阶段。
pub struct PlatformInput {
    reader: MessageCursor<WindowEvent>,
    input: InputAdapter,
    window: Option<Entity>,
    cursor: Option<Vec2>,
    geometry: Option<WindowGeometry>,
    focused: bool,
}

impl Default for PlatformInput {
    fn default() -> Self {
        Self {
            reader: MessageCursor::default(),
            input: InputAdapter::default(),
            window: None,
            cursor: None,
            geometry: None,
            focused: true,
        }
    }
}

impl PlatformInput {
    /// Bevy 的位置为窗口逻辑像素；DPI 仅交给 InputAdapter 转换一次。
    pub fn read(
        &mut self,
        window: Entity,
        physical_size: (u32, u32),
        dpi: f32,
        logical_size: (f32, f32),
        messages: &Messages<WindowEvent>,
    ) -> Vec<InputEvent> {
        let mut events = Vec::new();
        if self.window != Some(window) {
            events.extend(self.input.cancel_all());
            self.window = Some(window);
            self.cursor = None;
            self.geometry = None;
            self.focused = true;
        }
        let snapshot = WindowGeometry { physical_size, dpi };
        // 尺寸和 DPI 分别由对应消息推进；没有对应消息的字段才同步快照。
        let mut preview = self.reader.clone();
        let (resized, dpi_changed) = preview.read(messages).fold(
            (false, false),
            |(resized, dpi_changed), event| match event {
                WindowEvent::WindowResized(event) if event.window == window => (true, dpi_changed),
                WindowEvent::WindowScaleFactorChanged(event) if event.window == window => {
                    (resized, true)
                }
                _ => (resized, dpi_changed),
            },
        );
        let mut geometry = self.geometry.unwrap_or(snapshot);
        let synchronized = WindowGeometry {
            physical_size: if resized {
                geometry.physical_size
            } else {
                snapshot.physical_size
            },
            dpi: if dpi_changed {
                geometry.dpi
            } else {
                snapshot.dpi
            },
        };
        if geometry != synchronized {
            events.extend(self.input.cancel_all());
            rescale_cursor(&mut self.cursor, geometry.dpi, synchronized.dpi);
            geometry = synchronized;
        }
        let mut batch_view = geometry.viewport(logical_size);
        for event in self.reader.read(messages) {
            let pointer = match event {
                WindowEvent::WindowFocused(event) if event.window == window => {
                    self.focused = event.focused;
                    if !event.focused {
                        events.extend(self.input.cancel_all());
                    }
                    None
                }
                WindowEvent::WindowResized(event) if event.window == window => {
                    events.extend(self.input.cancel_all());
                    // Bevy resize 消息是逻辑窗口尺寸；恢复该阶段的物理像素。
                    geometry.physical_size = (
                        (event.width * geometry.dpi).round() as u32,
                        (event.height * geometry.dpi).round() as u32,
                    );
                    batch_view = geometry.viewport(logical_size);
                    None
                }
                WindowEvent::WindowScaleFactorChanged(event) if event.window == window => {
                    events.extend(self.input.cancel_all());
                    // Bevy 只更新有效比例；物理尺寸保持，直到后续 resize。
                    let new_dpi = event.scale_factor as f32;
                    rescale_cursor(&mut self.cursor, geometry.dpi, new_dpi);
                    geometry.dpi = new_dpi;
                    batch_view = geometry.viewport(logical_size);
                    None
                }
                WindowEvent::WindowDestroyed(event) if event.window == window => {
                    events.extend(self.input.cancel_all());
                    self.cursor = None;
                    self.focused = false;
                    None
                }
                WindowEvent::CursorLeft(event) if event.window == window => {
                    self.cursor = None;
                    Some((u64::MAX, PointerPhase::Cancel, Vec2::NAN))
                }
                WindowEvent::CursorMoved(event) if event.window == window => {
                    self.cursor = Some(event.position);
                    self.focused
                        .then_some((u64::MAX, PointerPhase::Move, event.position))
                }
                WindowEvent::MouseButtonInput(event)
                    if event.window == window
                        && event.button == MouseButton::Left
                        && self.focused =>
                {
                    let phase = match event.state {
                        ButtonState::Pressed => PointerPhase::Down,
                        ButtonState::Released => PointerPhase::Up,
                    };
                    self.cursor.map(|position| (u64::MAX, phase, position))
                }
                WindowEvent::TouchInput(event) if event.window == window && self.focused => {
                    let phase = match event.phase {
                        TouchPhase::Started => PointerPhase::Down,
                        TouchPhase::Moved => PointerPhase::Move,
                        TouchPhase::Ended => PointerPhase::Up,
                        TouchPhase::Canceled => PointerPhase::Cancel,
                    };
                    Some((event.id, phase, event.position))
                }
                _ => None,
            };
            if let Some((id, phase, position)) = pointer
                && let Some(event) = self.input.pointer(
                    batch_view,
                    geometry.dpi,
                    id,
                    phase,
                    LogicalPoint {
                        x: position.x,
                        y: position.y,
                    },
                )
            {
                events.push(event);
            }
        }
        self.geometry = Some(geometry);
        events
    }
}

#[derive(Clone, Copy, PartialEq)]
struct WindowGeometry {
    physical_size: (u32, u32),
    dpi: f32,
}

impl WindowGeometry {
    fn viewport(self, logical_size: (f32, f32)) -> Viewport {
        Viewport::fit(
            (self.physical_size.0 as f32, self.physical_size.1 as f32),
            logical_size,
        )
    }
}

// DPI 改变不会移动 Bevy 的物理光标；缓存的逻辑点需采用新比例。
fn rescale_cursor(cursor: &mut Option<Vec2>, old_dpi: f32, new_dpi: f32) {
    *cursor = cursor.and_then(|point| {
        (old_dpi.is_finite() && old_dpi > 0.0 && new_dpi.is_finite() && new_dpi > 0.0)
            .then_some(point * old_dpi / new_dpi)
    });
}
