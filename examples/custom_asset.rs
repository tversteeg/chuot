//! Show how to load a custom asset.
//!
//! To define custom assets the [`serde`](https://docs.rs/serde/latest/serde) crate is required.
//!
//! When the `hot-reloading-assets` flag is enabled you can update the text live by editing the file.

use std::str::FromStr;

use pixel_game_lib::{
    assets::{Asset, BoxedError, ParseLoader},
    glamour::Size2,
    Context, GameConfig, KeyCode, PixelGame,
};
use serde::Deserialize;

/// We define a custom asset that will load a string from a '.txt' file.
#[derive(Deserialize)]
struct TxtString(pub String);

impl Asset for TxtString {
    const EXTENSION: &'static str = "txt";

    // The parse loader loads any type with a `FromStr` implementation
    type Loader = ParseLoader;
}

/// Implement loading from string so the `ParseLoader` type in the `Asset` trait can load it from disk.
impl FromStr for TxtString {
    type Err = BoxedError;

    fn from_str(string_from_file: &str) -> Result<Self, Self::Err> {
        Ok(Self(string_from_file.to_owned()))
    }
}

/// Define an empty game state, because all asset state will be loaded using the context.
struct GameState;

impl PixelGame for GameState {
    /// Game render tick, handle drawing things here.
    fn render(&mut self, ctx: Context) {
        // Load a reference to the asset
        let example_txt = ctx.asset::<TxtString>("example");

        // Draw the asset text
        ctx.text("Beachball", &example_txt.0).draw();
    }

    /// Game update tick, this is where you would handle the game logic.
    fn update(&mut self, ctx: Context) {
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
        .run(
            pixel_game_lib::load_assets!(),
            GameConfig::default().with_buffer_size(Size2::new(400.0, 50.0)),
        )
        .expect("Error running game");
}
