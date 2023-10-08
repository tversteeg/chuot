use pixel_game_lib::{
    font::Font,
    vek::Vec2,
    window::{Key, WindowConfig},
};

/// Load the font from disk or embedded if using external assets.
#[cfg(any(feature = "hot-reloading-assets", feature = "embedded-assets"))]
fn font() -> Font {
    pixel_game_lib::asset_owned("Beachball")
}
/// Use the default font if using the `default-font` feature flag.
#[cfg(all(
    not(any(feature = "hot-reloading-assets", feature = "embedded-assets")),
    feature = "default-font"
))]
fn font() -> Font {
    Font::default()
}
/// Throw an error when none of the features are loaded.
#[cfg(not(any(
    feature = "hot-reloading-assets",
    feature = "embedded-assets",
    feature = "default-font"
)))]
compile_error!("Either feature \"assets\" or \"default-font\" must be enabled for this example.");

/// Open an empty window.
fn main() {
    // Window configuration with default pixel size and scaling
    let window_config = WindowConfig {
        ..Default::default()
    };

    // Load a font for the widgets
    let font = font();

    // Open the window and start the game-loop
    pixel_game_lib::window(
        // We don't use any state so we pass a zero-sized type
        (),
        window_config.clone(),
        // Update loop exposing input events we can handle, this is where you would handle the game logic
        |_state, input, _mouse, _dt| {
            // Exit when escape is pressed
            input.key_pressed(Key::Escape)
        },
        // Render loop exposing the pixel buffer we can mutate
        move |_state, canvas, _dt| {
            // Draw the text at the center of the screen
            font.render_centered(
                "pixel-game-lib font example",
                Vec2::new(
                    window_config.buffer_size.w / 2,
                    window_config.buffer_size.h / 2,
                )
                .as_(),
                canvas,
                window_config.buffer_size,
            );
        },
    )
    .expect("Error opening window");
}
