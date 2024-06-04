//! Handle the state of connected gamepads.
//!
//! Show text for most button presses.
//! The state of the gamepads is checked in the `render` function, which is something you normally would not want to do, you would want to check it in the `update` function. But for this example that doesn't matter because the update delta time is low enough that it lasts multiple render functions.

use chuot::{
    context::{Axis, Button},
    Context, GameConfig, PixelGame,
};

/// Define empty game state.
struct GameState;

impl PixelGame for GameState {
    /// Show some text with gamepad buttons being pressed.
    fn render(&mut self, ctx: Context) {
        // Position of the next line of text
        let mut line_y = 2.0;

        // Get all IDs of connected gamepads
        let connected_gamepad_ids = ctx.gamepads_ids();

        if connected_gamepad_ids.is_empty() {
            ctx.text("Beachball", "Activate or\nconnect a gamepad")
                .translate((2.0, line_y))
                .draw();
        }

        // Show the D-Pad button presses for each connected gamepad
        for (index, gamepad_id) in connected_gamepad_ids.iter().enumerate() {
            // Draw the gamepad index
            ctx.text("Beachball", &format!("Connected gamepad {}:", index + 1))
                .translate((2.0, line_y))
                .draw();
            line_y += 12.0;

            // Show the button states
            for (button, name) in [
                (Button::North, "North"),
                (Button::South, "South"),
                (Button::East, "East"),
                (Button::West, "West"),
                (Button::C, "C"),
                (Button::Z, "Z"),
                (Button::LeftTrigger, "Left Trigger"),
                (Button::LeftTrigger2, "Left Trigger 2"),
                (Button::RightTrigger, "Right Trigger"),
                (Button::RightTrigger2, "Right Trigger 2"),
                (Button::Select, "Select"),
                (Button::Start, "Start"),
                (Button::Mode, "Mode"),
                (Button::LeftThumb, "Left Thumb"),
                (Button::RightThumb, "Right Thumb"),
                (Button::DPadUp, "D-Pad Up"),
                (Button::DPadDown, "D-Pad Down"),
                (Button::DPadLeft, "D-Pad Left"),
                (Button::DPadRight, "D-Pad Right"),
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
                        .translate((2.0, line_y))
                        .draw();
                    line_y += 12.0;
                }

                // Show the button value if an inbetween value is received
                if let Some(value) = ctx.gamepad_button_value(*gamepad_id, button) {
                    // Only show when engaged
                    if value > 0.0 && value < 1.0 {
                        ctx.text("Beachball", &format!(" {name}: {value:.1}"))
                            .translate((2.0, line_y))
                            .draw();
                        line_y += 12.0;
                    }
                }
            }

            // Show the axis values
            for (axis, name) in [
                (Axis::LeftStickX, "Left Stick X"),
                (Axis::LeftStickY, "Left Stick Y"),
                (Axis::LeftZ, "Left Z"),
                (Axis::RightStickX, "Right Stick X"),
                (Axis::RightStickY, "Right Stick Y"),
                (Axis::RightZ, "Right Z"),
                (Axis::DPadX, "D-Pad X"),
                (Axis::DPadY, "D-Pad Y"),
            ] {
                if let Some(value) = ctx.gamepad_axis(*gamepad_id, axis) {
                    // Only show when engaged
                    if value != 0.0 {
                        ctx.text("Beachball", &format!(" {name}: {value:.1}"))
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
    GameState
        .run(
            chuot::load_assets!(),
            GameConfig::default()
                .with_buffer_size((240.0, 192.0))
                .with_scaling(3.0)
                // Ensure the gamepads state only updates 10 times every frame for this example
                .with_update_delta_time(0.1),
        )
        .expect("Error running game");
}
