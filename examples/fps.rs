//! Show how to add a basic FPS counter.
//!
//! Text is a bitmap split into exact sizes for each glyph.
//! The `Beachball.png` font image for this example is:
//! {{ img(src="/assets/Beachball.png" alt="Font") }}
//! With the following `Beachball.ron` RON file:
//! ```ron
//! (glyph_width: 10,  glyph_height: 10)
//! ```

use chuot::{Config, Context, Game};

/// Define a game state for our example.
#[derive(Default)]
struct GameState;

impl Game for GameState {
    /// Render the game.
    fn render(&mut self, ctx: Context) {
        // Get the FPS
        let fps = ctx.frames_per_second();

        // Load a text asset and draw the FPS with 1 decimal accuracy
        ctx.text("Beachball", &format!("{fps:.1}"))
            // Offset a bit from the corner
            .translate((2.0, 2.0))
            // Draw the text on the screen
            .draw();
    }

    /// Do nothing during the update loop.
    fn update(&mut self, _ctx: Context) {}
}

/// Open an empty window.
fn main() {
    // Game configuration
    let config = Config {
        buffer_width: 120.0,
        buffer_height: 96.0,
        // Apply a minimum of 6 times scaling for the buffer
        // Will result in a minimum, and on web exact, window size of 960x720
        scaling: 6.0,
        ..Default::default()
    };

    // Spawn the window and run the 'game'
    GameState.run(config);
}
