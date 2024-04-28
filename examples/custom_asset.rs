//! Show how to load a custom asset.
//!
//! When the `hot-reloading-assets` flag is enabled you can update the text live by editing the file.

use chuot::{
    assets::{loader::Loader, AssetSource, Id, Loadable},
    glamour::Size2,
    Context, GameConfig, KeyCode, PixelGame,
};
use serde::Deserialize;

/// A custom asset loader for loading '.txt' files.
struct TxtLoader;

impl Loader<String> for TxtLoader {
    const EXTENSION: &'static str = "txt";

    /// Load the bytes into UTF-8, ignoring invalid characters.
    fn load(bytes: &[u8]) -> String {
        String::from_utf8_lossy(bytes).into_owned()
    }
}

/// We define a custom asset that will load a string from a '.txt' file.
#[derive(Deserialize)]
struct TxtString(pub String);

impl Loadable for TxtString {
    fn load_if_exists(id: &Id, assets: &AssetSource) -> Option<Self>
    where
        Self: Sized,
    {
        // Use the created loader to load a txt asset
        let text = assets.load_if_exists::<TxtLoader, _>(id)?;

        Some(Self(text))
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
            chuot::load_assets!(),
            GameConfig::default()
                .with_buffer_size(Size2::new(360.0, 50.0))
                .with_scaling(2.0),
        )
        .expect("Error running game");
}
