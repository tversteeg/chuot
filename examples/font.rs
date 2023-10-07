use pixel_game_lib::{
    font::Font,
    gui::Gui,
    vek::Vec2,
    window::{Key, WindowConfig},
};
use taffy::style::Style;

/// Load the font from disk or embedded if using the `assets` feature flag.
#[cfg(feature = "assets")]
fn font() -> Font {
    pixel_game_lib::asset_owned("Beachball")
}
/// Use the default font if using the `default-font` feature flag.
#[cfg(all(not(feature = "assets"), feature = "default-font"))]
fn font() -> Font {
    Font::default()
}
/// Throw an error when both features are not loaded.
#[cfg(not(any(feature = "assets", feature = "default-font")))]
compile_error!("Either feature \"assets\" or \"default-font\" must be enabled for this example.");

/// Open an empty window.
fn main() {
    // Window configuration with default pixel size and scaling
    let window_config = WindowConfig {
        ..Default::default()
    };

    // Load the font depending on the feature flag
    let font = font();

    // Create a new Gui
    let mut gui = Gui::new();
    let layout = gui.layout_mut();

    // Root node
    gui.root(layout.new_with_children(Style::DEFAULT, &[]));

    // Open the window and start the game-loop
    pixel_game_lib::window(
        // Use the gui as the state
        gui,
        window_config.clone(),
        // Update loop exposing input events we can handle, this is where you would handle the game logic
        |gui, input, _mouse, _dt| {
            // Exit when escape is pressed
            input.key_pressed(Key::Escape)
        },
        // Render loop exposing the pixel buffer we can mutate
        move |gui, canvas, _dt| {},
    )
    .expect("Error opening window");
}
