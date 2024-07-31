//! Show how to interpolate a sprite in the render loop by keeping a previous state in the update loop.

use chuot::{Config, Context, Game, MouseButton};

/// Define a game state for our example.
#[derive(Default)]
struct GameState {
    /// Current X position of the sprite.
    x: f32,
    /// Current Y position of the sprite.
    y: f32,
    /// Previous X position of the sprite, used to interpolate with in the render loop.
    previous_x: f32,
    /// Previous Y position of the sprite, used to interpolate with in the render loop.
    previous_y: f32,
    /// Show the effect of interpolation.
    interpolate: bool,
}

impl Game for GameState {
    /// Update the game.
    fn update(&mut self, ctx: Context) {
        // Store the previous position so it can be interpolated on in the render function
        self.previous_x = self.x;
        self.previous_y = self.y;

        // If the left mouse button is pressed add a new sprite
        if let Some((mouse_x, mouse_y)) = ctx.main_camera().mouse() {
            // Follow the mouse
            self.x += (mouse_x - self.x) * 0.1;
            self.y += (mouse_y - self.y) * 0.1;

            // Toggle interpolation when the mouse is released
            if ctx.mouse_released(MouseButton::Left) {
                self.interpolate = !self.interpolate;
            }
        }
    }

    /// Render the game.
    fn render(&mut self, ctx: Context) {
        // Which sprite to draw
        let sprite = ctx
            .sprite("threeforms")
            .translate_x(self.x)
            .translate_y(self.y);

        if self.interpolate {
            // Draw with interpolating with the blending factor to create a smooth transition
            sprite
                .translate_previous_x(self.previous_x)
                .translate_previous_y(self.previous_y)
                .draw();
        } else {
            // Draw the sprite following the mouse without interpolation
            sprite.draw();
        }

        // Draw a basic FPS counter
        ctx.text("Beachball", &format!("{:.1}", ctx.frames_per_second()))
            // Use the UI camera which draws the center in the top left
            .use_ui_camera()
            .draw();

        // Draw some instructions at the bottom of the screen
        ctx.text(
            "Beachball",
            &format!(
                "Interpolation: {}\nClick to toggle",
                if self.interpolate { "on" } else { "off" }
            ),
        )
            // Use the UI camera which draws the center in the top left
            .use_ui_camera()
        .translate_y(ctx.height() - 24.0)
        .draw();
    }
}

/// Open an empty window.
fn main() {
    // Game configuration
    let config = Config::default()
        .with_buffer_size((720.0, 576.0))
        .with_scaling(1.0)
        // Call update 10 times per second, this will show the interpolation
        // Normally you wouldn't want an update this slow
        .with_update_delta_time(10.0_f32.recip());

    // Spawn the window and run the 'game'
    GameState::default().run(chuot::load_assets!(), config);
}
