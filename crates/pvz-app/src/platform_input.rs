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
    view: Option<(Viewport, f32)>,
    focused: bool,
}

impl Default for PlatformInput {
    fn default() -> Self {
        Self {
            reader: MessageCursor::default(),
            input: InputAdapter::default(),
            window: None,
            cursor: None,
            view: None,
            focused: true,
        }
    }
}

impl PlatformInput {
    /// Bevy 的位置为窗口逻辑像素；DPI 仅交给 InputAdapter 转换一次。
    pub fn read(
        &mut self,
        window: Entity,
        view: Viewport,
        dpi: f32,
        messages: &Messages<WindowEvent>,
    ) -> Vec<InputEvent> {
        let mut events = Vec::new();
        if self.window != Some(window) {
            events.extend(self.input.cancel_all());
            self.window = Some(window);
            self.cursor = None;
            self.view = None;
            self.focused = true;
        }
        // 有序尺寸消息之前仍用上一批的映射，不能让帧末快照提前取消松开。
        let mut preview = self.reader.clone();
        let ordered_view_change = preview.read(messages).any(|event| match event {
            WindowEvent::WindowResized(event) => event.window == window,
            WindowEvent::WindowScaleFactorChanged(event) => event.window == window,
            _ => false,
        });
        let (mut batch_view, mut batch_dpi) = if ordered_view_change {
            self.view.unwrap_or((view, dpi))
        } else {
            (view, dpi)
        };
        if !ordered_view_change && self.view.is_some_and(|previous| previous != (view, dpi)) {
            events.extend(self.input.cancel_all());
        }
        self.view = Some((view, dpi));
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
                    (batch_view, batch_dpi) = (view, dpi);
                    None
                }
                WindowEvent::WindowScaleFactorChanged(event) if event.window == window => {
                    events.extend(self.input.cancel_all());
                    (batch_view, batch_dpi) = (view, dpi);
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
                    batch_dpi,
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
        events
    }
}
