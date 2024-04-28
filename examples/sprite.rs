//! Show how to draw a sprite.
//!
//! The sprite image for this example is:
//! {{ img(src="/assets/threeforms.png" alt="Font") }}
//! With the following TOML:
//! ```toml
//! offset = "Middle"
//! ```

use chuot::{Context, GameConfig, KeyCode, PixelGame};
use glamour::Size2;

/// Define a game state for our example.
#[derive(Default)]
struct GameState;

impl PixelGame for GameState {
    /// Update the game.
    fn update(&mut self, ctx: Context) {
        // Exit when escape is pressed
        if ctx.key_pressed(KeyCode::Escape) {
            ctx.exit();
        }
    }

    /// Render the game.
    fn render(&mut self, ctx: Context) {
        // Load a sprite asset and draw it
        ctx.sprite("threeforms")
            // Place the sprite in the middle of the screen
            .translate(ctx.size() / 2.0)
            // Draw the sprite on the screen
            .draw();
    }
}

/// Open an empty window.
fn main() {
    // Game configuration
    let config = GameConfig {
        buffer_size: Size2::new(240.0, 192.0),
        // Apply a minimum of 3 times scaling for the buffer
        // Will result in a minimum, and on web exact, window size of 720x576
        scaling: 3.0,
        ..Default::default()
    };

    // Spawn the window and run the 'game'
    GameState
        .run(chuot::load_assets!(), config)
        .expect("Error running game");
}
