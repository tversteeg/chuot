//! Show how a fullscreen mode can be toggled.
//!
//! Press 'f' to toggle fullscreen.

use chuot::{context::KeyCode, Config, Context, Game};

/// Define empty game state.
struct GameState;

impl Game for GameState {
    /// Handle input events to toggle fullscreen.
    fn update(&mut self, ctx: Context) {
        // Toggle fullscreen when 'f' is pressed
        if ctx.key_released(KeyCode::KeyF) {
            ctx.toggle_fullscreen();
        }
    }

    /// Render text explaining what to do.
    fn render(&mut self, ctx: Context) {
        ctx.text("Beachball", "Press 'f'\nto toggle\nfullscreen")
            .translate((2.0, 2.0))
            .draw();
    }
}

/// Run the game.
fn main() {
    // Start the game with defaults for the window
    GameState.run(
        chuot::load_assets!(),
        Config::default()
            .with_buffer_width(120.0)
            .with_buffer_height(96.0)
            .with_scaling(6.0),
    );
}
