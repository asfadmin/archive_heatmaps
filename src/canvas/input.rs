use std::collections::HashSet;

use winit::dpi::PhysicalPosition;
use winit::event::ElementState;
use winit::event::Modifiers;
use winit::event::MouseButton;
use winit::event::MouseScrollDelta;
use winit::event::WindowEvent;
use winit::keyboard::Key;

#[derive(Default, Clone)]
pub struct InputState {
    mouse_buttons: HashSet<MouseButton>,
    keys: HashSet<Key>,
    pub modifiers: Modifiers,
    pub cursor_position: PhysicalPosition<f64>,
    mouse_scroll_delta: f64,
    mouse_drag_delta: PhysicalPosition<f64>,
}

impl InputState {
    // Returns the amount of change in scroll delta since last function call
    pub fn consume_scroll_delta(&mut self) -> f64 {
        let delta = self.mouse_scroll_delta;
        self.mouse_scroll_delta = 0.0;
        delta
    }

    // Returns the amount of change in cursor position since last function call
    pub fn consume_drag_delta(&mut self) -> PhysicalPosition<f64> {
        let delta = self.mouse_drag_delta;
        self.mouse_drag_delta = PhysicalPosition::new(0.0, 0.0);
        delta
    }

    pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        self.mouse_buttons.contains(&button)
    }

    // Performs the specified action for the given window event
    pub fn eat_event(&mut self, event: WindowEvent) {
        use WindowEvent::*;

        match event {
            // Update drag delta based on the change in cursor position
            CursorMoved {
                device_id: _,
                position,
            } => {
                if self.is_mouse_button_pressed(MouseButton::Left) {
                    self.mouse_drag_delta.x += position.x - self.cursor_position.x;
                    self.mouse_drag_delta.y += position.y - self.cursor_position.y;
                }
                self.cursor_position = position;
            }

            // Add/remove a mouse button press from the list of currently pressed buttons
            MouseInput {
                device_id: _,
                state,
                button,
            } => {
                use ElementState::*;

                match state {
                    Pressed => {
                        self.mouse_buttons.insert(button);
                    }

                    Released => {
                        self.mouse_buttons.remove(&button);
                    }
                }
            }

            // Track the change in the mouse wheel scroll
            MouseWheel {
                device_id: _,
                delta,
                phase: _,
            } => {
                use MouseScrollDelta::*;

                match delta {
                    LineDelta(_x, y) => {
                        self.mouse_scroll_delta += (y * 10.0) as f64;
                    }

                    PixelDelta(position) => {
                        self.mouse_scroll_delta += position.y;
                    }
                }
            }

            // Add/Remove the keyboard key from the list of pressed keys
            KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } => {
                use ElementState::*;

                match event.state {
                    Pressed => {
                        self.keys.insert(event.logical_key);
                    }

                    Released => {
                        self.keys.remove(&event.logical_key);
                    }
                }
            }

            ModifiersChanged(modifiers) => {
                self.modifiers = modifiers;
            }

            _ => {}
        }
    }
}
