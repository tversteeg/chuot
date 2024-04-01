//! Show how a sprite can be loaded from disk and rendered multiple times..

use glamour::{Rect, Size2, Vector2};
use pixel_game_lib::{Context, GameConfig, KeyCode, MouseButton, PixelGame};

/// A single sprite instance to draw.
struct Sprite {
    /// Absolute position in pixels on the buffer.
    position: Vector2,
    /// Rotation in radians.
    rotation: f32,
}

/// Define a game state for our example.
#[derive(Default)]
struct GameState {
    /// Sprites to draw.
    sprites: Vec<Sprite>,
}

impl PixelGame for GameState {
    // Update and render the game
    fn tick(&mut self, ctx: Context) {
        // Exit when escape is pressed
        if ctx.key_pressed(KeyCode::Escape) {
            ctx.exit();

            return;
        }

        // If the left mouse button is pressed add a new sprite
        if let Some(mouse) = ctx.mouse() {
            if ctx.mouse_pressed(MouseButton::Left) {
                // Spawn a new sprite in the render loop
                self.sprites.push(Sprite {
                    position: mouse,
                    rotation: 0.0,
                });
            }

            // If the middle mouse button is held draw a pixel on the sprite
            if ctx.mouse_held(MouseButton::Middle) {
                ctx.update_sprite_pixels(
                    "threeforms",
                    Rect::new(mouse, Size2::splat(1.0)),
                    [0xFF00FF00],
                );
            }
        }

        // If the right mouse button is held rotate every sprite a tiny bit
        if ctx.mouse_held(MouseButton::Right) {
            self.sprites
                .iter_mut()
                .for_each(|sprite| sprite.rotation += ctx.delta_time());
        }

        // Draw sprite, will be loaded from disk if the `hot-reloading` feature is enabled, otherwise it will be embedded in the binary
        for sprite in &self.sprites {
            ctx.sprite("threeforms")
                .translate(sprite.position)
                .rotate(sprite.rotation)
                .draw();
        }

        // Draw a basic FPS counter
        let fps = ctx.delta_time().recip();
        ctx.draw_text("Beachball", Vector2::ZERO, format!("{fps:.1}"));
        ctx.draw_text(
            "Beachball",
            Vector2::new(0.0, 240.0 - 30.0),
            "Left mouse: new sprite\nRight mouse: rotate\nMiddle mouse: update pixel",
        );
    }
}

/// Open an empty window.
fn main() {
    // Game configuration
    let config = GameConfig {
        buffer_size: Size2::new(320.0, 240.0),
        // Apply a minimum of 3 times scaling for the buffer
        // Will result in a minimum, and on web exact, window size of 960x720
        scaling: 3.0,
        ..Default::default()
    };

    // Spawn the window and run the 'game'
    GameState::default()
        .run(config)
        .expect("Error running game");
}
