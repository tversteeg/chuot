use miette::Result;
use pixel_game_lib::{
    gui::{
        button::{Button, ButtonRef},
        label::{Label, LabelRef},
        Gui, GuiBuilder, Widget,
    },
    vek::Vec2,
    window::{Key, WindowConfig},
};
use taffy::{prelude::Size, style::Style};

/// Game state passed around the update and render functions.
pub struct State {
    /// How many times the button was pressed.
    button_pressed_amount: usize,
    /// The Gui so it can be updated.
    gui: Gui,
}

/// Open an empty window.
fn main() -> Result<()> {
    // Window configuration with default pixel size and scaling
    let window_config = WindowConfig {
        ..Default::default()
    };

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
    let button_node = gui.add_widget::<ButtonRef>(
        Button {
            label: Some("Press me!".to_string()),
            ..Default::default()
        },
        Style {
            size: Size::from_points(120.0, 20.0),
            ..Default::default()
        },
        gui.root(),
    )?;

    // Create a label showing how many times the button is pressed
    let label_node = gui.add_widget::<LabelRef>(
        Label {
            label: "Button not pressed yet".to_string(),
            ..Default::default()
        },
        Style {
            min_size: Size::from_points(120.0, 20.0),
            ..Default::default()
        },
        gui.root(),
    )?;

    // Create the shareable game state
    let state = State {
        button_pressed_amount: 0,
        gui: gui.build(),
    };

    // Open the window and start the game-loop
    pixel_game_lib::window(
        state,
        window_config.clone(),
        // Update loop exposing input events we can handle, this is where you would handle the game logic
        move |state, input, mouse_pos, _dt| {
            // Update the GUI to fill the screen
            state
                .gui
                .update_layout(Vec2::zero(), window_config.buffer_size.as_());

            // Update the button manually
            let button: &mut Button = state.gui.widget_mut(button_node).unwrap();

            // Handle the button press
            if button.update(input, mouse_pos) {
                state.button_pressed_amount += 1;

                // Update the button label
                let label: &mut Label = state.gui.widget_mut(label_node).unwrap();
                label.label = format!("Button pressed {} times", state.button_pressed_amount);
            }

            // Exit when escape is pressed
            input.key_pressed(Key::Escape)
        },
        // Render loop exposing the pixel buffer we can mutate
        move |state, canvas, _dt| {
            // Reset the canvas
            canvas.fill(0xFFFFFFFF);

            // Render the button manually
            let button: &Button = state.gui.widget(button_node).unwrap();
            button.render(canvas);

            // Render the text with how many times the button got pressed
            let label: &Label = state.gui.widget(label_node).unwrap();
            label.render(canvas);
        },
    )?;

    Ok(())
}
