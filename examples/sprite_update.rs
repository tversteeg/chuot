//! Show how to update the pixels of a sprite.
//!
//! The sprite image for this example is:
//! {{ img(src="/assets/threeforms.png" alt="Font") }}
//! With the following TOML:
//! ```toml
//! offset = "Middle"
//! ```

use chuot::{
    config::RotationAlgorithm,
    context::MouseButton,
    glamour::{Size2, Vector2},
    Config, Context, Game,
};
use glamour::Rect;

/// Which sprite to draw.
const SPRITE: &str = "threeforms";

/// Define a game state for our example.
struct GameState;

impl Game for GameState {
    /// Update the game.
    fn update(&mut self, ctx: Context) {
        // Only do something when the mouse is on a pixel
        let Some(mouse) = ctx.mouse() else { return };

        // Update the texture when pressing the mouse
        if ctx.mouse_held(MouseButton::Left) {
            // Get the size of the sprite
            let sprite_size = ctx.sprite(SPRITE).size();

            // Convert mouse to texture coordinates
            let mouse_on_texture = Vector2::new(
                // Subtract half of the texture because the sprite is centered
                (mouse.x - sprite_size.width / 2.0) % sprite_size.width,
                (mouse.y - sprite_size.height / 2.0) % sprite_size.height,
            );

            // Update a single pixel to green
            ctx.sprite(SPRITE)
                .update_pixels(Rect::new(mouse_on_texture, Size2::splat(1.0)), [0xFF00FF00]);
        }
    }

    /// Render the game.
    fn render(&mut self, ctx: Context) {
        // Draw the sprite in a tiling fashion, filling the screen

        // Get the size of the sprite
        let sprite_size = ctx.sprite(SPRITE).size();

        // Get the size of the buffer
        let buffer_size = ctx.size();

        // Calculate how much we need to tile
        let tile_x = (buffer_size.width / sprite_size.width).ceil() as u32 + 1;
        let tile_y = (buffer_size.height / sprite_size.height).ceil() as u32 + 1;

        // Draw each sprite efficiently using an iterator
        ctx.sprite(SPRITE)
            .draw_multiple_translated((0..(tile_x * tile_y)).map(|index| {
                let x = (index % tile_x) * sprite_size.width as u32;
                let y = (index / tile_x) * sprite_size.height as u32;

                Vector2::new(x as f32, y as f32)
            }));
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
        // Update more so the line draws more pixels
        update_delta_time: 100_f32.recip(),
        ..Default::default()
    };

    // Spawn the window and run the 'game'
    GameState {}
        .run(chuot::load_assets!(), config)
        .expect("Error running game");
}
