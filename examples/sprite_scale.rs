//! Show how to scale a sprite horizontally and vertically.
//!
//! The `threeforms.png` sprite image for this example is:
//! {{ img(src="/assets/threeforms.png" alt="Sprite") }}
//! With the following `threeforms.ron` RON configuration file for positioning the center of the sprite:
//! ```ron
//! (offset: Middle)
//! ```

use chuot::{Config, Context, Game, config::RotationAlgorithm};

/// How much we will scale with the mouse.
const SCALE_FACTOR: f32 = 50.0;

/// Define a game state for our example.
#[derive(Default)]
struct GameState {
    /// Current horizontal scaling.
    scale_x: f32,
    /// Current vertical scaling.
    scale_y: f32,
}

impl Game for GameState {
    /// Update the game.
    fn update(&mut self, ctx: Context) {
        // Scale based on the mouse cursor
        if let Some((mouse_x, mouse_y)) = ctx.mouse() {
            // Only scale when the mouse is hovering over the buffer

            // Take the distance from the center of the screen to the mouse, and multiply it with the scale factor
            self.scale_x = (mouse_x - ctx.width() / 2.0) / SCALE_FACTOR;
            self.scale_y = (mouse_y - ctx.height() / 2.0) / SCALE_FACTOR;
        } else {
            // Flip horizontally when mouse is outside of screen
            self.scale_x = -1.0;
            self.scale_y = 1.0;
        }
    }

    /// Render the game.
    fn render(&mut self, ctx: Context) {
        // Draw the rotated sprite
        ctx.sprite("threeforms")
            // Scale it
            .scale_x(self.scale_x)
            .scale_y(self.scale_y)
            .draw();
    }
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
        // We use a custom rotation algorithm shader so the sprite looks more smooth while rotating with less noisy artifacts, this one is a bit slow
        rotation_algorithm: RotationAlgorithm::CleanEdge,
        ..Default::default()
    };

    // Spawn the window and run the 'game'
    GameState::default().run(chuot::load_assets!(), config);
}
