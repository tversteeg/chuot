//! Show how an window is spawned using the window configuration.

use chuot::{Context, GameConfig, KeyCode, PixelGame};

/// Define an empty game state, because we don't need any state for our window.
struct GameState {}

impl PixelGame for GameState {
    /// Game loop tick, this is where you would handle the game logic.
    fn update(&mut self, ctx: Context) {
        // Exit when escape is pressed
        if ctx.key_pressed(KeyCode::Escape) {
            ctx.exit();
        }
    }

    /// Game render tick, handle drawing things here.
    fn render(&mut self, ctx: Context) {
        // Draw a basic FPS counter
        let fps = ctx.delta_time().recip();
        ctx.text("Beachball", &format!("{fps:.1}")).draw();
    }
}

/// Open an empty window.
fn main() {
    // Spawn the window with the default configuration
    GameState {}
        .run(chuot::load_assets!(), GameConfig::default())
        .expect("Error running game");
}
