//! Create a sprite without loading any assets from disk or memory.

use chuot::{AssetSource, Config, Context, Game, RGBA8};

/// Define a game state for our example.
#[derive(Default)]
struct GameState {
    /// Whether the sprite already has been created.
    created: bool,
}

impl Game for GameState {
    /// Do nothing during the update loop.
    fn update(&mut self, ctx: Context) {
        // Create the sprite once
        if !self.created {
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

            // Create a new sprite with the size of the screen
            ctx.sprite("pattern").create((width, height), pixels);

            // Only create the image once
            self.created = true;
        }
    }

    /// Render the game.
    fn render(&mut self, ctx: Context) {
        if !self.created {
            // Only do something when the texture has been created
            return;
        }

        // Load a sprite asset and draw it
        ctx.sprite("pattern")
            // Draw the sprite on the screen
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
        ..Default::default()
    };

    // Spawn the window and run the 'game'
    GameState::default().run(
        // In this example we don't use any stored assets so we also don't have to embed them into the binary
        AssetSource::new(),
        config,
    );
}
