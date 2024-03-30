//! Show how to load a custom asset.
//!
//! To define custom assets the [`serde`](https://docs.rs/serde/latest/serde) crate is required.
//!
//! When the `hot-reloading-assets` flag is enabled you can update the text live by editing the file.

use std::str::FromStr;

use glamour::Size2;
use pixel_game_lib::{
    assets::{Asset, BoxedError},
    glamour::Vector2,
    Context, GameConfig, KeyCode, PixelGame,
};
use serde::Deserialize;

/// We define a custom asset that will load a string from a '.txt' file.
#[derive(Asset, Deserialize)]
#[asset_format = "txt"]
struct TxtString(pub String);

impl FromStr for TxtString {
    type Err = BoxedError;

    fn from_str(string_from_file: &str) -> Result<Self, Self::Err> {
        Ok(Self(string_from_file.to_owned()))
    }
}

/// Define an empty game state, because all asset state will be loaded using the context.
struct GameState;

impl PixelGame for GameState {
    // Game loop tick, this is where you would handle the game logic
    fn tick(&mut self, ctx: Context) {
        // Load a reference to the asset
        let example_txt = ctx.asset::<TxtString>("example");

        // Draw the asset text
        ctx.draw_text("Beachball", Vector2::ZERO, &example_txt.0);

        // Exit when escape is pressed
        if ctx.key_pressed(KeyCode::Escape) {
            ctx.exit();
        }
    }
}

/// Open an empty window.
fn main() {
    // Spawn the window with the default configuration but with a horizontally stretched buffer for displaying longer text
    GameState {}
        .run(GameConfig::default().with_buffer_size(Size2::new(400.0, 50.0)))
        .expect("Error running game");
}
