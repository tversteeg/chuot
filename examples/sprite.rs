//! Show how a sprite can be loaded from disk and rendered multiple times..

use pixel_game_lib::{
    vek::Extent2,
    vek::Vec2,
    window::{Input, KeyCode, MouseButton, WindowConfig},
    PixelGame, RenderContext,
};

/// Define a game state for our example.
struct GameState {
    /// Sprites to draw.
    sprites: Vec<Vec2<f64>>,
}

impl PixelGame for GameState {
    // Update loop exposing input events we can handle, this is where you would handle the game logic
    fn update(&mut self, input: &Input, mouse_pos: Option<Vec2<usize>>, _dt: f64) -> bool {
        // If the mouse is pressed add a new sprite
        if let Some(mouse_pos) = mouse_pos {
            if input.mouse_pressed(MouseButton::Left) {
                // Spawn a new sprite in the render loop
                self.sprites.push(mouse_pos.as_());
            }
        }

        // Exit when escape is pressed
        input.key_pressed(KeyCode::Escape)
    }

    // Render loop exposing the pixel buffer we can mutate
    fn render(&mut self, ctx: &mut RenderContext) {
        // Draw sprite, will be loaded from disk if the `hot-reloading` feature is enabled, otherwise it will be embedded in the binary
        for sprite in &self.sprites {
            ctx.draw_sprite("crate", *sprite);
        }
    }
}

/// Open an empty window.
fn main() {
    // Window configuration with huge pixels
    let window_config = WindowConfig {
        buffer_size: Extent2::new(320, 240),
        scaling: 8,
        ..Default::default()
    };

    let sprites = vec![Vec2::zero()];

    // Spawn the window and run the 'game'
    GameState { sprites }
        .run(window_config)
        .expect("Error running game");
}
