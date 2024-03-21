//! Very simple example showing how a sound file can be loaded from disk.
//!
//! To play it press the left mouse button.

use pixel_game_lib::{audio::Audio, Context, KeyCode, MouseButton, PixelGame, WindowConfig};

/// Define empty game state.
struct GameState;

impl PixelGame for GameState {
    fn tick(&mut self, ctx: Context) {
        // Play a sound when the mouse is pressed
        if ctx.mouse_released(MouseButton::Left) {
            pixel_game_lib::asset::<Audio>("switch31").play();
        }

        // Exit when escape is pressed
        if ctx.key_pressed(KeyCode::Escape) {
            ctx.exit();
        }
    }
}

/// Run the game.
fn main() {
    // Start the game with defaults for the window
    GameState
        .run(WindowConfig::default())
        .expect("Error running game");
}
