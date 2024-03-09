use pixel_game_lib::{
    bitmap::BitMap,
    canvas::Canvas,
    sprite::SpriteOffset,
    vek::Extent2,
    vek::Vec2,
    window::{Input, KeyCode, MouseButton, WindowConfig},
    PixelGame,
};

/// What action clicking does.
#[derive(Default)]
enum Action {
    /// Set a single pixel.
    #[default]
    Set,
}

/// Define a game state with a bitmap that we will draw.
struct GameState {
    /// Bitmap with the same dimensions as the screen buffer.
    pub bitmap: BitMap,
    /// What action clicking does.
    pub action: Action,
}

impl PixelGame for GameState {
    /// Update loop exposing input events we can handle, this is where you would handle the game logic.
    fn update(&mut self, input: &Input, mouse_pos: Option<Vec2<usize>>, _dt: f32) -> bool {
        // Apply the bitmap action if the left mouse is clicked
        if input.mouse_held(MouseButton::Left) {
            if let Some(mouse_pos) = mouse_pos {
                match self.action {
                    // Set a single value
                    Action::Set => self.bitmap.set(mouse_pos, true),
                }
            }
        }

        // Switch between the actions if the right mouse button is clicked
        if input.mouse_released(MouseButton::Right) {
            // Toggle between actions
            self.action = match self.action {
                Action::Set => Action::Set,
            };
        }

        // Exit when escape is pressed
        input.key_pressed(KeyCode::Escape)
    }

    /// Render loop exposing the pixel buffer we can mutate.
    fn render(&mut self, canvas: &mut Canvas<'_>) {
        // Fill the window with a background color, if we don't fill it the pixels of the last frame will be drawn again
        canvas.fill(0xFFFFFFFF);

        // Convert the bitmap to a sprite where every bit is filled with black we can draw
        let image = self.bitmap.to_sprite(0xFF000000, SpriteOffset::LeftTop);

        // Draw the sprite on the canvas
        image.render(Vec2::zero(), canvas);
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

    // Empty bitmap filling the buffer
    let bitmap = BitMap::empty(window_config.buffer_size);
    let action = Action::default();

    // Active modifiable state
    let state = GameState { bitmap, action };

    state.run(window_config).expect("Error running game");
}
