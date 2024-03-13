use pixel_game_lib::{
    sprite::Sprite,
    vek::Extent2,
    vek::Vec2,
    window::{Input, KeyCode, WindowConfig},
    PixelGame,
};

/// Define a game state with the mouse position for our example.
struct GameState(Vec2<f64>);

impl PixelGame for GameState {
    // Update loop exposing input events we can handle, this is where you would handle the game logic
    fn update(&mut self, input: &Input, mouse_pos: Option<Vec2<usize>>, _dt: f64) -> bool {
        // Store the mouse position for the render phase
        if let Some(mouse_pos) = mouse_pos {
            self.0 = mouse_pos.as_();
        }

        // Exit when escape is pressed
        input.key_pressed(KeyCode::Escape)
    }

    // Render loop exposing the pixel buffer we can mutate
    fn render(&mut self) -> Vec<Sprite> {
        // Draw sprite, will be loaded from disk if the `hot-reloading` feature is enabled, otherwise it will be embedded in the binary
        let mut sprite: Sprite = pixel_game_lib::asset_owned("crate");
        sprite.render(Vec2::zero());
        sprite.render(self.0);
        sprite.render(self.0 / 2.0);
        sprite.render(self.0 / 8.0);

        vec![sprite]
    }
}

/// Open an empty window.
fn main() {
    // Window configuration with huge pixels
    let window_config = WindowConfig {
        buffer_size: Extent2::new(64, 64),
        scaling: 8,
        ..Default::default()
    };

    GameState(Vec2::zero())
        .run(window_config)
        .expect("Error running game");
}
