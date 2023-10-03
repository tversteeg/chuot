use pixel_game_lib::{Extent2, Key, WindowConfig};

/// Define a game state with a simple counter.
struct GameState {
    /// Simple number that will be incremented when the mouse is pressed.
    ///
    /// For each number a pixel is drawn.
    pub pixels_to_draw: usize,
}

/// Open an empty window.
fn main() {
    // Active modifiable state
    let state = GameState { pixels_to_draw: 0 };

    // Window configuration with huge pixels
    let window_config = WindowConfig {
        buffer_size: Extent2::new(64, 64),
        scaling: 8,
        ..Default::default()
    };

    // Open the window and start the game-loop
    pixel_game_lib::window(
        state,
        window_config.clone(),
        // Update loop exposing input events we can handle, this is where you would handle the game logic
        |state, input, _mouse, _dt| {
            // Increment when mouse is clicked
            if input.mouse_held(0) {
                state.pixels_to_draw += 1;
            }

            // Exit when escape is pressed
            input.key_pressed(Key::Escape)
        },
        // Render loop exposing the pixel buffer we can mutate
        move |state, canvas, _dt| {
            // Ensure that we don't draw pixels outside of the canvas
            let max_pixels_to_draw = window_config.buffer_size.product();
            let pixels_to_draw = state.pixels_to_draw.min(max_pixels_to_draw);

            // Draw a red color for each pixel
            canvas[0..pixels_to_draw].fill(0xFFFF0000);
        },
    )
    .expect("Error opening window");
}
