use line_drawing::Bresenham;
use miette::Result;
use pixel_game_lib::{
    canvas::Canvas,
    physics::{
        collision::shape::Shape,
        rigidbody::{RigidBodyBuilder, RigidBodyHandle},
        Physics, PhysicsSettings,
    },
    vek::Vec2,
    window::{KeyCode, WindowConfig},
};
use vek::Extent2;

/// Game state passed around the update and render functions.
#[derive(Default)]
pub struct State {
    /// Physics engine state.
    physics: Physics,
    /// All spawned boxes.
    boxes: Vec<RigidBodyHandle>,
}

/// Open an empty window.
fn main() -> Result<()> {
    // Window configuration with default pixel size and scaling
    let window_config = WindowConfig {
        ..Default::default()
    };

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
    pixel_game_lib::window(
        state,
        window_config.clone(),
        // Update loop exposing input events we can handle, this is where you would handle the game logic
        move |state, input, mouse_pos, dt| {
            state.physics.step(dt as f64, &PhysicsSettings::default());

            // Spawn a box when the mouse is pressed
            if input.mouse_released(0) {
                if let Some(mouse_pos) = mouse_pos {
                    // Spawn a falling box
                    let rigidbody = RigidBodyBuilder::new(mouse_pos.as_())
                        .with_collider(Shape::rectangle(Extent2::new(20.0, 20.0)))
                        .with_density(0.001)
                        .with_friction(0.3)
                        .with_restitution(0.2)
                        .spawn(&mut state.physics);

                    state.boxes.push(rigidbody);
                }
            }

            // Exit when escape is pressed
            input.key_pressed(KeyCode::Escape)
        },
        // Render loop exposing the pixel buffer we can mutate
        move |state, canvas, _dt| {
            // Reset the canvas
            canvas.fill(0xFFEFEFEF);

            // Draw the colliders for the physics system
            state
                .physics
                .debug_info_vertices()
                .into_iter()
                .for_each(|vertices| draw_vertices(&vertices, canvas));
        },
    )?;

    Ok(())
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
            canvas.draw_line(prev.as_(), cur.as_(), 0);

            cur
        });
}
