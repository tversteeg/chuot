use miette::{IntoDiagnostic, Result};
use pixel_game_lib::{
    font::Font,
    gui::{button::Button, Gui, GuiBuilder, Widget},
    vek::Vec2,
    window::{Key, WindowConfig},
};
use taffy::{prelude::Size, style::Style};

/// Open an empty window.
fn main() -> Result<()> {
    // Window configuration with default pixel size and scaling
    let window_config = WindowConfig {
        ..Default::default()
    };

    // Load the font depending on the feature flag
    let font = Font::default();

    // Create a new Gui
    let mut gui = GuiBuilder::new(Style {
        // Use the amount of pixels as the calculation size
        size: Size::from_points(
            window_config.buffer_size.w as f32,
            window_config.buffer_size.h as f32,
        ),
        ..Default::default()
    });

    // Create a button attached to the root
    let button_node = gui.add_widget(
        Button {
            ..Default::default()
        },
        Style {
            size: Size::from_points(30.0, 30.0),
            ..Default::default()
        },
        gui.root(),
    )?;

    // Open the window and start the game-loop
    pixel_game_lib::window(
        // Use the gui as the state
        gui.build(),
        window_config.clone(),
        // Update loop exposing input events we can handle, this is where you would handle the game logic
        move |gui, input, mouse_pos, _dt| {
            // Update the GUI to fill the screen
            gui.update_layout(Vec2::zero(), window_config.buffer_size.as_());

            // Update the button manually
            let button: &mut Button = gui.widget_mut(button_node).unwrap();
            button.update(input, mouse_pos);

            // Exit when escape is pressed
            input.key_pressed(Key::Escape)
        },
        // Render loop exposing the pixel buffer we can mutate
        move |gui, canvas, _dt| {
            // Reset the canvas
            canvas.fill(0xFFFFFFFF);

            // Render the button manually
            let button: &Button = gui.widget(button_node).unwrap();
            button.render(canvas);
        },
    )?;

    Ok(())
}
