use line_drawing::Bresenham;
use miette::Result;
use pixel_game_lib::{
    canvas::Canvas,
    gui::{
        button::{Button, ButtonRef},
        label::{Label, LabelRef},
        Gui, GuiBuilder, Widget,
    },
    physics::{collision::shape::Shape, rigidbody::RigidBodyBuilder, Physics, PhysicsSettings},
    vek::Vec2,
    window::{Key, WindowConfig},
};
use taffy::{prelude::Size, style::Style};
use vek::Extent2;

/// Game state passed around the update and render functions.
#[derive(Default)]
pub struct State {
    /// Physics engine state.
    physics: Physics,
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

            // Exit when escape is pressed
            input.key_pressed(Key::Escape)
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
            draw_line(prev.as_(), cur.as_(), canvas);

            cur
        });
}

/// Draw a line using the bresenham algorithm.
fn draw_line(start: Vec2<f64>, end: Vec2<f64>, canvas: &mut Canvas) {
    for (x, y) in Bresenham::new(
        (start.x as i32, start.y as i32),
        (end.x as i32, end.y as i32),
    ) {
        canvas.set_pixel(Vec2::new(x, y).as_(), 0)
    }
}
