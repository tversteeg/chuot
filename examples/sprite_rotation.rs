//! Show how simple it is to rotate a sprite.
//!
//! The `threeforms.png` sprite image for this example is:
//! {{ img(src="/assets/threeforms.png" alt="Sprite") }}
//! With the following `threeforms.ron` RON configuration file for positioning the center of the sprite:
//! ```ron
//! (pivot_x: Center, pivot_y: Center)
//! ```

use chuot::{config::RotationAlgorithm, Config, Context, Game};

/// Define a game state for our example.
#[derive(Default)]
struct GameState {
    /// Current rotation.
    rotation: f32,
}

impl Game for GameState {
    /// Update the game.
    fn update(&mut self, ctx: Context) {
        // Increment the rotation with with the timestep so it rotates smoothly
        self.rotation += ctx.delta_time();
    }

    /// Render the game.
    fn render(&mut self, ctx: Context) {
        // Draw the rotated sprite
        ctx.sprite("threeforms")
            // Rotate it
            .rotate(self.rotation)
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
