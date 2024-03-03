use pixel_game_lib::{
    canvas::Canvas,
    vek::Extent2,
    vek::Vec2,
    window::{Input, KeyCode, WindowConfig},
    PixelGame,
};

/// Define a game state with a simple counter.
struct GameState {
    /// Simple number that will be incremented when the mouse is pressed.
    ///
    /// For each number a pixel is drawn.
    pub pixels_to_draw: usize,
}

impl PixelGame for GameState {
    // Update loop exposing input events we can handle, this is where you would handle the game logic
    fn update(&mut self, input: &Input, _mouse_pos: Option<Vec2<usize>>, _dt: f32) -> bool {
        // Increment when mouse is clicked
        if input.mouse_held(0) {
            self.pixels_to_draw += 1;
        }

        // Exit when escape is pressed
        input.key_pressed(KeyCode::Escape)
    }

    // Render loop exposing the pixel buffer we can mutate
    fn render(&mut self, canvas: &mut Canvas<'_>) {
        // Fill the window with a background color, if we don't fill it the pixels of the last frame will be drawn again
        canvas.fill(0xFFFFFFFF);

        // Ensure that we don't draw pixels outside of the canvas
        let max_pixels_to_draw = canvas.size().product();
        let pixels_to_draw = self.pixels_to_draw.min(max_pixels_to_draw);

        // Draw a red color for each pixel
        canvas.raw_buffer()[0..pixels_to_draw].fill(0xFFFF0000);
    }
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

    state.run(window_config).expect("Error running game");
}
