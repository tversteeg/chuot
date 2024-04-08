//! Very simple example showing how a sound file can be loaded from disk.
//!
//! To play it press the left mouse button.

use pixel_game_lib::{audio::Audio, Context, GameConfig, KeyCode, MouseButton, PixelGame};

/// Define empty game state.
struct GameState;

impl PixelGame for GameState {
    /// Handle input events to play a sound.
    fn update(&mut self, ctx: Context) {
        // Play a sound when the mouse is pressed
        if ctx.mouse_released(MouseButton::Left) {
            // Load the asset if not loaded yet
            ctx.asset::<Audio>("switch31")
                // Play the loaded sound
                .play();
        }

        // Exit when escape is pressed
        if ctx.key_pressed(KeyCode::Escape) {
            ctx.exit();
        }
    }

    /// Don't render anything.
    fn render(&mut self, _ctx: Context) {}
}

/// Run the game.
fn main() {
    // Start the game with defaults for the window
    GameState
        .run(GameConfig::default())
        .expect("Error running game");
}
