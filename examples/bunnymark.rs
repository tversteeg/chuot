//! An inaccurate benchmark, but still fun to show.
//!
//! The `bunnymark.png` sprite image for this example is:
//! {{ img(src="/assets/bunnymark.png" alt="Sprite") }}
//! With the following `bunnymark.ron` RON configuration file for positioning the center of the sprite:
//! ```ron
//! (offset: Middle)
//! ```

use chuot::{config::RotationAlgorithm, Config, Context, Game, MouseButton};

/// How many bunnies to spawn per frame when clicking.
const BUNNIES_TO_SPAWN: usize = 500;
/// How much gravity is applied each second.
const GRAVITY: f32 = 98.1;
/// Border at which the bunnies bounce off from the edges.
const BORDER: f32 = 10.0;

/// A single bunny instance to draw.
struct Bunny {
    /// Absolute X position in pixels on the buffer.
    x: f32,
    /// Absolute Y position in pixels on the buffer.
    y: f32,
    /// Horizontal velocity applied every second.
    velocity_x: f32,
    /// Vertical velocity applied every second.
    velocity_y: f32,
}

/// Define a game state for our example.
#[derive(Default)]
struct GameState {
    /// Bunnies to draw.
    bunnies: Vec<Bunny>,
}

impl Game for GameState {
    /// Update the game.
    fn update(&mut self, ctx: Context) {
        // Spawn the bunnies from the mouse
        if ctx.mouse_held(MouseButton::Left) {
            // Spawn many bunnies when clicking
            for _ in 0..BUNNIES_TO_SPAWN {
                self.bunnies.push(Bunny {
                    x: BORDER,
                    y: BORDER,
                    velocity_x: chuot::random(20.0, 100.0),
                    velocity_y: chuot::random(-1.0, 1.0),
                });
            }
        }

        // Load the context values outside of a hot loop, since all `ctx.` calls go through an `Rc<Refcell<..>>`

        // Get the deltatime once
        let dt = ctx.delta_time();

        // Get the size once
        let boundary_width = ctx.width() - BORDER;
        let boundary_height = ctx.height() - BORDER;

        // Remove all bunnies that are dead, and update all other bunnies
        self.bunnies.iter_mut().for_each(|bunny| {
            // Update the bunny
            bunny.x += bunny.velocity_x * dt;
            bunny.y += bunny.velocity_y * dt;

            // Bounce the bunnies on the left and right edges of the screen
            if bunny.x < BORDER {
                bunny.x = BORDER;
                bunny.velocity_x = -bunny.velocity_x;
            } else if bunny.x > boundary_width {
                bunny.x = boundary_width;
                bunny.velocity_x = -bunny.velocity_x;
            }

            // Bounce the bunnies when they hit the bottom of the screen
            if bunny.y > boundary_height {
                bunny.y = boundary_height;
                bunny.velocity_y = -bunny.velocity_y;
            }

            // Apply gravity
            bunny.velocity_y += GRAVITY * dt;
        });
    }

    /// Render the game.
    fn render(&mut self, ctx: Context) {
        // Draw all bunnies
        ctx.sprite("bunnymark")
            .draw_multiple_translated(self.bunnies.iter().map(|bunny| (bunny.x, bunny.y)));

        // Draw a basic FPS counter with the amount of bunnies
        ctx.text(
            "Beachball",
            &format!(
                "FPS: {:.1}\nBunnies (left click): {}",
                ctx.frames_per_second(),
                self.bunnies.len()
            ),
        )
        .use_ui_camera()
        .draw();
    }

    /// Configure the camera.
    fn init(&mut self, ctx: Context) {
        // Draw the bunnies with absolute coordinates from the top left
        ctx.main_camera().set_top_left();
    }
}

/// Setup and run the game.
fn main() {
    // Game configuration
    let config = Config {
        buffer_width: 360.0,
        buffer_height: 288.0,
        // Scale the pixels two times
        scaling: 2.0,
        // Disable vsync so we can see the effect of the bunnies on the FPS
        vsync: false,
        // We don't rotate the sprites so use the best performing algorithm
        rotation_algorithm: RotationAlgorithm::NearestNeighbor,
        ..Default::default()
    };

    // Spawn the window and run the 'game'
    GameState::default().run(chuot::load_assets!(), config);
}
