//! Show how to draw a sprite.
//!
//! The `threeforms.png` sprite image for this example is:
//! {{ img(src="/assets/threeforms.png" alt="Sprite") }}
//! With the following `threeforms.ron` RON configuration file for positioning the center of the sprite:
//! ```ron
//! (offset: Middle)
//! ```

use chuot::{Config, Context, Game};

/// Define a game state for our example.
struct GameState;

impl Game for GameState {
    /// Render the game.
    fn render(&mut self, ctx: Context) {
        // Load a sprite asset and draw it
        ctx.sprite("threeforms")
            // Draw the sprite on the screen, it is centered because by default it uses the main camera
            .draw();
    }

    /// Do nothing during the update loop.
    fn update(&mut self, _ctx: Context) {}
}

/// Open an empty window.
fn main() {
    // Game configuration
    let config = Config {
        buffer_width: 240.0,
        buffer_height: 192.0,
        // Apply a minimum of 3 times scaling for the buffer
        // Will result in a minimum, and on web exact, window size of 720x576
        scaling: 3.0,
        ..Default::default()
    };

    // Spawn the window and run the 'game'
    GameState.run(chuot::load_assets!(), config);
}
