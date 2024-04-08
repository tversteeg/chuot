//! Show how we can efficiently draw thousands of particles.

use glamour::{Size2, Vector2};
use pixel_game_lib::{Context, GameConfig, KeyCode, MouseButton, PixelGame};

/// How long a particle lives in seconds.
const PARTICLE_LIFE_SECS: f32 = 10.0;
/// How much gravity is applied each second.
const GRAVITY: f32 = 98.1;

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
    /// Update the game.
    fn update(&mut self, ctx: Context) {
        // Exit when escape is pressed
        if ctx.key_pressed(KeyCode::Escape) {
            ctx.exit();

            return;
        }

        // Spawn the particles from the mouse
        if let Some(mouse) = ctx.mouse() {
            if ctx.mouse_pressed(MouseButton::Left) {
                // Spawn many particles when clicking
                for _ in 0..1000 {
                    self.particles.push(Particle {
                        position: mouse,
                        velocity: Vector2::new(
                            pixel_game_lib::random_range(-100.0, 100.0),
                            pixel_game_lib::random_range(-100.0, 100.0),
                        ),
                        life: PARTICLE_LIFE_SECS,
                    });
                }
            }

            // Spawn a new particle at the mouse
            self.particles.push(Particle {
                position: mouse,
                velocity: Vector2::new(
                    pixel_game_lib::random_range(-10.0, 10.0),
                    pixel_game_lib::random_range(-10.0, 10.0),
                ),
                life: PARTICLE_LIFE_SECS,
            });
        }

        // Get the deltatime once
        let dt = ctx.delta_time();

        // Remove all particles that are dead, and update all other particles
        self.particles.retain_mut(|particle| {
            // Update the particle
            particle.position += particle.velocity * dt;

            // Bounce the particles on the left and right edges of the screen
            if particle.position.x < 10.0 {
                particle.position.x = 10.0;
                particle.velocity.x = -particle.velocity.x;
            } else if particle.position.x > 310.0 {
                particle.position.x = 310.0;
                particle.velocity.x = -particle.velocity.x;
            }

            // Bounce the particles when they hit the bottom of the screen
            if particle.position.y > 220.0 {
                particle.position.y = 220.0;
                particle.velocity.y = -particle.velocity.y * 0.9;
            }

            // Apply gravity
            particle.velocity.y += GRAVITY * dt;

            // Reduce the particle's life
            particle.life -= dt;

            // Keep the particle if it's still alive
            particle.life > 0.0
        });
    }

    /// Render the game.
    fn render(&mut self, ctx: Context) {
        // Draw all particles
        // Will be loaded from disk if the `hot-reloading` feature is enabled, otherwise it will be embedded in the binary
        ctx.sprite("crate")
            .draw_multiple_translated(self.particles.iter().map(|particle| particle.position));

        // Draw a basic FPS counter with the amount of particles
        ctx.text(
            "Beachball",
            &format!(
                "FPS: {:.1}\nParticles: {}",
                ctx.frames_per_second(),
                self.particles.len()
            ),
        )
        .draw();
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
        // Disable vsync so we can see the effect of the particles on the FPS
        vsync: false,
        ..Default::default()
    };

    // Spawn the window and run the 'game'
    GameState::default()
        .run(config)
        .expect("Error running game");
}
