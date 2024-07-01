//! Show how we can efficiently draw tens of thousands of particles.

use chuot::{Config, config::RotationAlgorithm, Context, context::MouseButton, Game};

/// How long a particle lives in seconds.
const PARTICLE_LIFE_SECS: f32 = 10.0;
/// How much gravity is applied each second.
const GRAVITY: f32 = 98.1;
/// Border at which the particles bounce off from the edges.
const BORDER: f32 = 10.0;

/// A single particle instance to draw.
struct Particle {
    /// Absolute X position in pixels on the buffer.
    x: f32,
    /// Absolute Y position in pixels on the buffer.
    y: f32,
    /// Horizontal velocity applied every second.
    velocity_x: f32,
    /// Vertical velocity applied every second.
    velocity_y: f32,
    /// How long the particle still lives.
    life: f32,
}

/// Define a game state for our example.
#[derive(Default)]
struct GameState {
    /// Particles currently alive to draw.
    particles: Vec<Particle>,
}

impl Game for GameState {
    /// Update the game.
    fn update(&mut self, ctx: Context) {
        // Spawn the particles from the mouse
        if let Some((mouse_x, mouse_y)) = ctx.mouse() {
            if ctx.mouse_pressed(MouseButton::Left) {
                // Spawn many particles when clicking
                for _ in 0..1000 {
                    self.particles.push(Particle {
                        x: mouse_x,
                        y: mouse_y,
                        velocity_x: chuot::random(-100.0, 100.0),
                        velocity_y: chuot::random(-100.0, 100.0),
                        life: PARTICLE_LIFE_SECS,
                    });
                }
            }

            if ctx.mouse_pressed(MouseButton::Right) {
                // Spawn many particles when clicking
                for _ in 0..10_000 {
                    self.particles.push(Particle {
                        x: mouse_x,
                        y: mouse_y,
                        velocity_x: chuot::random(-200.0, 200.0),
                        velocity_y: chuot::random(-200.0, 200.0),
                        life: PARTICLE_LIFE_SECS,
                    });
                }
            }

            if ctx.mouse_pressed(MouseButton::Middle) {
                // Spawn many particles when clicking
                for _ in 0..100_000 {
                    self.particles.push(Particle {
                        x: mouse_x,
                        y: mouse_y,
                        velocity_x: chuot::random(-300.0, 300.0),
                        velocity_y: chuot::random(-300.0, 300.0),
                        life: PARTICLE_LIFE_SECS,
                    });
                }
            }

            // Spawn a new particle at the mouse
            self.particles.push(Particle {
                x: mouse_x,
                y: mouse_y,
                velocity_x: chuot::random(-10.0, 10.0),
                velocity_y: chuot::random(-10.0, 10.0),
                life: PARTICLE_LIFE_SECS,
            });
        }

        // Load the context values outside of a hot loop, since all `ctx.` calls go through an `Rc<Refcell<..>>`

        // Get the deltatime once
        let dt = ctx.delta_time();

        // Get the size once
        let boundary_width = ctx.width() - BORDER;
        let boundary_height = ctx.height() - BORDER;

        // Remove all particles that are dead, and update all other particles
        self.particles.retain_mut(|particle| {
            // Update the particle
            particle.x += particle.velocity_x * dt;
            particle.y += particle.velocity_y * dt;

            // Bounce the particles on the left and right edges of the screen
            if particle.x < BORDER {
                particle.x = BORDER;
                particle.velocity_x = -particle.velocity_x;
            } else if particle.x > boundary_width {
                particle.x = boundary_width;
                particle.velocity_x = -particle.velocity_x;
            }

            // Bounce the particles when they hit the bottom of the screen
            if particle.y > boundary_height {
                particle.y = boundary_height;
                particle.velocity_y = -particle.velocity_y * 0.9;
            }

            // Apply gravity
            particle.velocity_y += GRAVITY * dt;

            // Reduce the particle's life
            particle.life -= dt;

            // Keep the particle if it's still alive
            particle.life > 0.0
        });
    }

    /// Render the game.
    fn render(&mut self, ctx: Context) {
        // Draw all particles
        ctx.sprite("crate").draw_multiple_translated(
            self.particles
                .iter()
                .map(|particle| (particle.x, particle.y)),
        );

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

        // Draw some instructions at the bottom of the screen
        ctx.text(
            "Beachball",
            "Left click to spawn 1000 particles\nRight click to spawn 10.000 particles\nMiddle mouse click to spawn 100.000 particles",
        )
            .translate_y(ctx.height() - 36.0)
            .draw();
    }
}

/// Open an empty window.
fn main() {
    // Game configuration
    let config = Config {
        buffer_width: 720.0,
        buffer_height: 576.0,
        // Don't scale the pixels
        scaling: 1.0,
        // Disable vsync so we can see the effect of the particles on the FPS
        vsync: false,
        // We don't rotate the sprites so use the best performing algorithm
        rotation_algorithm: RotationAlgorithm::NearestNeighbor,
        ..Default::default()
    };

    // Spawn the window and run the 'game'
    GameState::default().run(config);
}
