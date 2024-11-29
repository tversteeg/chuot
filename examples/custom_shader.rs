//! Show how to load a custom shader.
//! TODO.

use chuot::{Config, Context, Game};

/// Define a game state for our example.
struct GameState;

impl Game for GameState {
    /// Render the game.
    fn render(&mut self, ctx: Context) {
        // Draw a sprite with a custom shader
        ctx.sprite("threeforms")
            // Use the custom shader
            .shader("shader")
            .translate_x(50.0)
            .draw();

        // Draw a sprite with the default shader
        ctx.sprite("threeforms").translate_x(-50.0).draw();
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
