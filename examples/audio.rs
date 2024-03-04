use pixel_game_lib::{
    audio::Audio,
    canvas::Canvas,
    font::Font,
    vek::Vec2,
    window::{Input, KeyCode, MouseButton, WindowConfig},
    PixelGame,
};

/// Define a game state with a simple counter.
struct GameState;

impl PixelGame for GameState {
    // Update loop exposing input events we can handle, this is where you would handle the game logic
    fn update(&mut self, input: &Input, _mouse_pos: Option<Vec2<usize>>, _dt: f32) -> bool {
        // Play a sound when the mouse is pressed
        if input.mouse_released(MouseButton::Left) {
            pixel_game_lib::asset::<Audio>("switch31").play();
        }

        // Exit when escape is pressed
        input.key_pressed(KeyCode::Escape)
    }

    // Render loop exposing the pixel buffer we can mutate
    fn render(&mut self, canvas: &mut Canvas<'_>) {
        // Fill the window with a background color, if we don't fill it the pixels of the last frame will be drawn again
        canvas.fill(0xFFFFFFFF);

        // Draw the pixels
        Font::default().render_centered(
            "Click to play a sound",
            Vec2::from_slice(&canvas.size().into_array()).as_() / 2.0,
            canvas,
        );
    }
}

/// Run the game.
fn main() {
    // Start the game with defaults for the window
    GameState
        .run(WindowConfig::default())
        .expect("Error running game");
}
