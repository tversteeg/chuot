use pixel_game_lib::{
    audio::Audio,
    vek::Vec2,
    window::{Input, KeyCode, MouseButton, WindowConfig},
    PixelGame, RenderContext,
};

/// Define a game state with a simple counter.
struct GameState;

impl PixelGame for GameState {
    // Update loop exposing input events we can handle, this is where you would handle the game logic
    fn update(&mut self, input: &Input, _mouse_pos: Option<Vec2<f64>>, _dt: f64) -> bool {
        // Play a sound when the mouse is pressed
        if input.mouse_released(MouseButton::Left) {
            pixel_game_lib::asset::<Audio>("switch31").play();
        }

        // Exit when escape is pressed
        input.key_pressed(KeyCode::Escape)
    }

    // Don't draw anything for this example
    fn render(&mut self, ctx: &mut RenderContext) {
        /*
        // Draw the pixels
        Font::default().render_centered(
            "Click to play a sound",
            Vec2::from_slice(&canvas.size().into_array()).as_() / 2.0,
            canvas,
        );
        */
    }
}

/// Run the game.
fn main() {
    // Start the game with defaults for the window
    GameState
        .run(WindowConfig::default())
        .expect("Error running game");
}
