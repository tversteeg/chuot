//! Use various sprite operations to create a pseudo-3D effect.
//!
//! The `lizard.png` sprite "font" for this example is:
//! {{ img(src="/assets/lizard.png" alt="Lizard") }}
//! It has been created with [voxelizer](https://drububu.com/miscellaneous/voxelizer/?out=stk) from a 3D model, but there are many other tools to generate a sprite stack image.
//! With the following `lizard.ron` RON configuration file for parsing the different layers as characters:
//! ```ron
//! (glyph_width: 23,  glyph_height: 74, first_char: 1, last_char: 18)
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
        // Get the layer range as "characters" of the font
        let start_layer = ctx.font("lizard").first_char();
        let last_layer = ctx.font("lizard").last_char();

        for layer in start_layer..=last_layer {
            ctx.font("lizard")
                .glyph(layer)
                // Draw each layer a slight bit higher than the previous to give the illusion of a 3D image
                .translate_y(-(layer as f32))
                // Changing the center of rotation also works when applied to every layer
                .pivot(
                    // Rotate around the horizontal center
                    0.5,
                    // Rotate at a bit lower than the center so the body of the lizard is centered
                    0.7,
                )
                // Rotate it
                .rotate(self.rotation)
                .draw();
        }
    }
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
        // Use a nice rotation algorithm to reduce pixel noise
        rotation_algorithm: RotationAlgorithm::CleanEdge,
        ..Default::default()
    };

    // Spawn the window and run the 'game'
    GameState::default().run(chuot::load_assets!(), config);
}
