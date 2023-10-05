use pixel_game_lib::{
    font::Font,
    vek::Vec2,
    window::{Key, WindowConfig},
};

/// Load the font from disk or embedded if using the `assets` feature flag.
#[cfg(feature = "assets")]
fn font() -> Font {
    todo!()
}
/// Use the default font if using the `default-font` feature flag.
#[cfg(all(not(feature = "assets"), feature = "default-font"))]
fn font() -> Font {
    Font::default()
}
/// Throw an error when both features are not loaded.
#[cfg(not(any(feature = "assets", feature = "default-font")))]
compile_error!("Either feature \"assets\" or \"default-font\" must be enabled for this crate.");

/// Open an empty window.
fn main() {
    // Window configuration with default pixel size and scaling
    let window_config = WindowConfig {
        ..Default::default()
    };

    // Load the font depending on the feature flag
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
