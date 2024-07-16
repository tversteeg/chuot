//! Handle the state of connected gamepads.
//!
//! Show text for most button presses.
//! The state of the gamepads is checked in the `render` function, which is something you normally would not want to do, you would want to check it in the `update` function. But for this example that doesn't matter because the update delta time is low enough that it lasts multiple render functions.

use chuot::{Config, Context, Game, GamepadAxis, GamepadButton};

/// Define empty game state.
struct GameState;

impl Game for GameState {
    /// Show some text with gamepad buttons being pressed.
    fn render(&mut self, ctx: Context) {
        // Position of the next line of text
        let mut line_y = 2.0;

        // Get all IDs of connected gamepads
        let connected_gamepad_ids = ctx.gamepad_ids();

        if connected_gamepad_ids.is_empty() {
            ctx.text("Beachball", "Activate or\nconnect a gamepad")
                .use_ui_camera()
                .translate((2.0, line_y))
                .draw();
        }

        // Show the D-Pad button presses for each connected gamepad
        for (index, gamepad_id) in connected_gamepad_ids.iter().enumerate() {
            // Draw the gamepad index
            ctx.text("Beachball", &format!("Connected gamepad {}:", index + 1))
                .use_ui_camera()
                .translate((2.0, line_y))
                .draw();
            line_y += 12.0;

            // Show the button states
            for (button, name) in [
                (GamepadButton::North, "North"),
                (GamepadButton::South, "South"),
                (GamepadButton::East, "East"),
                (GamepadButton::West, "West"),
                (GamepadButton::C, "C"),
                (GamepadButton::Z, "Z"),
                (GamepadButton::LeftTrigger, "Left Trigger"),
                (GamepadButton::LeftTrigger2, "Left Trigger 2"),
                (GamepadButton::RightTrigger, "Right Trigger"),
                (GamepadButton::RightTrigger2, "Right Trigger 2"),
                (GamepadButton::Select, "Select"),
                (GamepadButton::Start, "Start"),
                (GamepadButton::Mode, "Mode"),
                (GamepadButton::LeftThumb, "Left Thumb"),
                (GamepadButton::RightThumb, "Right Thumb"),
                (GamepadButton::DPadUp, "D-Pad Up"),
                (GamepadButton::DPadDown, "D-Pad Down"),
                (GamepadButton::DPadLeft, "D-Pad Left"),
                (GamepadButton::DPadRight, "D-Pad Right"),
            ] {
                // Get the state of the button
                let state = if ctx.gamepad_button_pressed(*gamepad_id, button) == Some(true) {
                    Some("Pressed")
                } else if ctx.gamepad_button_released(*gamepad_id, button) == Some(true) {
                    Some("Released")
                } else if ctx.gamepad_button_held(*gamepad_id, button) == Some(true) {
                    Some("Held")
                } else {
                    None
                };

                // Show the button state text
                if let Some(state) = state {
                    ctx.text("Beachball", &format!(" {name}: {state}"))
                        .use_ui_camera()
                        .translate((2.0, line_y))
                        .draw();
                    line_y += 12.0;
                }

                // Show the button value if an inbetween value is received
                if let Some(value) = ctx.gamepad_button_value(*gamepad_id, button) {
                    // Only show when engaged
                    if value > 0.0 && value < 1.0 {
                        ctx.text("Beachball", &format!(" {name}: {value:.1}"))
                            .use_ui_camera()
                            .translate((2.0, line_y))
                            .draw();
                        line_y += 12.0;
                    }
                }
            }

            // Show the axis values
            for (axis, name) in [
                (GamepadAxis::LeftStickX, "Left Stick X"),
                (GamepadAxis::LeftStickY, "Left Stick Y"),
                (GamepadAxis::LeftZ, "Left Z"),
                (GamepadAxis::RightStickX, "Right Stick X"),
                (GamepadAxis::RightStickY, "Right Stick Y"),
                (GamepadAxis::RightZ, "Right Z"),
                (GamepadAxis::DPadX, "D-Pad X"),
                (GamepadAxis::DPadY, "D-Pad Y"),
            ] {
                if let Some(value) = ctx.gamepad_axis(*gamepad_id, axis) {
                    // Only show when engaged
                    if value != 0.0 {
                        ctx.text("Beachball", &format!(" {name}: {value:.1}"))
                            .use_ui_camera()
                            .translate((2.0, line_y))
                            .draw();
                        line_y += 12.0;
                    }
                }
            }
        }
    }

    /// Do nothing during the update loop.
    fn update(&mut self, _ctx: Context) {}
}

/// Run the game.
fn main() {
    // Start the game with defaults for the window
    GameState.run(
        chuot::load_assets!(),
        Config::default()
            .with_buffer_size((240.0, 192.0))
            .with_scaling(3.0)
            // Ensure the gamepads state only updates 10 times every frame for this example
            .with_update_delta_time(0.1),
    );
}
