//! Create a sprite without loading any assets from disk or memory.

use chuot::{AssetSource, Config, Context, Game, RGBA8};

/// Define an empty game state for our example.
struct GameState;

impl Game for GameState {
    /// Create the image at startup once.
    fn init(&mut self, ctx: Context) {
        // Create the sprite once
        // Size of the buffer
        let (width, height) = ctx.size();

        // Generate the pixels for the sprite with a nice XOR pattern
        let pixels = (0..(width * height) as u32)
            .map(|index| {
                let x = index % width as u32;
                let y = index / width as u32;

                // Create a nice XOR pattern
                RGBA8::new((x ^ y) as u8, 0, 0, 0xFF)
            })
            .collect::<Vec<_>>();

        // Create a new sprite with the size of the screen, pivoting at the top left
        ctx.sprite("pattern")
            .create((width, height), (0.0, 0.0), pixels);
    }

    /// Render the game.
    fn render(&mut self, ctx: Context) {
        // Load a sprite asset and draw it
        ctx.sprite("pattern")
            // Use the UI camera which draws the center in the top left
            .use_ui_camera()
            // Draw the sprite on the screen
            .draw();
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
    GameState.run(
        // In this example we don't use any stored assets so we also don't have to embed them into the binary
        AssetSource::new(),
        config,
    );
}
