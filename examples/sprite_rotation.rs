//! Show how simple it is to rotate a sprite.

use chuot::{config::RotationAlgorithm, glamour::Size2, Context, GameConfig, KeyCode, PixelGame};

/// Define a game state for our example.
#[derive(Default)]
struct GameState {
    /// Current rotation.
    rotation: f32,
}

impl PixelGame for GameState {
    /// Update the game.
    fn update(&mut self, ctx: Context) {
        // Exit when escape is pressed
        if ctx.key_pressed(KeyCode::Escape) {
            ctx.exit();

            return;
        }

        // Increment the rotation with with the timestep so it rotates smoothly
        self.rotation += ctx.delta_time();
    }

    /// Render the game.
    fn render(&mut self, ctx: Context) {
        // Draw the rotated sprite
        ctx.sprite("threeforms")
            // Place the sprite in the middle of the screen
            .translate(ctx.size() / 2.0)
            // Rotate it
            .rotate(self.rotation)
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
        // We use a custom rotation algorithm shader so the sprite looks more smooth while rotating with less noisy artifacts
        rotation_algorithm: RotationAlgorithm::Scale3x,
        ..Default::default()
    };

    // Spawn the window and run the 'game'
    GameState::default()
        .run(chuot::load_assets!(), config)
        .expect("Error running game");
}
