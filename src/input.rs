//! Handle different input events.

use gilrs::{
    Axis,
    Button, ev::state::{AxisData, ButtonData}, GamepadId, Gilrs, GilrsBuilder,
};
use hashbrown::HashMap;
use smallvec::SmallVec;
use winit::{
    event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

use crate::graphics::Graphics;

/// Any button state.
#[derive(Default)]
pub(crate) struct ButtonState {
    /// Whether the button is being held down this update tick.
    is_down: bool,
    /// Whether the button was being held down the previous update tick.
    was_down_previous_tick: bool,
}

impl ButtonState {
    /// Create a new state.
    pub(crate) const fn new(is_down: bool) -> Self {
        let was_down_previous_tick = false;

        Self {
            is_down,
            was_down_previous_tick,
        }
    }

    /// Handle the state if the button is currently pressed.
    pub(crate) fn handle_event(&mut self, pressed: bool) {
        self.is_down = pressed;
    }

    /// Handle the state changes using the update tick to respond to changes.
    pub(crate) fn update(&mut self) {
        self.was_down_previous_tick = self.is_down;
    }

    /// Whether the button is being pressed now.
    pub(crate) const fn held(&self) -> bool {
        self.is_down
    }

    /// Whether the button goes from released to pressed.
    pub(crate) const fn pressed(&self) -> bool {
        !self.was_down_previous_tick && self.is_down
    }

    /// Whether the button goes from pressed to released.
    pub(crate) const fn released(&self) -> bool {
        self.was_down_previous_tick && !self.is_down
    }
}

/// Manager for handling different input events.
pub(crate) struct Input {
    /// Mouse position.
    ///
    /// `None` if not on screen.
    mouse: Option<(f32, f32)>,
    /// Mouse button states.
    mouse_buttons: HashMap<MouseButton, ButtonState>,
    /// All keyboard buttons.
    keys: HashMap<KeyCode, ButtonState>,
    /// Horizontal scroll delta.
    scroll_delta_x: f32,
    /// Vertical scroll delta.
    scroll_delta_y: f32,
    /// Gamepad input.
    gilrs: Gilrs,
}

impl Input {
    /// Setup the input.
    pub(crate) fn new() -> Self {
        let mouse = None;
        let mouse_buttons = HashMap::new();
        let keys = HashMap::new();
        let scroll_delta_x = 0.0;
        let scroll_delta_y = 0.0;

        let gilrs = GilrsBuilder::new()
            // Manually handle all updates
            .set_update_state(false)
            .build()
            .unwrap();

        Self {
            mouse,
            mouse_buttons,
            keys,
            scroll_delta_x,
            scroll_delta_y,
            gilrs,
        }
    }

    /// Handle a winit window event.
    #[inline]
    pub(crate) fn handle_event(&mut self, event: WindowEvent, graphics: &Graphics) {
        match event {
            // Handle keyboard buttons
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(keycode) = event.physical_key {
                    let is_down = event.state == ElementState::Pressed;

                    if let Some(state) = self.keys.get_mut(&keycode) {
                        // Key already registered, update the state
                        state.handle_event(is_down);
                    } else {
                        // Key not found, register it
                        self.keys.insert(keycode, ButtonState::new(is_down));
                    }
                }
            }
            // Handle mouse cursor position
            WindowEvent::CursorMoved { position, .. } => {
                // Map the coordinates to the buffer
                self.mouse = graphics.map_window_coordinate(position.x as f32, position.y as f32);
            }
            // Handle mouse scroll wheel
            WindowEvent::MouseWheel { delta, .. } => {
                let (x, y) = match delta {
                    // Treat a line as a single pixel
                    MouseScrollDelta::LineDelta(x, y) => (x, y),
                    MouseScrollDelta::PixelDelta(position) => {
                        (position.x as f32, position.y as f32)
                    }
                };

                self.scroll_delta_x = x;
                self.scroll_delta_y = y;
            }
            // Handle mouse buttons
            WindowEvent::MouseInput { state, button, .. } => {
                let is_down = state == ElementState::Pressed;

                if let Some(state) = self.mouse_buttons.get_mut(&button) {
                    // Key already registered, update the state
                    state.handle_event(is_down);
                } else {
                    // Key not found, register it
                    self.mouse_buttons.insert(button, ButtonState::new(is_down));
                }
            }
            _ => (),
        }
    }

    /// Update all registered buttons.
    ///
    /// Only allowed to be called once per update tick.
    #[inline]
    pub(crate) fn update(&mut self) {
        // Update all button states, needed to handle "pressed" and "released"
        self.mouse_buttons
            .iter_mut()
            .for_each(|(_, state)| state.update());
        self.keys.iter_mut().for_each(|(_, state)| state.update());

        // Tell the gamepad that we handled all events
        // This will increase the internal counter we can check on release and on press events with
        self.gilrs.inc();

        // Handle the gamepad events
        while let Some(gamepad_event) = self.gilrs.next_event() {
            self.gilrs.update(&gamepad_event);
        }
    }

    /// Check the mouse pressed state for a mouse button.
    #[inline]
    #[must_use]
    pub(crate) fn mouse_pressed(&self, mouse_button: MouseButton) -> bool {
        let Some(mouse_button_state) = self.mouse_buttons.get(&mouse_button) else {
            return false;
        };

        mouse_button_state.pressed()
    }

    /// Check the mouse released state for a mouse button.
    #[inline]
    #[must_use]
    pub(crate) fn mouse_released(&self, mouse_button: MouseButton) -> bool {
        let Some(mouse_button_state) = self.mouse_buttons.get(&mouse_button) else {
            return false;
        };

        mouse_button_state.released()
    }

    /// Check the mouse held state for a mouse button.
    #[inline]
    #[must_use]
    pub(crate) fn mouse_held(&self, mouse_button: MouseButton) -> bool {
        let Some(mouse_button_state) = self.mouse_buttons.get(&mouse_button) else {
            return false;
        };

        mouse_button_state.held()
    }

    /// Absolute mouse position if on screen.
    pub(crate) const fn mouse(&self) -> Option<(f32, f32)> {
        self.mouse
    }

    /// How much the mouse scrolled this update tick.
    pub(crate) const fn scroll_diff(&self) -> (f32, f32) {
        (self.scroll_delta_x, self.scroll_delta_y)
    }

    /// Check the key pressed state for a keyboard button.
    #[inline]
    #[must_use]
    pub(crate) fn key_pressed(&self, key: KeyCode) -> bool {
        let Some(key_button_state) = self.keys.get(&key) else {
            return false;
        };

        key_button_state.pressed()
    }

    /// Check the key released state for a keyboard button.
    #[inline]
    #[must_use]
    pub(crate) fn key_released(&self, key: KeyCode) -> bool {
        let Some(key_button_state) = self.keys.get(&key) else {
            return false;
        };

        key_button_state.released()
    }

    /// Check the key held state for a keyboard button.
    #[inline]
    #[must_use]
    pub(crate) fn key_held(&self, key_button: KeyCode) -> bool {
        let Some(key_button_state) = self.keys.get(&key_button) else {
            return false;
        };

        key_button_state.held()
    }

    /// List connected gamepads.
    #[inline]
    #[must_use]
    pub(crate) fn gamepads_ids(&self) -> SmallVec<[GamepadId; 4]> {
        self.gilrs.gamepads().map(|(id, _gamepad)| id).collect()
    }

    /// Whether a gamepad button goes from "not pressed" to "pressed".
    #[inline]
    #[must_use]
    pub fn gamepad_button_pressed(&self, gamepad_id: GamepadId, button: Button) -> Option<bool> {
        self.gilrs
            .connected_gamepad(gamepad_id)
            .and_then(|gamepad| {
                gamepad.button_data(button).map(|button_data| {
                    // The counter is updated once per update loop so we can keep track of that to ensure that the button only now changed state
                    button_data.is_pressed() && button_data.counter() == self.gilrs.counter()
                })
            })
    }

    /// Whether a gamepad button goes from "pressed" to "not pressed".
    #[inline]
    #[must_use]
    pub fn gamepad_button_released(&self, gamepad_id: GamepadId, button: Button) -> Option<bool> {
        self.gilrs
            .connected_gamepad(gamepad_id)
            .and_then(|gamepad| {
                gamepad.button_data(button).map(|button_data| {
                    // The counter is updated once per update loop so we can keep track of that to ensure that the button only now changed state
                    !button_data.is_pressed() && button_data.counter() == self.gilrs.counter()
                })
            })
    }

    /// Whether a gamepad button is in a "pressed" state.
    #[inline]
    #[must_use]
    pub fn gamepad_button_held(&self, gamepad_id: GamepadId, button: Button) -> Option<bool> {
        self.gilrs
            .connected_gamepad(gamepad_id)
            .map(|gamepad| gamepad.is_pressed(button))
    }

    /// "Value" of a gamepad button between 0.0 and 1.0.
    #[inline]
    #[must_use]
    pub fn gamepad_button_value(&self, gamepad_id: GamepadId, button: Button) -> Option<f32> {
        self.gilrs
            .connected_gamepad(gamepad_id)
            .and_then(|gamepad| gamepad.button_data(button).map(ButtonData::value))
    }

    /// "Value" of a gamepad element between -1.0 and 1.0.
    #[inline]
    #[must_use]
    pub fn gamepad_axis(&self, gamepad_id: GamepadId, axis: Axis) -> Option<f32> {
        self.gilrs
            .connected_gamepad(gamepad_id)
            .and_then(|gamepad| gamepad.axis_data(axis).map(AxisData::value))
    }
}
