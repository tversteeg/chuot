//! Show how to load a custom asset.
//!
//! When developing on the desktop you can update the text live by editing the file, it will be hot-reloaded automatically.
//!
//! `example.txt`:
//! ```txt
//! This string is loaded
//! from 'assets/example.txt'!
//! ```

use chuot::{
    assets::{loadable::Loadable, loader::Loader, Id},
    context::ContextInner,
    Config, Context, Game,
};

/// A custom asset loader for loading '.txt' files.
struct TxtLoader;

impl Loader<String> for TxtLoader {
    const EXTENSION: &'static str = "txt";

    /// Load the bytes into UTF-8, ignoring invalid characters.
    fn load(bytes: &[u8], _id: &Id) -> String {
        String::from_utf8_lossy(bytes).into_owned()
    }
}

/// We define a custom asset that will load a string from a '.txt' file.
struct TxtString(pub String);

impl Loadable for TxtString {
    fn load_if_exists(id: &Id, ctx: &mut ContextInner) -> Option<Self>
    where
        Self: Sized,
    {
        // Use the created loader to load a txt asset
        let text = ctx.asset_source.load_if_exists::<TxtLoader, _>(id)?;

        Some(Self(text))
    }
}

/// Define an empty game state, because all asset state will be loaded using the context.
struct GameState;

impl Game for GameState {
    /// Game render tick, handle drawing things here.
    fn render(&mut self, ctx: Context) {
        // Load a reference to the asset
        let example_txt = ctx.asset::<TxtString>("example");

        // Draw the asset text
        ctx.text("Beachball", &example_txt.0)
            // Use the UI camera which draws the center in the top left
            .use_ui_camera()
            .draw();
    }

    /// Do nothing during the update loop.
    fn update(&mut self, _ctx: Context) {}
}

/// Open an empty window.
fn main() {
    // Spawn the window with the default configuration but with a horizontally stretched buffer for displaying longer text
    GameState.run(
        chuot::load_assets!(),
        Config::default()
            .with_buffer_size((360.0, 50.0))
            .with_scaling(2.0),
    );
}
