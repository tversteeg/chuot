use pixel_game_lib::{
    vek::Extent2,
    window::{KeyCode, WindowConfig},
};
use vek::{Disk, Vec2};

/// Open an empty window.
fn main() {
    // Window configuration with huge pixels
    let window_config = WindowConfig {
        buffer_size: Extent2::new(64, 64),
        scaling: 8,
        ..Default::default()
    };

    // Open the window and start the game-loop
    pixel_game_lib::window(
        // Keep track of the mouse as our "game state"
        Vec2::zero(),
        window_config.clone(),
        // Update loop exposing input events we can handle, this is where you would handle the game logic
        |state, input, mouse, _dt| {
            // Set the mouse position as the game state
            if let Some(mouse) = mouse {
                *state = mouse;
            }

            // Exit when escape is pressed
            input.key_pressed(KeyCode::Escape)
        },
        // Render loop exposing the pixel buffer we can mutate
        move |mouse, canvas, _dt| {
            // Reset the canvas with a white color
            canvas.fill(0xFFFFFFFF);

            // Draw a gray circle with the radius being the distance of the center to the mouse
            let circle_center = Vec2::new(50.0, 50.0);
            let dist_from_mouse = mouse.as_().distance(circle_center);
            canvas.draw_circle(Disk::new(circle_center, dist_from_mouse), 0xFF999999);
            // Draw a darker gray circle outline on top of the circle
            canvas.draw_circle_outline(Disk::new(circle_center, dist_from_mouse), 0xFF333333);

            // Draw a light green blue triangle with one corner being snapped to the mouse
            canvas.draw_triangle(
                [Vec2::new(45.0, 5.0), Vec2::new(60.0, 8.0), mouse.as_()],
                0xFF99FF99,
            );

            // Draw a light blue quadrilateral with one corner being snapped to the mouse
            canvas.draw_quad(
                [
                    Vec2::new(5.0, 5.0),
                    Vec2::new(20.0, 8.0),
                    Vec2::new(8.0, 30.0),
                    mouse.as_(),
                ],
                0xFF9999FF,
            );

            // Draw a black line from the center of the canvas to our mouse
            canvas.draw_line(
                (window_config.buffer_size.as_() / 2.0).into(),
                mouse.as_(),
                0xFF000000,
            );

            // Draw a red pixel under the mouse
            canvas.set_pixel(mouse.as_(), 0xFFFF0000);
        },
    )
    .expect("Error opening window");
}
