//! Show how to pivot a sprite.
//!
//! The `threeforms.png` sprite image for this example is:
//! {{ img(src="/assets/threeforms.png" alt="Sprite") }}
//! With the following `threeforms.ron` RON configuration file where pivot is the default pivot:
//! ```ron
//! (pivot: Middle)
//! ```

use chuot::{Config, Context, Game, Pivot, config::RotationAlgorithm};

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
        // Draw a sprite at the left top
        ctx.sprite("threeforms")
            // Use the UI coordinate system so it's placed at the left top of the screen
            .use_ui_camera()
            // Override the default pivot
            .pivot(Pivot::LeftTop)
            .draw();

        // Draw a rotated sprite at the center with a slight pivot offset
        ctx.sprite("threeforms")
            .rotate(self.rotation)
            // Override the default pivot
            .pivot(Pivot::Custom {
                // Horizontal pivot is the center of the sprite
                x: ctx.sprite("threeforms").width() / 2.0,
                // Vertical pivot is a quarter of the sprite
                y: ctx.sprite("threeforms").height() / 4.0,
            })
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
