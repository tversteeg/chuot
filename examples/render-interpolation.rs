//! Show how to interpolate a sprite in the render loop by keeping a previous state in the update loop.

use chuot::{Context, GameConfig, KeyCode, MouseButton, PixelGame};
use glamour::Vector2;

/// Define a game state for our example.
#[derive(Default)]
struct GameState {
    /// Current position of the sprite.
    position: Vector2,
    /// Previous position of the sprite, used to interpolate with in the render loop.
    previous_position: Vector2,
    /// Show the effect of interpolation.
    interpolate: bool,
}

impl PixelGame for GameState {
    /// Update the game.
    fn update(&mut self, ctx: Context) {
        // Exit when escape is pressed
        if ctx.key_pressed(KeyCode::Escape) {
            ctx.exit();

            return;
        }

        // Store the previous position so it can be interpolated on in the render function
        self.previous_position = self.position;

        // If the left mouse button is pressed add a new sprite
        if let Some(mouse) = ctx.mouse() {
            // Follow the mouse
            self.position += (mouse - self.position) * 0.1;

            // Toggle interpolation when the mouse is released
            if ctx.mouse_released(MouseButton::Left) {
                self.interpolate = !self.interpolate;
            }
        }
    }

    /// Render the game.
    fn render(&mut self, ctx: Context) {
        if self.interpolate {
            // Draw the sprite following the mouse with interpolation
            ctx.sprite("crate")
                .translate(
                    self.position * ctx.blending_factor()
                        + self.previous_position * (1.0 - ctx.blending_factor()),
                )
                .draw();
        } else {
            // Draw the sprite following the mouse without interpolation
            ctx.sprite("crate").translate(self.position).draw();
        }

        // Draw a basic FPS counter
        ctx.text("Beachball", &format!("{:.1}", ctx.frames_per_second()))
            .draw();

        // Draw some instructions at the bottom of the screen
        ctx.text(
            "Beachball",
            &format!(
                "Interpolation: {}\nClick to toggle",
                if self.interpolate { "on" } else { "off" }
            ),
        )
        .translate(Vector2::new(0.0, ctx.size().height - 24.0))
        .draw();
    }
}

/// Open an empty window.
fn main() {
    // Game configuration
    let config = GameConfig::default()
        .with_buffer_size((720.0, 576.0))
        .with_scaling(1.0)
        // Call update 10 times per second, this will show the interpolation
        // Normally you wouldn't want an update this slow
        .with_update_delta_time(10.0_f32.recip());

    // Spawn the window and run the 'game'
    GameState::default()
        .run(chuot::load_assets!(), config)
        .expect("Error running game");
}
