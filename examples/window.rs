//! Show how an window is spawned using the window configuration.

use glamour::Vector2;
use pixel_game_lib::{Context, GameConfig, KeyCode, PixelGame};

/// Define an empty game state, because we don't need any state for our window.
struct GameState {}

impl PixelGame for GameState {
    // Game loop tick, this is where you would handle the game logic
    fn tick(&mut self, ctx: Context) {
        // Draw a basic FPS counter
        let fps = ctx.delta_time().recip();
        ctx.draw_text("Beachball", Vector2::ZERO, format!("{fps:.1}"));

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
        .run(GameConfig::default())
        .expect("Error running game");
}
