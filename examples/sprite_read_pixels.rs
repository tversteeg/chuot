//! Show how to read the pixels of a sprite.
//!
//! Calling `ctx.sprite(..).read_pixels()` requires the `read-image` feature flag.
//!
//! The sprite image for this example is:
//! {{ img(src="/assets/threeforms.png" alt="Font") }}
//! With the following TOML:
//! ```toml
//! offset = "Middle"
//! ```

use chuot::{glamour::Size2, Context, GameConfig, PixelGame};

/// Which sprite to draw.
const SPRITE: &str = "threeforms";

/// Define a game state for our example.
#[derive(Default)]
struct GameState {
    /// Pixel value underneath the mouse.
    pixel: Option<u32>,
    /// Pixel values of the sprite.
    ///
    /// Will be set once.
    pixels: Option<(Size2, Vec<u32>)>,
}

impl PixelGame for GameState {
    /// Update the game.
    fn update(&mut self, ctx: Context) {
        // Cache the pixels of the sprite once
        if self.pixels.is_none() {
            self.pixels = Some(ctx.sprite(SPRITE).read_pixels());
        }

        // Get the cached pixel values
        let Some((sprite_size, pixels)) = &self.pixels else {
            return;
        };

        // Only do something when the mouse is on a pixel
        let Some(mouse) = ctx.mouse() else { return };

        // Offset the mouse with the sprite in the middle of the screen
        let mouse = mouse - ctx.size().to_vector() / 2.0
            // Also offset by half of the sprite itself since it's centered in the configuration
            + sprite_size.to_vector() / 2.0;

        // Convert the mouse coordinate to the pixel, ignoring when we don't hover over the image
        self.pixel = if mouse.x < 0.0
            || mouse.x > sprite_size.width
            || mouse.y < 0.0
            || mouse.y > sprite_size.height
        {
            None
        } else {
            // Convert the mouse coordinates to the index inside the pixel data
            let index = mouse.x.floor() as usize + (mouse.y.floor() * sprite_size.width) as usize;

            // Return the pixel value
            Some(pixels[index])
        }
    }

    /// Render the game.
    fn render(&mut self, ctx: Context) {
        // Draw the sprite
        ctx.sprite(SPRITE)
            // Place the sprite in the middle of the screen
            .translate(ctx.size() / 2.0)
            .draw();

        if let Some(pixel) = self.pixel {
            // Draw the pixel value on the mouse
            ctx.text("Beachball", &format!("{pixel:08X}"))
                .translate(ctx.mouse().unwrap_or_default())
                .draw();
        } else {
            // Notify the user to hover
            ctx.text("Beachball", "Hover the mouse\nover the sprite")
                .translate((2.0, 2.0))
                .draw();
        }
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
        ..Default::default()
    };

    // Spawn the window and run the 'game'
    GameState::default()
        .run(chuot::load_assets!(), config)
        .expect("Error running game");
}
