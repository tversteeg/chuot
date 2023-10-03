use pixel_game_lib::{font::Font, Key, Vec2, WindowConfig};

/// Open an empty window.
fn main() {
    // Window configuration with default pixel size and scaling
    let window_config = WindowConfig {
        ..Default::default()
    };

    // Load the default font, requires 'default-font' feature to be on
    let font = Font::default();

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
