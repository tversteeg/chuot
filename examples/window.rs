//! Show how an empty window is spawned using the window configuration.

use pixel_game_lib::{
    vek::Extent2,
    vek::Vec2,
    window::{Input, KeyCode, WindowConfig},
    PixelGame, RenderContext,
};

/// Define an empty game state, because we don't need any state for our window.
struct GameState {}

impl PixelGame for GameState {
    // Update loop exposing input events we can handle, this is where you would handle the game logic
    fn update(&mut self, input: &Input, _mouse_pos: Option<Vec2<f64>>, _dt: f64) -> bool {
        // Exit when escape is pressed
        input.key_pressed(KeyCode::Escape)
    }

    // Render loop must be implemented, even though we don't use it in this example
    fn render(&mut self, _ctx: &mut RenderContext) {}
}

/// Open an empty window.
fn main() {
    // Window configuration with huge pixels
    let window_config = WindowConfig {
        buffer_size: Extent2::new(64, 64),
        scaling: 8,
        ..Default::default()
    };

    // Spawn the window
    GameState {}.run(window_config).expect("Error running game");
}
