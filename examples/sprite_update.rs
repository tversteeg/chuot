//! Show how to update the pixels of a sprite.
//!
//! The `threeforms.png` sprite image for this example is:
//! {{ img(src="/assets/threeforms.png" alt="Sprite") }}
//! With the following `threeforms.ron` RON configuration file for positioning the center of the sprite:
//! ```ron
//! (offset: Middle)
//! ```

use chuot::{config::RotationAlgorithm, context::MouseButton, Config, Context, Game, RGBA8};

/// Which sprite to draw.
const SPRITE: &str = "threeforms";

/// Define a game state for our example.
struct GameState;

impl Game for GameState {
    /// Update the game.
    fn update(&mut self, ctx: Context) {
        // Only do something when the mouse is on a pixel
        let Some((mouse_x, mouse_y)) = ctx.mouse() else {
            return;
        };

        // Update the texture when pressing the mouse
        if ctx.mouse_held(MouseButton::Left) {
            // Get the size of the sprite
            let (sprite_width, sprite_height) = ctx.sprite(SPRITE).size();

            // Convert mouse to texture coordinates
            // Subtract half of the texture because the sprite is centered
            let mouse_on_texture_x = (mouse_x - sprite_width / 2.0) % sprite_width;
            let mouse_on_texture_y = (mouse_y - sprite_height / 2.0) % sprite_height;

            // Update a single pixel to green
            ctx.sprite(SPRITE).update_pixels(
                // "Rectangle" of size 1 to fill with the "pixels"
                (mouse_on_texture_x, mouse_on_texture_y, 1.0, 1.0),
                [RGBA8::new(0x00, 0xFF, 0x00, 0xFF)],
            );
        }
    }

    /// Render the game.
    fn render(&mut self, ctx: Context) {
        // Draw the sprite in a tiling fashion, filling the screen

        // Get the size of the sprite
        let (sprite_width, sprite_height) = ctx.sprite(SPRITE).size();

        // Get the size of the buffer
        let (buffer_width, buffer_height) = ctx.size();

        // Calculate how much we need to tile
        let tile_x = (buffer_width / sprite_width).ceil() as u32 + 1;
        let tile_y = (buffer_height / sprite_height).ceil() as u32 + 1;

        // Draw each sprite efficiently using an iterator
        ctx.sprite(SPRITE)
            .draw_multiple_translated((0..(tile_x * tile_y)).map(|index| {
                let x = (index % tile_x) * sprite_width as u32;
                let y = (index / tile_x) * sprite_height as u32;

                (x as f32, y as f32)
            }));
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
        // We use a custom rotation algorithm shader so the sprite looks more smooth while rotating with less noisy artifacts
        rotation_algorithm: RotationAlgorithm::Scale3x,
        // Update more so the line draws more pixels
        update_delta_time: 100_f32.recip(),
        ..Default::default()
    };

    // Spawn the window and run the 'game'
    GameState {}.run(chuot::load_assets!(), config);
}
