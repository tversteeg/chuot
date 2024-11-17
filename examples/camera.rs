//! Show how to work with the optional camera systems.
//!
//! There's a separate configuration for the main game camera and the UI camera.
//!
//! The `threeforms.png` sprite image for this example is:
//! {{ img(src="/assets/threeforms.png" alt="Sprite") }}
//! With the following `threeforms.ron` RON configuration file for positioning the center of the sprite:
//! ```ron
//! (pivot_x: Center, pivot_y: Center)
//! ```
//!
//! The `bunnymark.png` sprite image for this example is:
//! {{ img(src="/assets/bunnymark.png" alt="Sprite") }}
//! With the following `bunnymark.ron` RON configuration file for positioning the center of the sprite:
//! ```ron
//! (pivot_x: Center, pivot_y: Center)
//! ```

use chuot::{Config, Context, Game, KeyCode, MouseButton};

/// How fast the "player" moves.
const PLAYER_SPEED: f32 = 90.0;

/// Define a game state for our example.
#[derive(Default)]
struct GameState {
    /// Simulate the horizontal player movement, the camera will follow this.
    player_x: f32,
    /// Simulate the vertical player movement, the camera will follow this.
    player_y: f32,
    /// Previous horizontal position, for smooth rendering.
    previous_player_x: f32,
    /// Previous vertical position, for smooth rendering.
    previous_player_y: f32,
}

impl Game for GameState {
    /// Move the "player" around with keyboard input.
    fn update(&mut self, ctx: Context) {
        // Set the previous position before updating the current one
        self.previous_player_x = self.player_x;
        self.previous_player_y = self.player_y;

        // Move left on left arrow or 'a' key
        if ctx.key_held(KeyCode::ArrowLeft) || ctx.key_held(KeyCode::KeyA) {
            self.player_x -= PLAYER_SPEED * ctx.delta_time();
        }
        // Move right on right arrow or 'd' key
        if ctx.key_held(KeyCode::ArrowRight) || ctx.key_held(KeyCode::KeyD) {
            self.player_x += PLAYER_SPEED * ctx.delta_time();
        }
        // Move up on up arrow or 'w' key
        if ctx.key_held(KeyCode::ArrowUp) || ctx.key_held(KeyCode::KeyW) {
            self.player_y -= PLAYER_SPEED * ctx.delta_time();
        }
        // Move down on down arrow or 's' key
        if ctx.key_held(KeyCode::ArrowDown) || ctx.key_held(KeyCode::KeyS) {
            self.player_y += PLAYER_SPEED * ctx.delta_time();
        }

        // Follow the player with the main camera
        ctx.main_camera().follow((self.player_x, self.player_y));

        // Shake the camera when clicking, using the mouse position to determine the intensity and duration
        if let Some((mouse_x, mouse_y)) = ctx.mouse() {
            if ctx.mouse_pressed(MouseButton::Left) {
                // Horizontal mouse is the duration, vertical mouse the force
                ctx.main_camera().shake(1.0, mouse_x / 10.0, mouse_y / 4.0);
            }
        }
    }

    /// Render the game.
    fn render(&mut self, ctx: Context) {
        // Draw some bunnies to show the camera movement
        for y in -10..10 {
            for x in -10..10 {
                // Load a sprite asset and draw it statically
                ctx.sprite("bunnymark")
                    // Draw it at the same position every frame
                    .translate_x(x as f32 * 150.0)
                    .translate_y(y as f32 * 150.0)
                    // Draw the sprite on the screen
                    .draw();
            }
        }

        // Load a sprite asset and draw it
        ctx.sprite("threeforms")
            // Make the sprite move, the camera will follow it
            .translate_x(self.player_x)
            .translate_y(self.player_y)
            // Make the sprite render smoothly by interpolating based on the render blending factor,
            // this is applicable when there's multiple update tick calls in a single render tick,
            // if we don't do this the player will move in a jittery motion
            .translate_previous_x(self.previous_player_x)
            .translate_previous_y(self.previous_player_y)
            // Draw the sprite on the screen
            .draw();

        // Draw the instructions
        let instructions = if let Some((mouse_x, mouse_y)) = ctx.mouse() {
            format!(
                "Arrow keys to move\n\nClick to shake camera:\n- Duration:  1.00 s\n- Amplitude: {:.2} px\n- Frequency: {:.2} hz",
                mouse_x / 10.0,
                mouse_y / 4.0,
            )
        } else {
            "Arrow keys to move\n\nClick to shake camera".to_owned()
        };

        ctx.text("Beachball", &instructions).use_ui_camera().draw();
    }

    /// Setup the camera.
    fn init(&mut self, ctx: Context) {
        // Follow the camera slowly horizontally
        ctx.main_camera().set_lerp_x(0.1);
        // Follow the camera quickly vertically
        ctx.main_camera().set_lerp_y(0.5);
    }
}

/// Open an empty window.
fn main() {
    // Game configuration
    let config = Config {
        buffer_width: 360.0,
        buffer_height: 288.0,
        // Apply a minimum of 2 times scaling for the buffer
        // Will result in a minimum, and on web exact, window size of 720x576
        scaling: 2.0,
        ..Default::default()
    };

    // Spawn the window and run the 'game'
    GameState::default().run(chuot::load_assets!(), config);
}
