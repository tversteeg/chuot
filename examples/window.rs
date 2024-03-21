//! Show how an empty window is spawned using the window configuration.

use pixel_game_lib::{Context, KeyCode, PixelGame, WindowConfig};

/// Define an empty game state, because we don't need any state for our window.
struct GameState {}

impl PixelGame for GameState {
    // Game loop tick, this is where you would handle the game logic
    fn tick(&mut self, ctx: Context) {
        // Exit when escape is pressed
        if ctx.key_pressed(KeyCode::Escape) {
            ctx.exit();
        }
    }
}

/// Open an empty window.
fn main() {
    // Spawn the window with the default configuration
    GameState {}
        .run(WindowConfig::default())
        .expect("Error running game");
}
