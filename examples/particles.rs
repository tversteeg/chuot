//! Show how we can efficiently draw thousands of particles.

use glamour::{Size2, Vector2};
use pixel_game_lib::{Context, GameConfig, KeyCode, MouseButton, PixelGame};

/// How long a particle lives in seconds.
const PARTICLE_LIFE_SECS: f32 = 1.0;

/// A single particle instance to draw.
struct Particle {
    /// Absolute position in pixels on the buffer.
    position: Vector2,
    /// Velocity applied every second.
    velocity: Vector2,
    /// How long the particle still lives.
    life: f32,
}

/// Define a game state for our example.
#[derive(Default)]
struct GameState {
    /// Particles currently alive to draw.
    particles: Vec<Particle>,
}

impl PixelGame for GameState {
    // Update and render the game
    fn tick(&mut self, ctx: Context) {
        // Exit when escape is pressed
        if ctx.key_pressed(KeyCode::Escape) {
            ctx.exit();

            return;
        }

        // Spawn the particles from the mouse
        if let Some(mouse) = ctx.mouse() {
            if ctx.mouse_pressed(MouseButton::Left) {
                // Spawn many particles when clicking
                for _ in 0..10 {
                    self.particles.push(Particle {
                        position: mouse,
                        velocity: Vector2::ZERO,
                        life: PARTICLE_LIFE_SECS,
                    });
                }
            }

            // Spawn a new particle at the mouse
            self.particles.push(Particle {
                position: mouse,
                velocity: Vector2::ZERO,
                life: PARTICLE_LIFE_SECS,
            });
        }

        // Get the deltatime once
        let dt = ctx.delta_time();

        // Remove all particles that are dead, and update all other particles
        self.particles.retain_mut(|particle| {
            // Update the particle
            particle.position += particle.velocity * dt;
            particle.life -= dt;

            // Keep the particle if it's still alive
            particle.life > 0.0
        });

        // Draw all particles
        // Will be loaded from disk if the `hot-reloading` feature is enabled, otherwise it will be embedded in the binary
        ctx.sprite("crate")
            .draw_multiple_translated(self.particles.iter().map(|particle| particle.position));

        // Draw a basic FPS counter
        let fps = dt.recip();
        ctx.text("Beachball", &format!("{fps:.1}")).draw();
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
