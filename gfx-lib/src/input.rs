use crate::Point2d;
use ::winit::{event::ElementState, event::KeyboardInput, dpi::PhysicalPosition};
use std::collections::HashMap;

pub use ::winit::event::{MouseButton, VirtualKeyCode};

#[derive(Default, Clone)]
pub struct InputState {
    current_keys: HashMap<VirtualKeyCode, bool>,
    pressed_keys: HashMap<VirtualKeyCode, bool>,
    released_keys: HashMap<VirtualKeyCode, bool>,
    current_mouse_buttons: HashMap<MouseButton, bool>,
    pressed_mouse_buttons: HashMap<MouseButton, bool>,
    released_mouse_buttons: HashMap<MouseButton, bool>,
    cursor_pos: Option<Point2d>,
}

impl InputState {
    pub fn new() -> InputState {
        InputState {
            current_keys: HashMap::new(),
            pressed_keys: HashMap::new(),
            released_keys: HashMap::new(),
            current_mouse_buttons: HashMap::new(),
            pressed_mouse_buttons: HashMap::new(),
            released_mouse_buttons: HashMap::new(),
            cursor_pos: None,
        }
    }

    pub fn clear_pressed_and_released(&mut self) {
        self.pressed_keys.clear();
        self.released_keys.clear();
        self.pressed_mouse_buttons.clear();
        self.released_mouse_buttons.clear();
    }

    pub fn handle_keyboard_input(&mut self, input: &KeyboardInput) {
        let keycode: VirtualKeyCode = input.virtual_keycode.unwrap();

        match input.state {
            ElementState::Pressed => {
                if !self.is_key_held(keycode) {
                    self.pressed_keys.insert(keycode, true);
                }

                self.current_keys.insert(keycode, true);
            }
            ElementState::Released => {
                self.released_keys.insert(keycode, true);
                self.current_keys.insert(keycode, false);
            }
        }
    }

    pub fn handle_mouse_input(&mut self, state: ElementState, button: MouseButton) {
        match state {
            ElementState::Pressed => {
                if !self.is_mouse_button_held(button) {
                    self.pressed_mouse_buttons.insert(button, true);
                }

                self.current_mouse_buttons.insert(button, true);
            }
            ElementState::Released => {
                self.released_mouse_buttons.insert(button, true);
                self.current_mouse_buttons.insert(button, false);
            }
        }
    }

    pub fn handle_cursor_movement(&mut self, position: PhysicalPosition<f64>) {
        self.cursor_pos = Some(Point2d::new(position.x, position.y));
    }

    pub fn cursor_pos(&self) -> Point2d {
        self.cursor_pos.unwrap_or(Point2d::origin())
    }

    #[allow(dead_code)]
    pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        *self.pressed_mouse_buttons.get(&button).unwrap_or(&false)
    }

    #[allow(dead_code)]
    pub fn is_mouse_button_released(&self, button: MouseButton) -> bool {
        *self.released_mouse_buttons.get(&button).unwrap_or(&false)
    }

    #[allow(dead_code)]
    pub fn is_mouse_button_held(&self, button: MouseButton) -> bool {
        *self.current_mouse_buttons.get(&button).unwrap_or(&false)
    }

    #[allow(dead_code)]
    pub fn is_key_pressed(&self, keycode: VirtualKeyCode) -> bool {
        *self.pressed_keys.get(&keycode).unwrap_or(&false)
    }

    #[allow(dead_code)]
    pub fn is_key_released(&self, keycode: VirtualKeyCode) -> bool {
        *self.released_keys.get(&keycode).unwrap_or(&false)
    }

    #[allow(dead_code)]
    pub fn is_key_held(&self, keycode: VirtualKeyCode) -> bool {
        *self.current_keys.get(&keycode).unwrap_or(&false)
    }
}
