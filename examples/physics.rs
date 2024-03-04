use miette::Result;
use pixel_game_lib::{
    canvas::Canvas,
    physics::{
        collision::shape::Shape,
        rigidbody::{RigidBodyBuilder, RigidBodyHandle},
        Physics, PhysicsSettings,
    },
    vek::{Extent2, Vec2},
    window::{Input, KeyCode, MouseButton, WindowConfig},
    PixelGame,
};

/// Game state passed around the update and render functions.
#[derive(Default)]
pub struct State {
    /// Physics engine state.
    physics: Physics,
    /// All spawned boxes.
    objects: Vec<RigidBodyHandle>,
    /// Which shape to spawn when clicking.
    spawn_shape: u8,
}

impl PixelGame for State {
    // Update loop exposing input events we can handle, this is where you would handle the game logic
    fn update(&mut self, input: &Input, mouse_pos: Option<Vec2<usize>>, dt: f32) -> bool {
        self.physics.step(dt as f64, &PhysicsSettings::default());

        // Spawn a box when the mouse is pressed
        if input.mouse_released(MouseButton::Left) {
            if let Some(mouse_pos) = mouse_pos {
                // Choose a shape to spawn
                let shape = match self.spawn_shape {
                    // Simple circle
                    0 => Shape::circle(10.0),
                    // Simple square
                    1 => Shape::rectangle(Extent2::new(20.0, 20.0)),
                    // Simple triangle
                    2 => Shape::triangle(
                        Vec2::new(-10.0, -10.0),
                        Vec2::new(10.0, 10.0),
                        Vec2::new(-10.0, 10.0),
                    ),
                    _ => unreachable!(),
                };

                // Rotate through the shapes
                self.spawn_shape += 1;
                if self.spawn_shape > 2 {
                    self.spawn_shape = 0;
                }

                // Spawn a falling object
                let rigidbody = RigidBodyBuilder::new(mouse_pos.as_())
                    .with_collider(shape)
                    .with_density(0.001)
                    .with_friction(0.3)
                    .with_restitution(0.2)
                    .spawn(&mut self.physics);

                self.objects.push(rigidbody);
            }
        }

        // Exit when escape is pressed
        input.key_pressed(KeyCode::Escape)
    }

    // Render loop exposing the pixel buffer we can mutate
    fn render(&mut self, canvas: &mut Canvas<'_>) {
        // Reset the canvas
        canvas.fill(0xFFEFEFEF);

        // Draw the colliders for the physics system
        self.physics
            .debug_info_vertices()
            .into_iter()
            .for_each(|vertices| draw_vertices(&vertices, canvas));
    }
}

/// Open an empty window.
fn main() -> Result<()> {
    // Window configuration with default pixel size and scaling
    let window_config = WindowConfig::default();

    // Create the shareable game state
    let mut state = State::default();

    // Spawn a big platform the boxes can drop on
    // The reference to the rigidbody must stay in scope otherwise it will be removed from the physics engine
    let _platform = RigidBodyBuilder::new_static(
        // Place the platform at the center of the screen
        Vec2::new(
            window_config.buffer_size.w / 2,
            window_config.buffer_size.h / 2,
        )
        .as_(),
    )
    // Create a big rectangle collider for it
    .with_collider(Shape::rectangle(
        Extent2::new(window_config.buffer_size.w / 2, 50).as_(),
    ))
    .with_friction(0.7)
    .with_restitution(0.0)
    .spawn(&mut state.physics);

    // Open the window and start the game-loop
    state.run(window_config)
}

/// Draw the vertices of a shape with lines between them.
fn draw_vertices(vertices: &[Vec2<f64>], canvas: &mut Canvas) {
    if vertices.is_empty() {
        return;
    }

    // Draw a line between each vertex and the next
    vertices
        .iter()
        .chain(std::iter::once(&vertices[0]))
        .reduce(|prev, cur| {
            canvas.draw_line(prev.as_(), cur.as_(), 0xFF000000);

            cur
        });
}
