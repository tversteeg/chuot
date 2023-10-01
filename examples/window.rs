use pixel_game_lib::WindowConfig;

/// Define a game state with a simple counter.
struct GameState {
    /// Simple number that will be incremented when the mouse is pressed.
    ///
    /// For each number a pixel is drawn.
    pub pixels_to_draw: usize,
}

/// Open an empty window.
/// Entry point starting either a WASM future or a Tokio runtime.
fn main() {
    // Active modifiable state
    let state = GameState { pixels_to_draw: 0 };

    // Window configuration
    let window_config = WindowConfig {
        ..Default::default()
    };

    // Open the window and start the game-loop
    pixel_game_lib::window(
        state,
        window_config.clone(),
        // Update loop exposing input events we can handle, this is where you would handle the game logic
        |_state, _dt| {},
        // Render loop exposing the pixel buffer we can mutate
        move |state, canvas, _dt| {
            // Ensure that we don't draw pixels outside of the canvas
            let max_pixels_to_draw = window_config.buffer_size.product();
            let pixels_to_draw = state.pixels_to_draw.min(max_pixels_to_draw);

            // Draw a white color for each pixel
            canvas[0..pixels_to_draw].fill(0xFFFFFFFF);
        },
    )
    .expect("Error opening window");
}
