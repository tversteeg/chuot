//! Show how a sound file can be played.
//!
//! To play it press the left mouse button on the canvas.

use chuot::{context::MouseButton, Config, Context, Game};

/// Define empty game state.
struct GameState;

impl Game for GameState {
    /// Handle input events to play a sound.
    fn update(&mut self, ctx: Context) {
        // Play a sound when the mouse is pressed
        if ctx.mouse_released(MouseButton::Left) {
            // Load the asset if not loaded yet
            ctx.audio("switch31")
                // Play the loaded sound
                .play();
        }
    }

    /// Don't render anything.
    fn render(&mut self, _ctx: Context) {}
}

/// Run the game.
fn main() {
    // Start the game with defaults for the window
    GameState.run(
        chuot::load_assets!(),
        Config::default()
            .with_buffer_size((720.0, 576.0))
            .with_scaling(1.0),
    );
}
