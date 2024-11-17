//! Show how to read the pixels of a sprite.
//!
//! Calling `ctx.sprite(..).read_pixels()` requires the `read-texture` feature flag.
//!
//! The `threeforms.png` sprite image for this example is:
//! {{ img(src="/assets/threeforms.png" alt="Sprite") }}
//! With the following `threeforms.ron` RON configuration file for positioning the center of the sprite:
//! ```ron
//! (pivot_x: Center, pivot_y: Center)
//! ```

use chuot::{Config, Context, Game, RGBA8};

/// Which sprite to draw.
const SPRITE: &str = "threeforms";

/// Define a game state for our example.
#[derive(Default)]
struct GameState {
    /// Pixel value underneath the mouse.
    pixel: Option<RGBA8>,
}

impl Game for GameState {
    /// Update the game.
    fn update(&mut self, ctx: Context) {
        // Read the size of the sprite
        let (sprite_width, sprite_height) = ctx.sprite(SPRITE).size();
        // Read the pixels of the sprite
        let pixels = ctx.sprite(SPRITE).read_pixels();

        // Only do something when the mouse is on a pixel
        let Some((mouse_x, mouse_y)) = ctx.main_camera().mouse() else {
            return;
        };

        // Offset the mouse by the image size
        let mouse_x = mouse_x + sprite_width / 2.0;
        let mouse_y = mouse_y + sprite_height / 2.0;

        // Convert the mouse coordinate to the pixel, ignoring when we don't hover over the image
        self.pixel =
            if mouse_x < 0.0 || mouse_x > sprite_width || mouse_y < 0.0 || mouse_y > sprite_height {
                None
            } else {
                // Convert the mouse_coordinates to the index inside the pixel data
                let index = mouse_x.floor() as usize + (mouse_y.floor() * sprite_width) as usize;

                // Return the pixel value
                Some(pixels[index])
            }
    }

    /// Render the game.
    fn render(&mut self, ctx: Context) {
        // Draw the sprite
        ctx.sprite(SPRITE).draw();

        if let Some(pixel) = self.pixel {
            // Draw the pixel value on the mouse
            ctx.text(
                "Beachball",
                &format!("{:02X}{:02X}{:02X}", pixel.r, pixel.g, pixel.b),
            )
            // Use the UI camera which draws the center in the top left
            .use_ui_camera()
            .translate(ctx.mouse().unwrap_or_default())
            .draw();
        } else {
            // Notify the user to hover
            ctx.text("Beachball", "Hover the mouse\nover the sprite")
            // Use the UI camera which draws the center in the top left
            .use_ui_camera()
                .translate((2.0, 2.0))
                .draw();
        }
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
        ..Default::default()
    };

    // Spawn the window and run the 'game'
    GameState::default().run(chuot::load_assets!(), config);
}
