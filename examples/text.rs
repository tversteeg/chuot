//! Show how to draw text from a font bitmap.
//!
//! Text is a bitmap split into exact sizes for each glyph.
//! The font image for this example is:
//! {{ img(src="/assets/Beachball.png" alt="Font") }}
//! With the following TOML:
//! ```toml
//! {{ load_data(path="/assets/Beachball.toml") }}
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
        // Load a text asset and draw it
        ctx.text("Beachball", "Hello world!")
            // Draw the text on the screen
            .draw();
    }
}

/// Open an empty window.
fn main() {
    // Game configuration
    let config = GameConfig {
        buffer_size: Size2::new(240.0, 192.0),
        // Apply a minimum of 3 times scaling for the buffer
        // Will result in a minimum, and on web exact, window size of 960x720
        scaling: 3.0,
        ..Default::default()
    };

    // Spawn the window and run the 'game'
    GameState
        .run(chuot::load_assets!(), config)
        .expect("Error running game");
}
